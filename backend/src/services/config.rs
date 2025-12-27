use crate::core::config::AppConfig;

pub struct ConfigService {
    pub config: AppConfig,
}

impl ConfigService {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}
