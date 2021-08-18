use embedded_hal::digital::OutputPin;
use embedded_hal::blocking::delay::DelayUs;
use serde::{Serialize, Deserialize};
use ux::u24;

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Remote {
  address: u24,
  rolling_code: u16,
}

impl Remote {
  pub fn new(address: u24, rolling_code: u16) -> Self {
    Self {
      address,
      rolling_code,
    }
  }

  pub fn address(&self) -> u24 {
    self.address
  }

  pub fn rolling_code(&self) -> u16 {
    self.rolling_code
  }

  pub fn send<T, D, E>(&mut self, sender: &mut Sender<T, D>, command: Command) -> Result<(), E>
  where
    T: OutputPin<Error = E>,
    D: DelayUs<u32, Error = E>,
  {
    self.send_repeat(sender, command, 0)
  }

  pub fn send_repeat<T, D, E>(&mut self, sender: &mut Sender<T, D>, command: Command, repetitions: usize) -> Result<(), E>
  where
    T: OutputPin<Error = E>,
    D: DelayUs<u32, Error = E>,
  {
    let frame = Frame::builder()
      .key(0xA7)
      .command(command)
      .remote_address(self.address)
      .rolling_code(self.rolling_code)
      .build()
      .unwrap();

    self.rolling_code += 1;

    sender.send_frame_repeat(&frame, repetitions)
  }
}
