use core::cell::RefCell;

use embedded_hal::{
  delay::DelayNs,
  digital::{ErrorType, OutputPin, PinState},
};

use crate::{Frame, SendFrame, Sender};

#[derive(Debug, Clone)]
struct Pulse {
  state: PinState,
  duration: u32,
}

struct SpiPulseAccumulator {
  current_state: Option<PinState>,
  current_duration: Option<u32>,
  pulses: Vec<Pulse>,
}

impl SpiPulseAccumulator {
  pub const fn new() -> Self {
    Self { current_state: None, current_duration: None, pulses: Vec::new() }
  }

  pub fn finish(mut self) -> Vec<Pulse> {
    if let Some(state) = self.current_state.take() {
      let duration = self.current_duration.take().unwrap_or(0);
      self.pulses.push(Pulse { state, duration });
    }

    self.pulses
  }
}

impl ErrorType for SpiPulseAccumulator {
  type Error = core::convert::Infallible;
}

impl OutputPin for SpiPulseAccumulator {
  fn set_low(&mut self) -> Result<(), Self::Error> {
    self.set_state(PinState::Low)
  }

  fn set_high(&mut self) -> Result<(), Self::Error> {
    self.set_state(PinState::High)
  }

  fn set_state(&mut self, new_state: PinState) -> Result<(), Self::Error> {
    match self.current_state.replace(new_state) {
      None => (),
      Some(old_state) if old_state == new_state => (),
      Some(state) => {
        if let Some(duration) = self.current_duration.take() {
          self.pulses.push(Pulse { state, duration });
        }
      },
    }

    Ok(())
  }
}

impl DelayNs for SpiPulseAccumulator {
  fn delay_ns(&mut self, ns: u32) {
    if let Some(ref mut current_duration) = self.current_duration {
      *current_duration += ns;
    } else {
      self.current_duration = Some(ns);
    }
  }
}

struct OutputPinDelayProxy<'a> {
  spi_sender: &'a RefCell<SpiPulseAccumulator>,
}

impl<'a> OutputPinDelayProxy<'a> {
  pub const fn new(spi_sender: &'a RefCell<SpiPulseAccumulator>) -> Self {
    Self { spi_sender }
  }
}

impl ErrorType for OutputPinDelayProxy<'_> {
  type Error = core::convert::Infallible;
}

impl OutputPin for OutputPinDelayProxy<'_> {
  fn set_low(&mut self) -> Result<(), Self::Error> {
    self.spi_sender.borrow_mut().set_low()
  }

  fn set_high(&mut self) -> Result<(), Self::Error> {
    self.spi_sender.borrow_mut().set_high()
  }
}

impl DelayNs for OutputPinDelayProxy<'_> {
  fn delay_ns(&mut self, ns: u32) {
    self.spi_sender.borrow_mut().delay_ns(ns)
  }
}

#[derive(Debug)]
pub struct SpiSender {}

impl SpiSender {
  pub const fn new() -> Self {
    Self {}
  }
}

impl SendFrame for SpiSender {
  type Error = core::convert::Infallible;

  /// Send a `Frame` once.
  fn send_frame(&mut self, frame: &Frame) -> Result<(), Self::Error> {
    self.send_frame_repeat(frame, 0)
  }

  /// Send a `Frame` with a given number of `repetitions`. The total number sent is
  /// `1 + repetitions`, i.e. `send_frame(…)` is the same as `send_frame_repeat(…, 0)`.
  fn send_frame_repeat(&mut self, frame: &Frame, repetitions: usize) -> Result<(), Self::Error> {
    let spi_pulse_accumulator = RefCell::new(SpiPulseAccumulator::new());

    let mut transmitter = OutputPinDelayProxy::new(&spi_pulse_accumulator);
    let mut delay = OutputPinDelayProxy::new(&spi_pulse_accumulator);

    let mut sender = Sender { transmitter: &mut transmitter, delay: &mut delay };
    sender.send_frame_repeat(frame, repetitions)?;

    let pulses = spi_pulse_accumulator.into_inner().finish();

    for pulse in pulses {
      println!("{:<4} {:>10}", format!("{:?}", pulse.state), pulse.duration);
    }

    Ok(())
  }
}
