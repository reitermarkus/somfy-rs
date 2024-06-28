use core::{fmt, str::FromStr};

#[derive(Debug)]
pub struct UnknownCommand;

impl fmt::Display for UnknownCommand {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Unknown command")
  }
}

impl std::error::Error for UnknownCommand {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    None
  }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Command {
  My       = 0x1 << 4,
  Up       = 0x2 << 4,
  MyUp     = 0x3 << 4,
  Down     = 0x4 << 4,
  MyDown   = 0x5 << 4,
  UpDown   = 0x6 << 4,
  MyUpDown = 0x7 << 4,
  Prog     = 0x8 << 4,
  SunFlag  = 0x9 << 4,
  Flag     = 0xA << 4,
}

impl FromStr for Command {
  type Err = UnknownCommand;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let commands = [
      ("my", Command::My),
      ("up", Command::Up),
      ("myup", Command::MyUp),
      ("down", Command::Down),
      ("mydown", Command::MyDown),
      ("updown", Command::UpDown),
      ("myupdown", Command::MyUpDown),
      ("prog", Command::Prog),
      ("sunflag", Command::SunFlag),
      ("flag", Command::Flag),
    ];

    for (string, variant) in commands {
      if s.eq_ignore_ascii_case(string) {
        return Ok(variant)
      }
    }

    Err(UnknownCommand)
  }
}
