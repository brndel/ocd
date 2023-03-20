use std::{
    io::{stdout, Write},
    sync::mpsc,
    thread::{self},
};

use crate::{parser::InteractionTest, runner::single_runner};

use super::{RunnerConfig, RunnerError};

mod result_char {
    pub const NONE: &str = "⬜";
    pub const OK: &str = "✅";
    pub const FAIL: &str = "🟥";
    pub const ERROR: &str = "💀";
}

type InteractionResult = Result<(), RunnerError>;
type InteractionResultVec = Vec<Option<InteractionResult>>;

trait ToIcon {
    fn to_icon(&self) -> &'static str;
}

impl ToIcon for InteractionResult {
    fn to_icon(&self) -> &'static str {
        match self {
            Ok(_) => result_char::OK,
            Err(RunnerError::Fail { .. }) => result_char::FAIL,
            Err(RunnerError::Error { .. }) => result_char::ERROR,
        }
    }
}

impl ToIcon for Option<InteractionResult> {
    fn to_icon(&self) -> &'static str {
        match self {
            None => result_char::NONE,
            Some(result) => result.to_icon(),
        }
    }
}

fn create_progress_bar(results: &InteractionResultVec) -> String {
    results
        .iter()
        .map(|r| r.to_icon())
        .collect::<Vec<&str>>()
        .join("")
}

fn update_is_done(results: &InteractionResultVec) -> bool {
    for result in results {
        if let None = result {
            return false;
        }
    }

    return true;
}

fn print_flush(message: String) {
    print!("{}", message);
    stdout().flush().unwrap();
}

fn print_over(message: String) {
    print!("\r{}", message);
    stdout().flush().unwrap();
}

pub fn run_interactions(
    interactions: Vec<InteractionTest>,
    config: RunnerConfig,
    thread_count: u64,
) -> Vec<RunnerError> {
    if thread_count == 0 {
        run_sync(interactions, config)
    } else {
        run_parallel(interactions, config, thread_count)
    }
}

fn run_sync(interactions: Vec<InteractionTest>, config: RunnerConfig) -> Vec<RunnerError> {
    let mut fails = Vec::new();

    for interaction in interactions {
        print_flush(format!("{} {}", None.to_icon(), interaction.name));

        let result = single_runner::run(interaction, &config);

        print_over(result.to_icon().to_string());

        if let Err(error) = result {
            fails.push(error);
        }
        println!();
    }

    fails
}

fn run_parallel(
    interactions: Vec<InteractionTest>,
    config: RunnerConfig,
    thread_count: u64,
) -> Vec<RunnerError> {
    let mut results: InteractionResultVec = interactions.iter().map(|_| None).collect();

    let (start_tx, start_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();

    let runner_thread = thread::spawn(move || {
        let result_tx = result_tx;
        let interactions = interactions;
        let mut threads = Vec::new();

        let mut i = 0;

        loop {
            if i >= interactions.len() {
                break;
            }

            start_rx.recv().unwrap();

            let interaction = interactions.get(i).unwrap().to_owned();
            let config = config.clone();
            let tx = result_tx.clone();

            let handle = thread::spawn(move || {
                let i = i;
                let result = single_runner::run(interaction, &config);
                tx.send((i, result)).expect("could not send");
            });

            i += 1;
            threads.push(handle);
        }
        for thread in threads {
            thread.join().unwrap();
        }
    });

    print_flush(create_progress_bar(&results));

    let mut is_done = update_is_done(&results);

    let mut running_threads = 0;

    while !is_done {
        while running_threads < thread_count {
            start_tx.send(()).unwrap();
            running_threads += 1;
        }

        let (index, result) = result_rx.recv().unwrap();

        results[index] = Some(result);
        running_threads -= 1;
        print_over(create_progress_bar(&results));

        is_done = update_is_done(&results);
    }
    println!();

    runner_thread.join().unwrap();

    let mut fails: Vec<RunnerError> = Vec::new();

    for result in results {
        if let Some(Err(error)) = result {
            fails.push(error);
        }
    }

    fails
}
