use core::fmt;

use ux::u24;

use embedded_hal::digital::OutputPin;
use embedded_hal::blocking::delay::DelayUs;

use super::*;

pub struct Remote<T, D, C> {
  address: u24,
  rolling_code: u16,
  sender: Sender<T, D>,
  rolling_code_callback: C,
}

impl<T, D, C> fmt::Debug for Remote<T, D, C>
where
  T: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Remote")
      .field("address", &self.address)
      .field("rolling_code", &self.rolling_code)
      .field("sender", &self.sender)
      .finish()
  }
}

impl<T, D, C, E> Remote<T, D, C>
where
  T: OutputPin<Error = E>,
  D: DelayUs<u32, Error = E>,
  C: FnMut(u16)
{
  pub fn new(address: u24, rolling_code: u16, sender: Sender<T, D>, rolling_code_callback: C) -> Self {
    Self {
      address,
      rolling_code,
      sender,
      rolling_code_callback,
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
    (self.rolling_code_callback)(self.rolling_code);

    self.sender.send_frame_repeat(&frame, repetitions)
  }
  pub fn address(&self) -> u24 {
    self.address
  }

  pub fn rolling_code(&self) -> u16 {
    self.rolling_code
  }
}
