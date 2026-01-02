use crate::http::common::{api_error::ApiError, response::Response};

#[utoipa::path(
    get,
    path = "/signin",
    tag = "drivers",
    responses(
        (status = 200, description = "Driver signed in successfully", body = ()),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn signin_driver() -> Result<Response<()>, ApiError> {
    Ok(Response::ok(()))
}
