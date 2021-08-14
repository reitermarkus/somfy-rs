use std::error::Error;
use std::rc::Rc;

use clap::{Arg, App};

use somfy::*;

use rppal::{gpio::Gpio, hal::Delay};

const TRANSMITTER_PIN: u8 = 4;

fn main() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let matches = App::new("somfy")
    .arg(Arg::with_name("remote")
      .help("The remote name")
      .requires("command")
    )
    .arg(Arg::with_name("command")
      .short("c")
      .long("command")
      .value_name("COMMAND")
      .help("The remote command to send")
      .takes_value(true)
      .requires("remote")
    )
    .get_matches();

  let remote_name = matches.value_of("remote").unwrap();
  let command = if let Some(command) = matches.value_of("command") {
     Some(command.parse()?)
  } else {
    None
  };

  dbg!(&remote_name);
  dbg!(&command);

  let mut storage = Rc::new(Storage::default());
  Rc::get_mut(&mut storage).unwrap().load()?;
  dbg!(&storage);

  let gpio = Gpio::new()?;

  let mut transmitter = gpio.get(TRANSMITTER_PIN)?.into_output();
  transmitter.set_low();

  let sender = Sender {
    transmitter,
    delay: Delay,
  };

  dbg!(&sender);

  if let Some(command) = command {
    let mut remote = storage.remote(remote_name, sender).unwrap();
    dbg!(&remote);

    log::info!("Sending command “{:?}” with remote “{}”.", command, remote_name);
    remote.send(command)?;
  }

  Ok(())
}
