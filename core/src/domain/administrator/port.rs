use crate::{
    domain::administrator::entities::AdministratorRow,
    infrastructure::administrator::repositories::error::AdministratorError,
};

pub trait AdministratorRepository: Send + Sync {
    fn get_first_administrator(
        &self,
    ) -> impl Future<Output = Result<Option<AdministratorRow>, AdministratorError>> + Send;
}
