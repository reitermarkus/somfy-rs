use core::fmt;

use ux::u24;

use embedded_hal::digital::OutputPin;
use embedded_hal::blocking::delay::DelayUs;

use super::*;

pub struct Remote<C> {
  address: u24,
  rolling_code: u16,
  rolling_code_callback: C,
}

impl<C> fmt::Debug for Remote<C> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Remote")
      .field("address", &self.address)
      .field("rolling_code", &self.rolling_code)
      .finish()
  }
}

impl<C> Remote<C> {
  pub fn new(address: u24, rolling_code: u16, rolling_code_callback: C) -> Self {
    Self {
      address,
      rolling_code,
      rolling_code_callback,
    }
  }
}

impl<C> Remote<C> {
  pub fn address(&self) -> u24 {
    self.address
  }

  pub fn rolling_code(&self) -> u16 {
    self.rolling_code
  }
}

impl<C> Remote<C>
where
  C: FnMut(u24, u16),
{
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
    (self.rolling_code_callback)(self.address, self.rolling_code);

    sender.send_frame_repeat(&frame, repetitions)
  }
}
