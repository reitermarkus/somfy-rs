use std::error::Error;

use ux::u24;

use somfy::*;

use rppal::{gpio::Gpio, hal::Delay};

const TRANSMITTER_PIN: u8 = 4;

fn main() -> Result<(), Box<dyn Error>> {
  let rolling_code = 42;
  let remote_address = u24::new(0xFFAA11);

  let frame = Frame::builder()
    .key(0xA7)
    .command(Command::Up)
    .rolling_code(rolling_code)
    .remote_address(remote_address)
    .build();

  dbg!(frame);

  let gpio = Gpio::new()?;

  let mut transmitter = gpio.get(TRANSMITTER_PIN)?.into_output();
  transmitter.set_low();

  let mut remote = Remote {
    transmitter,
    delay: Delay,
  };

  dbg!(&remote);

  remote.send_frame(&frame.unwrap())?;

  Ok(())
}
