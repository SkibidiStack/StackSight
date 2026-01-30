use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualEnvSummary {
    pub total: usize,
    pub active: usize,
}
