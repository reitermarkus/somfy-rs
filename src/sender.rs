use core::fmt;

use embedded_hal::{
  delay::DelayNs,
  digital::{
    OutputPin,
    PinState::{self, *},
  },
};

use crate::{Frame, SendFrame};

const SYMBOL_WIDTH: u32 = 1280;

#[derive(Debug, Clone, Copy)]
enum SyncType {
  Once,
  Repeat,
}

pub struct Sender<T, D> {
  pub transmitter: T,
  pub delay: D,
}

impl<T, D> fmt::Debug for Sender<T, D>
where
  T: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Sender").field("transmitter", &self.transmitter).finish()
  }
}

impl<T, D, E> SendFrame for Sender<T, D>
where
  T: OutputPin<Error = E>,
  D: DelayNs,
{
  type Error = E;

  /// Send a `Frame` once.
  fn send_frame(&mut self, frame: &Frame) -> Result<(), Self::Error> {
    self.send_frame_repeat(frame, 0)
  }

  /// Send a `Frame` with a given number of `repetitions`. The total number sent is
  /// `1 + repetitions`, i.e. `send_frame(…)` is the same as `send_frame_repeat(…, 0)`.
  fn send_frame_repeat(&mut self, frame: &Frame, repetitions: usize) -> Result<(), Self::Error> {
    self.wake_up()?;

    self.send_frame_with_type(frame, SyncType::Once)?;

    for _ in 0..repetitions {
      self.send_frame_with_type(frame, SyncType::Repeat)?;
    }

    Ok(())
  }
}

impl<T, D, E> Sender<T, D>
where
  T: OutputPin<Error = E>,
  D: DelayNs,
{
  fn send_frame_with_type(&mut self, frame: &Frame, sync_type: SyncType) -> Result<(), E> {
    self.hardware_sync(sync_type)?;
    self.software_sync()?;

    for &byte in frame.as_bytes() {
      self.send_byte(byte)?;
    }

    self.inter_frame_gap()
  }

  fn wake_up(&mut self) -> Result<(), E> {
    self.send_state(High, 9_415)?;
    self.send_state(Low, 89_565)
  }

  fn hardware_sync(&mut self, sync_type: SyncType) -> Result<(), E> {
    let sync_count = match sync_type {
      SyncType::Once => 2,
      SyncType::Repeat => 7,
    };

    for _ in 0..sync_count {
      self.send_state(High, 2 * SYMBOL_WIDTH)?;
      self.send_state(Low, 2 * SYMBOL_WIDTH)?;
    }

    Ok(())
  }

  fn software_sync(&mut self) -> Result<(), E> {
    self.send_state(High, 4_550)?;
    self.send_state(Low, SYMBOL_WIDTH / 2)
  }

  fn inter_frame_gap(&mut self) -> Result<(), E> {
    self.send_state(Low, 30_415)
  }

  fn send_state(&mut self, state: PinState, time: u32) -> Result<(), E> {
    self.transmitter.set_state(state)?;
    self.delay.delay_us(time);
    Ok(())
  }

  // Send a byte, starting with the most significant bit.
  fn send_byte(&mut self, byte: u8) -> Result<(), E> {
    for bit in 0..=7 {
      self.send_bit((byte & (1 << (7 - bit))) != 0)?;
    }

    Ok(())
  }

  // Send a single bit, using Manchester encoding.
  fn send_bit(&mut self, bit: bool) -> Result<(), E> {
    let (from, to) = if bit { (Low, High) } else { (High, Low) };

    self.send_state(from, SYMBOL_WIDTH / 2)?;
    self.send_state(to, SYMBOL_WIDTH / 2)
  }
}
