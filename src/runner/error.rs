use std::fmt::Display;

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

fn style<D: Display>(message: D, code: &str) -> String {
    format!("\x1b[{}m{}\x1b[0m", code, message)
}

fn style_fail(message: &str) -> String {
    style(message, "1;91")
}

fn style_error(message: &str) -> String {
    style(message, "1;30;41")
}

fn style_bold<D: Display>(message: D) -> String {
    style(message, "1")
}

fn style_green<D: Display>(message: D) -> String {
    style(message, "32")
}

fn style_red<D: Display>(message: D) -> String {
    style(message, "91")
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
                    "{} {}:{}\n",
                    style_fail("Failed"),
                    interaction.file_path,
                    line
                )
                .as_str();
                str += format!(
                    "{} in line {}\n",
                    style_bold(&interaction.name),
                    style_bold(line)
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
                    "{} in {}:{}\n",
                    style_error("Error"),
                    interaction.file_path,
                    line
                )
                .as_str();
                str += format!(
                    "{} in line {} with error code {}\n",
                    style_bold(&interaction.name),
                    style_bold(line),
                    style_bold(error_code)
                )
                .as_str();
                str += format!("{}\n", error_message).as_str();

                (prev_output.to_owned(), error_message)
            }
        };

        str += style_bold("Previous output:\n").as_str();

        let mut prev_output_str = String::new();

        if prev_output.len() > 10 {
            prev_output_str += "[...]\n"
        } else {
            prev_output_str += "<start>\n"
        }

        prev_output.reverse();
        prev_output.truncate(10);
        prev_output.reverse();

        prev_output_str += prev_output.join("\n").as_str();

        prev_output_str += "\n";

        str += style_green(prev_output_str).as_str();

        str += style_red(error).as_str();

        str
    }
}
