use std::time::Duration;


#[derive(Clone)]
pub struct RunnerConfig {
  pub arguments: Vec<String>,
  pub command: String,
  pub timeout: Duration,
}