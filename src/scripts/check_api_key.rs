use std::sync::Arc;

use crate::app::AppContext;
use crate::models::ContainersError;

pub fn check_api_key(app: &Arc<AppContext>, api_key: &str) -> Result<(), ContainersError> {
    if app.settings.is_api_key_valid(api_key) {
        return Ok(());
    }

    Err(ContainersError::Unauthorized)
}
