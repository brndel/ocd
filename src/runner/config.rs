use std::time::Duration;

#[derive(Clone)]
pub struct RunnerConfig {
    pub command: String,
    pub arguments: Vec<String>,
    pub timeout: Duration,
}
