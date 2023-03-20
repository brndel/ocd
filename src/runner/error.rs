use crate::parser::InteractionTest;


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