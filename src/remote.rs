use core::fmt;

use ux::u24;

use embedded_hal::digital::OutputPin;
use embedded_hal::blocking::delay::DelayUs;

use super::*;

pub struct Remote<T, D> {
  address: u24,
  rolling_code: u16,
  sender: Sender<T, D>,
}

impl<T, D> fmt::Debug for Remote<T, D>
where
  T: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Remote")
      .field("address", &self.address)
      .field("rolling_code", &self.rolling_code)
      .field("address", &self.sender)
      .finish()
  }
}

impl<T, D, E> Remote<T, D>
where
  T: OutputPin<Error = E>,
  D: DelayUs<u32, Error = E>,
{
  pub fn new(address: u24, rolling_code: u16, sender: Sender<T, D>) -> Self {
    Self {
      address,
      rolling_code,
      sender,
    }
  }

  pub fn send(&mut self, command: Command) -> Result<(), E> {
    self.send_repeat(command, 0)
  }

  pub fn send_repeat(&mut self, command: Command, repetitions: usize) -> Result<(), E> {
    let frame = Frame::builder()
      .key(0xA7)
      .command(command)
      .remote_address(self.address)
      .rolling_code(self.rolling_code)
      .build()
      .unwrap();

    self.rolling_code += 1;

    self.sender.send_frame_repeat(&frame, repetitions)
  }
  pub fn address(&self) -> u24 {
    self.address
  }

  pub fn rolling_code(&self) -> u16 {
    self.rolling_code
  }
}
