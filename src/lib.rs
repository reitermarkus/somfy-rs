use std::fmt;

mod command;
pub use command::{Command, UnknownCommand};

mod frame;
pub use frame::{Frame, SendFrame};

mod sender;
pub use sender::Sender;

mod spi_sender;
pub use spi_sender::SpiSender;

mod remote;
pub use remote::Remote;

pub enum Error<T, S> {
  TransmitError(T),
  StorageError(S),
}

impl<T, S> fmt::Debug for Error<T, S>
where
  T: fmt::Debug,
  S: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::TransmitError(err) => {
        write!(f, "TransmitError(")?;
        err.fmt(f)?;
        write!(f, ")")
      },
      Self::StorageError(err) => {
        write!(f, "StorageError(")?;
        err.fmt(f)?;
        write!(f, ")")
      },
    }
  }
}

impl<T, S> fmt::Display for Error<T, S>
where
  T: fmt::Display,
  S: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::TransmitError(err) => err.fmt(f),
      Self::StorageError(err) => err.fmt(f),
    }
  }
}

impl<T, S> std::error::Error for Error<T, S>
where
  T: std::error::Error + 'static,
  S: std::error::Error + 'static,
{
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::TransmitError(err) => Some(err),
      Self::StorageError(err) => Some(err),
    }
  }
}

pub trait RollingCodeStorage {
  type Error;

  fn persist(&mut self, remote: &Remote) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
  use ux::u24;

  use super::*;

  #[test]
  fn test_frame() {
    let rolling_code = 42;
    let remote_address = u24::new(0xFFAA11);

    let mut frame = Frame::builder()
      .key(0xA7)
      .command(Command::Up)
      .rolling_code(rolling_code)
      .remote_address(remote_address)
      .build()
      .expect("Failed to build frame");

    frame.deobfuscate();

    let command = frame.command_and_checksum & 0b11110000;
    let checksum = frame.command_and_checksum & 0b00001111;

    assert_eq!(frame.key, 0xA7);
    assert_eq!(command, Command::Up as u8);
    assert_eq!(checksum, 7);
    assert_eq!(u16::from_be_bytes(frame.rolling_code), rolling_code);
    assert_eq!(
      u24::new(
        frame.remote_address[0] as u32
          + ((frame.remote_address[1] as u32) << 8)
          + ((frame.remote_address[2] as u32) << 16)
      ),
      remote_address
    );
  }
}
