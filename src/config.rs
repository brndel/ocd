use serde::Deserialize;

#[derive(Deserialize)]
pub struct OcdConfig {
    pub class_path: String,
    pub main_class: String,
    pub interaction: OcdInteractionConfig,
    pub runner: OcdRunnerConfig,
}

#[derive(Deserialize)]
pub struct OcdInteractionConfig {
    pub path: String,
    pub pattern: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct OcdRunnerConfig {
    pub thread_count: Option<u64>,
    pub timeout: Option<u64>,
}
