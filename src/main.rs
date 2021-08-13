use std::error::Error;

use ux::u24;

use somfy::*;

use rppal::{gpio::Gpio, hal::Delay};

const TRANSMITTER_PIN: u8 = 4;

fn main() -> Result<(), Box<dyn Error>> {
  let gpio = Gpio::new()?;

  let mut transmitter = gpio.get(TRANSMITTER_PIN)?.into_output();
  transmitter.set_low();

  let sender = Sender {
    transmitter,
    delay: Delay,
  };

  dbg!(&sender);

  let rolling_code = 42;
  let remote_address = u24::new(0xFFAA11);
  let mut remote = Remote::new(remote_address, rolling_code, sender);

  dbg!(&remote);

  remote.send(Command::Up)?;

  Ok(())
}
