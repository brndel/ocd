use std::fs;

#[derive(Clone)]
pub struct InteractionTest {
  pub name: String,
  pub file_path: String,
  pub command_arguments: Vec<String>,
  pub lines: Vec<InteractionLine>,
}

#[derive(Debug, Clone)]
pub struct InteractionLine {
  pub line_idx: usize,
  pub content: String,
  pub kind: InteractionLineKind,
}

#[derive(Debug, Clone)]
pub enum InteractionLineKind {
  Input,
  OutputLiteral,
  OutputRegex,
}

#[derive(Debug)]
pub struct InteractionParseError(String);

pub fn parse(file_path: &String) -> Result<InteractionTest, InteractionParseError> {
  let file_content = fs::read_to_string(&file_path).expect("could not read file");

  let mut name = String::new();
  let mut command_arguments: Vec<String> = Vec::new();
  let mut lines: Vec<InteractionLine> = Vec::new();

  for (line_idx, line) in file_content.lines().enumerate() {
    let char0 = line.chars().nth(0).unwrap_or('\x0b');
    let char1 = line.chars().nth(1).unwrap_or('\x0b');
    let stripped_line = &line[2.min(line.len())..];
    match char0 {
      '#' => match char1 {
        '#' => {
          name = stripped_line.trim().to_string();
        }
        _ => {
          // normal comment -> ignore
        }
      },
      '$' => {
        if char1 == '$' {
          command_arguments.push(stripped_line.trim().to_string());
        } else {
          return Err(InteractionParseError(format!("invalid line '{}'", line)))
        }
      }
      '>' => {
        if char1 == ' ' {
          lines.push(InteractionLine {
            line_idx,
            content: stripped_line.to_string(),
            kind: InteractionLineKind::Input,
          });
        } else {
          return Err(InteractionParseError(format!("invalid line '{}'", line)))
        }
      }
      '<' => match char1 {
        'r' => {
          lines.push(InteractionLine {
            line_idx,
            content: stripped_line.to_string(),
            kind: InteractionLineKind::OutputRegex,
          });
        }
        'l' => {
          lines.push(InteractionLine {
            line_idx,
            content: stripped_line.to_string(),
            kind: InteractionLineKind::OutputLiteral,
          });
        }
        _ => {
          return Err(InteractionParseError(format!("invalid line '{}'", line)))
        }
      },
      _ => {
        lines.push(InteractionLine {
          line_idx,
          content: line.to_string(),
          kind: InteractionLineKind::OutputLiteral,
        });
      }
    }
  }
  Ok(InteractionTest {
    name,
    file_path: file_path.to_string(),
    command_arguments,
    lines,
  })
}
