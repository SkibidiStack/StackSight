use crate::core::config::AppConfig;

#[allow(dead_code)]
pub struct ConfigService {
    pub config: AppConfig,
}

#[allow(dead_code)]
impl ConfigService {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}
