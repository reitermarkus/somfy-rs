use ux::u24;

use crate::Command;

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Frame {
  key: u8,
  command_and_checksum: u8,
  rolling_code: [u8; 2],
  remote_address: [u8; 3],
}

impl Frame {
  pub fn builder() -> FrameBuilder {
    FrameBuilder::new()
  }

  pub fn rolling_code(&self) -> u16 {
    u16::from_be_bytes(self.rolling_code)
  }

  pub(crate) fn as_bytes(&self) -> &[u8] {
    // SAFETY: This is safe because a `Frame` is always
    // exactly 7 bytes containing valid data.
    unsafe { core::mem::transmute::<&Frame, &[u8; 7]>(self) }
  }

  // Obfuscate the message by XOR'ing all bytes.
  fn obfuscate(&mut self) {
    self.command_and_checksum ^= self.key;
    self.rolling_code[0] ^= self.command_and_checksum;
    self.rolling_code[1] ^= self.rolling_code[0];
    self.remote_address[0] ^= self.rolling_code[1];
    self.remote_address[1] ^= self.remote_address[0];
    self.remote_address[2] ^= self.remote_address[1];
  }

  // Deobfuscate the message by XOR'ing all bytes in reverse order.
  #[cfg(test)]
  fn deobfuscate(&mut self) {
    self.remote_address[2] ^= self.remote_address[1];
    self.remote_address[1] ^= self.remote_address[0];
    self.remote_address[0] ^= self.rolling_code[1];
    self.rolling_code[1] ^= self.rolling_code[0];
    self.rolling_code[0] ^= self.command_and_checksum;
    self.command_and_checksum ^= self.key;
  }
}

#[derive(Default, Debug, Clone)]
pub struct FrameBuilder {
  key: Option<u8>,
  command: Option<Command>,
  rolling_code: Option<u16>,
  remote_address: Option<u24>,
}

impl FrameBuilder {
  fn new() -> Self {
    Default::default()
  }

  pub fn key(&mut self, key: u8) -> &mut Self {
    self.key = Some(key);
    self
  }

  pub fn command(&mut self, command: Command) -> &mut Self {
    self.command = Some(command);
    self
  }

  pub fn rolling_code(&mut self, rolling_code: u16) -> &mut Self {
    self.rolling_code = Some(rolling_code);
    self
  }

  pub fn remote_address(&mut self, remote_address: u24) -> &mut Self {
    self.remote_address = Some(remote_address);
    self
  }

  pub fn build(&self) -> Option<Frame> {
    let key = self.key?;
    let command = self.command? as u8;
    let rolling_code = self.rolling_code?.to_be_bytes();
    let remote_address = u32::from(self.remote_address?).to_le_bytes();
    let remote_address = [remote_address[0], remote_address[1], remote_address[2]];

    // Calculate the checksum by XOR'ing all nibbles.
    let checksum = (key >> 4
      ^ key
      ^ command >> 4
      ^ rolling_code[0] >> 4
      ^ rolling_code[0]
      ^ rolling_code[1] >> 4
      ^ rolling_code[1]
      ^ remote_address[0] >> 4
      ^ remote_address[0]
      ^ remote_address[1] >> 4
      ^ remote_address[1]
      ^ remote_address[2] >> 4
      ^ remote_address[2])
      & 0b1111;

    let mut frame = Frame { key, command_and_checksum: command | checksum, rolling_code, remote_address };
    frame.obfuscate();

    Some(frame)
  }
}

pub trait SendFrame {
  type Error;

  fn send_frame(&mut self, frame: &Frame) -> Result<(), Self::Error> {
    self.send_frame_repeat(frame, 0)
  }

  fn send_frame_repeat(&mut self, frame: &Frame, repetitions: usize) -> Result<(), Self::Error>;
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
