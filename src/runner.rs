use std::io::{Read, Write};
use std::path::Path;
use std::process::{ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use regex::Regex;

use crate::parser::InteractionLineKind;
use crate::parser::InteractionTest;

#[derive(Clone)]
pub struct RunnerConfig {
  pub dir: String,
  pub arguments: Vec<String>,
  pub command: String,
}

pub enum RunnerError {
  Fail {
    interaction: InteractionTest,
    line: usize,
    expected: String,
    found: String,
    prev_output: Vec<String>,
  },
  Error {
    interaction: InteractionTest,
    line: usize,
    error_message: String,
    error_code: i32,
    prev_output: Vec<String>,
  },
}

impl ToString for RunnerError {
  fn to_string(&self) -> String {
    let mut str = String::new();

    let (mut prev_output, error) = match self {
      RunnerError::Fail {
        interaction,
        line,
        expected,
        found,
        prev_output,
      } => {
        str += format!(
          "\x1b[1;91mFailed\x1b[0m {}:{}\n",
          interaction.file_path, line
        )
        .as_str();
        str += format!(
          "\x1b[1m{}\x1b[0m in line \x1b[1m{}\x1b[0m\n",
          interaction.name, line
        )
        .as_str();
        str += format!("expected: '{}'\n", expected).as_str();
        str += format!("found:    '{}'\n", found).as_str();

        (prev_output.to_owned(), found)
      }
      RunnerError::Error {
        interaction,
        line,
        error_message,
        error_code,
        prev_output,
      } => {
        str += format!(
          "\x1b[1;30m\x1b[41mError\x1b[0m in {}:{}\n",
          interaction.file_path, line
        )
        .as_str();
        str += format!(
          "\x1b[1m{}\x1b[0m in line \x1b[1m{}\x1b[0m with error code \x1b[1m{}\x1b[0m\n",
          interaction.name, line, error_code
        )
        .as_str();
        str += format!("{}\n", error_message).as_str();

        (prev_output.to_owned(), error_message)
      }
    };

    str += "\x1b[1mPrevious ouput:\x1b[0m\n";

    str += "\x1b[32m";
    if prev_output.len() > 10 {
      str += "[...]\n"
    } else {
      str += "<start>\n"
    }

    prev_output.reverse();
    prev_output.truncate(10);
    prev_output.reverse();

    str += prev_output.join("\n").as_str();

    str += "\x1b[0m\n";

    str += format!("\x1b[91m{}\x1b[0m", error).as_str();

    str
  }
}

pub fn run(interaction: InteractionTest, config: &RunnerConfig) -> Result<(), RunnerError> {
  
  let current_dir = Path::new(&interaction.file_path).parent().unwrap().to_string_lossy().to_string();

  let mut child = Command::new(&config.command)
    .current_dir(current_dir)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .args(&config.arguments)
    .args(&interaction.command_arguments)
    .spawn()
    .expect("programm failed to start");

  let mut stdin = child.stdin.take().expect("failed to get stdin");
  let mut stdout = child.stdout.take().expect("failed to get stdout");

  let inout_thread: JoinHandle<Result<(), RunnerError>> = thread::spawn(move || {
    let interaction_lines = interaction.lines.clone();
    let mut line_index = 0;
    let mut prev_lines = Vec::new();

    while let Some(inter_line) = &interaction_lines.get(line_index) {
      line_index += 1;

      match inter_line.kind {
        InteractionLineKind::Input => {
          let mut line = inter_line.content.to_string();
          line.push('\n');
          stdin
            .write(line.as_bytes())
            .expect("could not write to stdin");
          let mut out_line = "> ".to_string();
          out_line += inter_line.content.as_str();
          prev_lines.push(out_line);
        }
        InteractionLineKind::OutputLiteral => {
          let result = read_line(stdout);

          if let Some((line, out)) = result {
            stdout = out;
            if line == inter_line.content {
              prev_lines.push(line);
              continue;
            }

            return Err(RunnerError::Fail {
              interaction,
              line: inter_line.line_idx,
              expected: inter_line.content.to_string(),
              found: line.to_string(),
              prev_output: prev_lines,
            });
          }

          return Err(RunnerError::Fail {
            interaction,
            line: inter_line.line_idx,
            expected: inter_line.content.to_string(),
            found: "<timeout>".to_string(),
            prev_output: prev_lines,
          });
        }
        InteractionLineKind::OutputRegex => {
          let result = read_line(stdout);

          if let Some((line, out)) = result {
            stdout = out;
            if Regex::new(&inter_line.content)
              .unwrap()
              .is_match(line.as_str())
            {
              prev_lines.push(line);
              continue;
            }

            return Err(RunnerError::Fail {
              interaction,
              line: inter_line.line_idx,
              expected: inter_line.content.to_string(),
              found: line.to_string(),
              prev_output: prev_lines,
            });
          }

          return Err(RunnerError::Fail {
            interaction,
            line: inter_line.line_idx,
            expected: inter_line.content.to_string(),
            found: "<timeout>".to_string(),
            prev_output: prev_lines,
          });
        }
      }
    }

    if let Some((line, _)) = read_line(stdout) {
      return Err(RunnerError::Fail {
        interaction,
        line: 0,
        expected: "<EOF>".to_string(),
        found: line,
        prev_output: prev_lines,
      });
    }
    return Ok(());
  });

  let child_result = child.wait().expect("could not wait for child");

  let result = inout_thread.join().expect("could not join inout_thread");

  if !child_result.success() {
    let mut stderr = child.stderr.take().expect("Failed to get stderr");
    let mut error_message = String::new();
    stderr
      .read_to_string(&mut error_message)
      .expect("could not read from stderr");

    if error_message.contains("java.util.NoSuchElementException")
      && error_message.contains("java.util.Scanner")
    {
    } else {
      if let Err(result) = result {
        if let RunnerError::Fail {
          interaction,
          line,
          expected: _,
          found: _,
          prev_output,
        } = result
        {
          return Err(RunnerError::Error {
            interaction,
            line,
            error_message,
            error_code: child_result.code().unwrap(),
            prev_output,
          });
        }
      }
      panic!("child had error while thread did not");
    }
  }

  return result;
}

fn read_line(mut stdout: ChildStdout) -> Option<(String, ChildStdout)> {
  let (tx, rx) = mpsc::channel();

  thread::spawn(move || {
    let mut line_buffer = Vec::new();
    let mut buf: [u8; 1] = [0];
    loop {
      let result = stdout.read_exact(&mut buf);
      if let Err(_) = result {
        return;
      }
      if buf[0] == ('\n' as u8) {
        break;
      }
      line_buffer.push(buf[0]);
    }

    let string = String::from_utf8_lossy(&line_buffer).to_string();

    tx.send((string, stdout)).expect("could not send");
  });

  let received = rx.recv_timeout(Duration::from_millis(1000)).ok();

  received
}
