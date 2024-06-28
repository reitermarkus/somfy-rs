use embedded_hal::{delay::DelayNs, digital::OutputPin};
use serde::{Deserialize, Serialize};
use ux::u24;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
  address: u24,
  rolling_code: u16,
}

impl Remote {
  pub fn new(address: u24, rolling_code: u16) -> Self {
    Self { address, rolling_code }
  }

  pub fn address(&self) -> u24 {
    self.address
  }

  pub fn rolling_code(&self) -> u16 {
    self.rolling_code
  }

  pub fn send<T, D, E, S, SE>(
    &mut self,
    sender: &mut Sender<T, D>,
    storage: &mut S,
    command: Command,
  ) -> Result<(), Error<E, SE>>
  where
    T: OutputPin<Error = E>,
    D: DelayNs,
    S: RollingCodeStorage<Error = SE>,
  {
    self.send_repeat(sender, storage, command, 0)
  }

  pub fn send_repeat<S, TE, CS, SE>(
    &mut self,
    sender: &mut S,
    storage: &mut CS,
    command: Command,
    repetitions: usize,
  ) -> Result<(), Error<TE, SE>>
  where
    S: SendFrame<Error = TE>,
    CS: RollingCodeStorage<Error = SE>,
  {
    let frame = Frame::builder()
      .key(0xA7)
      .command(command)
      .remote_address(self.address)
      .rolling_code(self.rolling_code)
      .build()
      .unwrap();

    if let Err(err) = sender.send_frame_repeat(&frame, repetitions) {
      return Err(Error::TransmitError(err))
    }

    self.rolling_code += 1;
    if let Err(err) = storage.persist(&*self) {
      return Err(Error::StorageError(err))
    }

    Ok(())
  }
}
