use serde::Deserialize;
use utoipa::IntoParams;
use validator::Validate;

#[derive(Deserialize, Validate, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetValueFormatingParams {
    #[validate(length(min = 1, message = "value must not be empty"))]
    pub value: String,
}
