use crate::{
    domain::employee::entities::EmployeeRow,
    infrastructure::employee::repositories::error::EmployeeError,
};

pub trait EmployeeRepository: Send + Sync {
    fn get_first_employee(
        &self,
    ) -> impl Future<Output = Result<Option<EmployeeRow>, EmployeeError>> + Send;
}
