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
