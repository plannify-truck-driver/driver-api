use std::cmp::Ordering;

use tracing::{error, info, warn};

use plannify_driver_api_core::{
    application::DriverRepositories,
    domain::{storage::port::StorageRepository, workday::port::WorkdayDatabaseRepository},
};

/// Number of DB rows / S3 keys fetched per round-trip.
const BATCH_SIZE: i64 = 1_000;

pub async fn run(repos: &DriverRepositories) -> i32 {
    run_inner(
        &repos.workday_database_repository,
        &repos.storage_repository,
    )
    .await
}

/// S3 clients (including Garage) sometimes create empty "directory marker" objects
/// whose keys end with `/` (e.g. `drivers/`, `drivers/uuid/workdays/`).
/// These are virtual artifacts of the client — they have no DB counterpart and
/// must be ignored rather than treated as orphaned files.
fn filter_directory_markers(keys: Vec<String>) -> Vec<String> {
    keys.into_iter().filter(|k| !k.ends_with('/')).collect()
}

/// Streaming sorted-merge reconciliation.
///
/// Both the DB cursor (`ORDER BY s3_file_path ASC`, LIMIT/offset via cursor) and
/// the S3 listing (`list_objects_v2`, sorted lexicographically by the API) produce
/// their keys in ascending order.  We advance both streams simultaneously — just
/// like the merge step in merge-sort — and act on divergences immediately.
///
/// Memory footprint: O(BATCH_SIZE), regardless of the total number of documents.
async fn run_inner<WDB, SR>(workday_db: &WDB, storage: &SR) -> i32
where
    WDB: WorkdayDatabaseRepository,
    SR: StorageRepository,
{
    info!("Starting reconcile_documents job");

    // ── Initial load ─────────────────────────────────────────────────────────

    let mut db_buf = match workday_db
        .get_document_s3_paths_batch(None, BATCH_SIZE)
        .await
    {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to fetch initial DB batch: {}", e);
            return 1;
        }
    };
    let mut db_idx: usize = 0;

    let (mut s3_buf, mut s3_token) = match storage.list_objects_page(None, None).await {
        Ok((keys, token)) => (filter_directory_markers(keys), token),
        Err(e) => {
            error!("Failed to fetch initial S3 page: {}", e);
            return 1;
        }
    };
    let mut s3_idx: usize = 0;

    let mut orphaned_db = 0u32;
    let mut orphaned_s3 = 0u32;
    let mut failed = 0u32;

    // ── Streaming merge ───────────────────────────────────────────────────────

    loop {
        // Refill DB buffer: only when we've consumed the current batch AND it was
        // full (meaning there may be more rows).
        if db_idx >= db_buf.len() && db_buf.len() == BATCH_SIZE as usize {
            let after = db_buf.last().cloned();
            match workday_db
                .get_document_s3_paths_batch(after.as_deref(), BATCH_SIZE)
                .await
            {
                Ok(batch) => {
                    db_buf = batch;
                    db_idx = 0;
                }
                Err(e) => {
                    error!("Failed to fetch DB batch: {}", e);
                    failed += 1;
                    break;
                }
            }
        }

        // Refill S3 buffer: only when we've consumed the current page AND a
        // continuation token exists.
        if s3_idx >= s3_buf.len() && s3_token.is_some() {
            match storage.list_objects_page(None, s3_token.take()).await {
                Ok((keys, token)) => {
                    s3_buf = filter_directory_markers(keys);
                    s3_token = token;
                    s3_idx = 0;
                }
                Err(e) => {
                    error!("Failed to fetch S3 page: {}", e);
                    failed += 1;
                    break;
                }
            }
        }

        let db_done = db_idx >= db_buf.len();
        let s3_done = s3_idx >= s3_buf.len();

        match (db_done, s3_done) {
            (true, true) => break,

            // DB record exists but the corresponding S3 file is gone.
            (false, true) => {
                let path = &db_buf[db_idx];
                match workday_db.delete_document_by_s3_path(path).await {
                    Ok(()) => {
                        orphaned_db += 1;
                        info!(path, "Deleted orphaned DB record (no S3 file)");
                    }
                    Err(e) => {
                        failed += 1;
                        error!(path, error = ?e, "Failed to delete orphaned DB record");
                    }
                }
                db_idx += 1;
            }

            // S3 object exists but there is no corresponding DB record.
            (true, false) => {
                let key = &s3_buf[s3_idx];
                match storage.delete(key).await {
                    Ok(()) => {
                        orphaned_s3 += 1;
                        info!(key, "Deleted orphaned S3 object (no DB record)");
                    }
                    Err(e) => {
                        failed += 1;
                        warn!(key, error = ?e, "Failed to delete orphaned S3 object");
                    }
                }
                s3_idx += 1;
            }

            // Both streams have data — compare the current positions.
            (false, false) => {
                let db_path = db_buf[db_idx].as_str();
                let s3_key = s3_buf[s3_idx].as_str();

                match db_path.cmp(s3_key) {
                    Ordering::Equal => {
                        db_idx += 1;
                        s3_idx += 1;
                    }
                    Ordering::Less => {
                        match workday_db.delete_document_by_s3_path(db_path).await {
                            Ok(()) => {
                                orphaned_db += 1;
                                info!(path = db_path, "Deleted orphaned DB record (no S3 file)");
                            }
                            Err(e) => {
                                failed += 1;
                                error!(path = db_path, error = ?e, "Failed to delete orphaned DB record");
                            }
                        }
                        db_idx += 1;
                    }
                    Ordering::Greater => {
                        match storage.delete(s3_key).await {
                            Ok(()) => {
                                orphaned_s3 += 1;
                                info!(key = s3_key, "Deleted orphaned S3 object (no DB record)");
                            }
                            Err(e) => {
                                failed += 1;
                                warn!(key = s3_key, error = ?e, "Failed to delete orphaned S3 object");
                            }
                        }
                        s3_idx += 1;
                    }
                }
            }
        }
    }

    info!(
        orphaned_db,
        orphaned_s3, failed, "reconcile_documents job completed"
    );

    if failed > 0 { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::run_inner;
    use bytes::Bytes;
    use plannify_driver_api_core::domain::{
        storage::port::MockStorageRepository, workday::port::MockWorkdayDatabaseRepository,
    };

    async fn seed_db(db: &MockWorkdayDatabaseRepository, paths: &[&str]) {
        use plannify_driver_api_core::domain::workday::port::WorkdayDatabaseRepository;
        use uuid::Uuid;
        for (i, path) in paths.iter().enumerate() {
            db.create_workday_document(
                Uuid::new_v4(),
                1,
                2026 + i as i32,
                path.to_string(),
                "report.pdf".to_string(),
            )
            .await
            .unwrap();
        }
    }

    async fn seed_s3(storage: &MockStorageRepository, keys: &[&str]) {
        use plannify_driver_api_core::domain::storage::port::StorageRepository;
        for key in keys {
            storage
                .upload(key, Bytes::from("pdf"), "application/pdf")
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_in_sync_returns_success() {
        let db = MockWorkdayDatabaseRepository::new();
        let storage = MockStorageRepository::new();
        seed_db(&db, &["drivers/1/report.pdf"]).await;
        seed_s3(&storage, &["drivers/1/report.pdf"]).await;

        assert_eq!(run_inner(&db, &storage).await, 0);
    }

    #[tokio::test]
    async fn test_orphaned_db_record_is_deleted() {
        let db = MockWorkdayDatabaseRepository::new();
        let storage = MockStorageRepository::new();
        seed_db(&db, &["drivers/1/missing.pdf"]).await;

        let result = run_inner(&db, &storage).await;
        assert_eq!(result, 0);

        use plannify_driver_api_core::domain::workday::port::WorkdayDatabaseRepository;
        let remaining = db.get_all_document_s3_paths().await.unwrap();
        assert!(remaining.is_empty(), "orphaned DB record should be deleted");
    }

    #[tokio::test]
    async fn test_orphaned_s3_object_is_deleted() {
        let db = MockWorkdayDatabaseRepository::new();
        let storage = MockStorageRepository::new();
        seed_s3(&storage, &["drivers/1/orphan.pdf"]).await;

        let result = run_inner(&db, &storage).await;
        assert_eq!(result, 0);

        use plannify_driver_api_core::domain::storage::port::StorageRepository;
        assert!(
            storage.download("drivers/1/orphan.pdf").await.is_err(),
            "orphaned S3 object should have been deleted"
        );
    }

    #[tokio::test]
    async fn test_empty_both_returns_success() {
        let db = MockWorkdayDatabaseRepository::new();
        let storage = MockStorageRepository::new();
        assert_eq!(run_inner(&db, &storage).await, 0);
    }

    #[tokio::test]
    async fn test_multiple_orphans_both_sides() {
        let db = MockWorkdayDatabaseRepository::new();
        let storage = MockStorageRepository::new();

        seed_db(&db, &["drivers/1/ok.pdf", "drivers/2/gone.pdf"]).await;
        seed_s3(&storage, &["drivers/1/ok.pdf", "drivers/3/extra.pdf"]).await;

        let result = run_inner(&db, &storage).await;
        assert_eq!(result, 0);

        use plannify_driver_api_core::domain::{
            storage::port::StorageRepository, workday::port::WorkdayDatabaseRepository,
        };

        let remaining = db.get_all_document_s3_paths().await.unwrap();
        assert!(remaining.contains(&"drivers/1/ok.pdf".to_string()));
        assert!(!remaining.contains(&"drivers/2/gone.pdf".to_string()));

        assert!(storage.download("drivers/3/extra.pdf").await.is_err());
    }

    #[tokio::test]
    async fn test_sort_order_respected() {
        // Ensures the merge handles lexicographic ordering correctly: 'b' < 'c' but 'b' > 'a'
        let db = MockWorkdayDatabaseRepository::new();
        let storage = MockStorageRepository::new();

        seed_db(&db, &["drivers/a/report.pdf", "drivers/c/report.pdf"]).await;
        seed_s3(&storage, &["drivers/a/report.pdf", "drivers/b/report.pdf"]).await;

        let result = run_inner(&db, &storage).await;
        assert_eq!(result, 0);

        use plannify_driver_api_core::domain::{
            storage::port::StorageRepository, workday::port::WorkdayDatabaseRepository,
        };

        // "a" is in sync — must survive
        let remaining_db = db.get_all_document_s3_paths().await.unwrap();
        assert!(remaining_db.contains(&"drivers/a/report.pdf".to_string()));
        // "c" was only in DB — must be deleted
        assert!(!remaining_db.contains(&"drivers/c/report.pdf".to_string()));
        // "b" was only in S3 — must be deleted
        assert!(storage.download("drivers/b/report.pdf").await.is_err());
    }
}
