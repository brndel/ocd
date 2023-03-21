mod config;
mod parser;
mod runner;

use std::{env::current_dir, fs, io::Read, path::Path, time::Duration};

use parser::InteractionTest;
use regex::Regex;

use crate::{config::OcdConfig, runner::RunnerConfig};

fn main() {
    let config_file_path = "ocd.toml";
    let file = fs::File::open(config_file_path);
    if let Err(_) = file {
        println!("could not find '{}'", config_file_path);
        return;
    }
    let mut file = file.unwrap();

    let mut file_str = String::new();
    if let Err(_) = file.read_to_string(&mut file_str) {
        println!("'{}' was found but could not be read", config_file_path);
    }

    let config = toml::from_str::<OcdConfig>(file_str.as_str());

    if let Err(e) = config {
        println!("{}", e);
        return;
    }

    let config = config.unwrap();

    let class_path = config.class_path;
    let main_path = config.main_class;
    let interactions_dir = config.interaction.path;
    let interaction_file_patterns = config
        .interaction
        .pattern
        .unwrap_or_else(|| vec![".*\\.txt".to_string()]);
    let runner_config = config.runner.unwrap_or_default();
    let threads = runner_config.thread_count.unwrap_or(0);
    let timeout = runner_config.timeout.unwrap_or(1000);

    let path = Path::new(&class_path);

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir().unwrap().join(path)
    };

    let class_path = absolute_path.to_str().unwrap().to_string();

    let config = RunnerConfig {
        command: "java".to_string(),
        arguments: vec!["-classpath".to_string(), class_path, main_path.to_string()],
        timeout: Duration::from_millis(timeout),
    };

    let interaction_path: &str = interactions_dir.as_str();

    let interactions = collect_interactions(interaction_path, &interaction_file_patterns);

    println!(
        "Found {} interactions in '{}'",
        interactions.len(),
        interaction_path
    );

    let fails = runner::run(interactions, config, threads);

    if fails.is_empty() {
        println!("All interactions passed 🎉");
    } else {
        for fail in fails {
            println!("{}", fail.to_string());
        }
    }
}

fn collect_interactions(path_name: &str, file_patterns: &Vec<String>) -> Vec<InteractionTest> {
    let file_pattern = format!("^({})$", file_patterns.join("|"));
    let file_regex = Regex::new(&file_pattern).expect("invalid regex");
    let interactions = collect_files(path_name, &file_regex)
        .into_iter()
        .map(|path| parser::parse(&path).expect("failed to parse interaction"));

    interactions.collect()
}

fn collect_files(path_name: &str, file_regex: &Regex) -> Vec<String> {
    let path = Path::new(path_name);

    if path.is_file() {
        let file_name = path.file_name().unwrap().to_str().unwrap();

        if file_regex.is_match(file_name) {
            return vec![path.to_string_lossy().to_string()];
        }

        return vec![];
    }

    let mut interactions = vec![];
    let entries = std::fs::read_dir(path).unwrap();

    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            interactions.append(&mut collect_files(path.to_str().unwrap(), file_regex));
        } else if path.is_file() {
            interactions.append(&mut collect_files(path.to_str().unwrap(), file_regex));
        }
    }

    interactions
}
