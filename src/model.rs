use std::fmt::Display;

use rand::RngExt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandType {
  Move,
  LineTo,
}

#[derive(Debug, Clone)]
pub struct Command {
  pub x: f64,
  pub y: f64,
  pub command_type: CommandType,
}

#[derive(Debug, Clone)]
pub struct Path {
  pub commands: Vec<Command>,
  pub width: f64,
  pub height: f64,
  pub color: String,
}

#[derive(Error, Debug)]
pub enum PathError {
  #[error("invalid path or unsupported command")]
  ParseError,
  #[error("regex error")]
  RegexError(#[from] regex::Error),
  #[error("unknown path error")]
  Unknown,
}

impl Command {
  pub fn new(x: f64, y: f64, command_type: CommandType) -> Command {
    Command { x, y, command_type }
  }

  pub fn offset(&self, x: f64, y: f64) -> Command {
    Command {
      x: self.x + x,
      y: self.y + y,
      command_type: self.command_type,
    }
  }

  pub fn scale(&self, x: f64, y: f64) -> Command {
    Command {
      x: self.x * x,
      y: self.y * y,
      command_type: self.command_type,
    }
  }

  /// Rotate the command aim point around the origin (0, 0).
  pub fn rotate(&self, angle: f64) -> Command {
    let x = self.x * angle.cos() - self.y * angle.sin();
    let y = self.x * angle.sin() + self.y * angle.cos();
    Command {
      x,
      y,
      command_type: self.command_type,
    }
  }

  // pub fn to_string(&self) -> String {
  //     match self.command_type {
  //         CommandType::Move => format!("M {} {} ", self.x, self.y),
  //         CommandType::LineTo => format!("L {} {} ", self.x, self.y),
  //     }
  // }
}

impl Display for Command {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // write!(f, "{}", self.to_string())
    match self.command_type {
      CommandType::Move => write!(f, "M {} {} ", self.x, self.y),
      CommandType::LineTo => write!(f, "L {} {} ", self.x, self.y),
    }
  }
}

impl Path {
  pub fn parse(path: &str) -> Result<Path, PathError> {
    let mut commands = Vec::new();
    let rx = regex::Regex::new(r"([ML])\s?(-?\d{1,}\.?\d{1,}?)\s(-?\d{1,}\.?\d{1,}?)")?;
    let mut max_x = 0.0;
    let mut min_x = 0.0;
    let mut max_y = 0.0;
    let mut min_y = 0.0;
    for cap in rx.captures_iter(path) {
      let command_type = match &cap[1] {
        "M" => CommandType::Move,
        "L" => CommandType::LineTo,
        _ => return Err(PathError::ParseError),
      };
      let x = cap[2].parse::<f64>().map_err(|_| PathError::ParseError)?;
      let y = cap[3].parse::<f64>().map_err(|_| PathError::ParseError)?;
      if x > max_x {
        max_x = x;
      } else if x < min_x {
        min_x = x;
      }
      if y > max_y {
        max_y = y;
      } else if y < min_y {
        min_y = y;
      }
      commands.push(Command::new(x, y, command_type));
    }
    // offset the original point to the center of the path
    let offset_x = (max_x + min_x) / 2.0;
    let offset_y = (max_y + min_y) / 2.0;

    for command in &mut commands {
      command.x -= offset_x;
      command.y -= offset_y;
    }

    let path = Path {
      commands,
      width: max_x - min_x,
      height: max_y - min_y,
      color: String::from("black"),
    };

    Ok(path)
  }

  pub fn scale(&self, x: f64, y: f64) -> Path {
    let mut commands = Vec::new();
    for command in &self.commands {
      commands.push(command.scale(x, y));
    }
    Path {
      commands,
      width: self.width * x,
      height: self.height * y,
      color: self.color.clone(),
    }
  }

  /// Rotate the path around the origin (0, 0).
  pub fn rotate(&self, angle: f64) -> Path {
    let mut commands = Vec::new();
    for command in &self.commands {
      commands.push(command.rotate(angle));
    }
    Path {
      commands,
      width: self.width,
      height: self.height,
      color: self.color.clone(),
    }
  }

  pub fn offset(&self, x: f64, y: f64) -> Path {
    let mut commands = Vec::new();
    for command in &self.commands {
      commands.push(command.offset(x, y));
    }
    Path {
      commands,
      width: self.width,
      height: self.height,
      color: self.color.clone(),
    }
  }

  pub fn with_color(&self, color: &str) -> Path {
    Path {
      commands: self.commands.clone(),
      width: self.width,
      height: self.height,
      color: String::from(color),
    }
  }

  pub fn random_split(&self) -> Vec<Path> {
    let mut rng = rand::rng();
    let mut paths = Vec::new();
    let mut commands = Vec::new();
    let mut break_limit = rng.random_range(2..=4);
    let mut start_cmd = self.commands[0].clone();
    for command in &self.commands {
      if commands.len() >= break_limit || command.command_type == CommandType::Move {
        if command.command_type == CommandType::LineTo {
          commands.push(command.clone());
        }

        if commands.len() > 1 {
          paths.push(Path {
            commands: commands.clone(),
            width: self.width,
            height: self.height,
            color: self.color.clone(),
          });
        }
        commands = Vec::new();
        start_cmd = command.clone();
        start_cmd.command_type = CommandType::Move;
        break_limit = rng.random_range(2..=4);
      } else {
        if commands.is_empty() {
          commands.push(start_cmd.clone());
        }
        commands.push(command.clone());
      }
    }

    if commands.len() > 1 {
      paths.push(Path {
        commands: commands.clone(),
        width: self.width,
        height: self.height,
        color: self.color.clone(),
      });
    }
    paths
  }

  // pub fn to_string(&self) -> String {
  //     let mut commands = String::new();
  //     for command in &self.commands {
  //         commands.push_str(&command.to_string());
  //     }
  //     // the stroke-width should be calculated by the path size
  //     format!(
  //         "<path d=\"{}\" stroke=\"{}\" stroke-width=\"{}\" fill=\"none\" />",
  //         commands.trim(),
  //         self.color,
  //         self.height / 12.0
  //     )
  // }
}

impl Display for Path {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut commands = String::new();
    for command in &self.commands {
      commands.push_str(&command.to_string());
    }
    // the stroke-width should be calculated by the path size
    write!(
      f,
      "<path d=\"{}\" stroke=\"{}\" stroke-width=\"{}\" fill=\"none\" />",
      commands.trim(),
      self.color,
      self.height / 12.0
    )
  }
}
