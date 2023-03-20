mod parser;
mod runner;
mod args;

use std::{
  io::{stdout, Write},
  path::Path,
  sync::mpsc,
  thread::{self, JoinHandle}, time::Duration,
};

use clap::Parser;
use parser::InteractionTest;
use regex::Regex;
use runner::RunnerError;

use crate::{runner::RunnerConfig, args::OcdArgs};

fn main() {
  let args = OcdArgs::parse();

  let project_dir = args.folder_path;
  let main_path = args.main_class;
  let interactions_dir = args.interactions_path;
  let interaction_file_pattern = args.pattern.unwrap_or_else(|| ".*\\.txt".to_string());
  let threads = args.threads.unwrap_or(true);
  let timeout = args.timeout.unwrap_or(1000);

  let path = Path::new(&project_dir);
  let out_path = path
    .join("out")
    .join("production")
    .join(path.components().last().unwrap());

  // println!("{:?}", out_path);

  let config = RunnerConfig {
        dir: path.to_str().unwrap().to_string(),
        command:  "java".to_string(),
        arguments:  vec!["-classpath".to_string(), out_path.to_str().unwrap().to_string(), main_path.to_string()],
        timeout: Duration::from_millis(timeout),
    };

  let interaction_path: &str = interactions_dir.as_str();

  let interactions = collect_interactions(interaction_path, &interaction_file_pattern);

  println!(
    "Found {} interactions in '{}'",
    interactions.len(),
    interaction_path
  );

  let fails = run_interactions(interactions, &config, threads);

  if fails.is_empty() {
    println!("All interactions passed 🎉");
  } else {
    for fail in fails {
      println!("{}", fail.to_string());
    }
  }

}

type InteractionResult = Result<(), RunnerError>;

fn run_interactions(interactions: Vec<InteractionTest>, config: &RunnerConfig, multihread: bool) -> Vec<RunnerError> {
  let mut stdout = stdout();

  let mut interaction_results: Vec<Option<InteractionResult>> =
    interactions.iter().map(|_| Option::None).collect();

  let mut is_done = false;

  let (tx, rx) = mpsc::channel::<(usize, InteractionResult)>();

  let mut threads: Vec<JoinHandle<()>> = Vec::new();

  let mut i = 0;

  for interaction in interactions {
    let interaction_moved = interaction.clone();
    let config_copy = config.clone();
    let tx_copy = tx.clone();
    let i_copy = i;
    let thread = thread::spawn(move || {
      let inter = interaction_moved;
      // thread::sleep(Duration::from_millis(100));
      
      let result = runner::run(inter, &config_copy);
      tx_copy.send((i_copy, result)).expect("failed to send");
    });
    i += 1;
    if multihread {
      threads.push(thread);
    } else {
      thread.join().expect("could not join thread");
    }
  }

  drop(tx);

  let create_progress_bar = |results: &Vec<Option<InteractionResult>>| {
    results
      .iter()
      .map(|r| match r {
        None => "⬜",
        Some(Ok(_)) => "✅",
        Some(Err(runner::RunnerError::Fail {
          interaction: _,
          line: _,
          expected: _,
          found: _,
          prev_output: _,
        })) => "🟥",
        Some(Err(runner::RunnerError::Error {
          interaction: _,
          line: _,
          error_message: _,
          error_code: _,
          prev_output: _,
        })) => "💀",
      })
      .collect::<Vec<&str>>()
      .join("")
  };

  let update_is_done = |results: &Vec<Option<InteractionResult>>| {
    for result in results {
      if let None = result {
        return false;
      }
    }

    return true;
  };

  print!("{}", create_progress_bar(&interaction_results));
  stdout.flush().expect("could not flush");

  while !is_done {
    // println!("waiting for finish");
    let (i, result) = rx.recv().expect("could not read from channel");
    // println!("received from {}", i);
    let res = &mut interaction_results[i];

    *res = Some(result);

    print!("\r{}", create_progress_bar(&interaction_results));
    stdout.flush().expect("could not flush");

    is_done = update_is_done(&interaction_results);
  }
  println!();

  for thread in threads {
    thread.join().expect("could not join thread");
  }

  let mut fails = Vec::new();

  for result in interaction_results {
    if let Some(Err(error)) = result {
      fails.push(error);
    }
  }

  fails
}

fn collect_interactions(path_name: &str, file_pattern: &str) -> Vec<InteractionTest> {
  let file_regex = Regex::new(file_pattern).expect("invalid regex");
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
