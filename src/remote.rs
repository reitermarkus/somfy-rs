use embedded_hal::blocking::delay::{DelayUs, DelayMs};
use embedded_hal::digital::{OutputPin, PinState::{self, *}};

use super::*;

const SYMBOL_RATE: u16 = 640;

#[derive(Debug, Clone, Copy)]
enum SyncType {
  Once,
  Repeat,
}

pub struct Remote<T, D> {
  pub transmitter: T,
  pub delay: D,
}

impl<T, D, E> Remote<T, D>
where
  T: OutputPin<Error = E>,
  D: DelayUs<u16, Error = E> + DelayMs<u8, Error = E>,
{
  /// Send a `Frame` once.
  pub fn send_frame(&mut self, frame: &Frame) -> Result<(), T::Error> {
    self.send_frame_repeat(frame, 0)
  }

  /// Send a `Frame` with a given number of `repetitions`. The total number sent is
  /// `1 + repetitions`, i.e. `send_frame(…)` is the same as `send_frame_repeat(…, 0)`.
  pub fn send_frame_repeat(&mut self, frame: &Frame, repetitions: usize) -> Result<(), T::Error> {
    self.send_frame_with_type(frame, SyncType::Once)?;

    for _ in 0..repetitions {
      self.send_frame_with_type(frame, SyncType::Repeat)?;
    }

    Ok(())
  }

  fn send_frame_with_type(&mut self, frame: &Frame, sync_type: SyncType) -> Result<(), T::Error> {
    self.wake_up()?;
    self.hardware_sync(sync_type)?;
    self.software_sync()?;

    for &byte in frame.as_bytes() {
      self.send_byte(byte)?;
    }

    self.send_state(Low, 415)?;
    self.delay.try_delay_ms(30)?;

    Ok(())
  }

  fn wake_up(&mut self) -> Result<(), T::Error> {
    self.send_state(High, 9415)?;
    self.send_state(Low, 9415)?;
    self.delay.try_delay_ms(80)?;

    Ok(())
  }

  fn hardware_sync(&mut self, sync_type: SyncType) -> Result<(), T::Error> {
    let sync_count = match sync_type {
      SyncType::Once => 2,
      SyncType::Repeat => 7,
    };

    for _ in 0..sync_count {
      self.send_state(High, 4 * SYMBOL_RATE)?;
      self.send_state(Low, 4 * SYMBOL_RATE)?;
    }

    Ok(())
  }

  fn software_sync(&mut self) -> Result<(), T::Error> {
    self.send_state(High, 4550)?;
    self.send_state(Low, SYMBOL_RATE)?;
    Ok(())
  }

  fn send_state(&mut self, state: PinState, time: u16) -> Result<(), T::Error> {
    self.transmitter.try_set_state(state)?;
    self.delay.try_delay_us(time)?;
    Ok(())
  }

  // Send a byte, starting with the most significant bit.
  fn send_byte(&mut self, byte: u8) -> Result<(), T::Error> {
    for bit in 0..=7 {
      self.send_bit((byte & (1 << (7 - bit))) != 0)?;
    }

    Ok(())
  }

  // Send a single bit, using Manchester encoding.
  fn send_bit(&mut self, bit: bool) -> Result<(), T::Error> {
    let (from, to) = if bit {
      (Low, High)
    } else {
      (High, Low)
    };

    self.send_state(from, SYMBOL_RATE)?;
    self.send_state(to, SYMBOL_RATE)?;

    Ok(())
  }
}
