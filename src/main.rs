use std::error::Error;
use std::process::exit;

use clap::{Arg, App, value_t};

use rppal::{gpio::Gpio, hal::Delay};

use somfy::*;

mod storage;
use storage::Storage;

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
    .arg(Arg::with_name("repetitions")
      .short("r")
      .long("repeat")
      .value_name("REPETITIONS")
      .help("Number of command repetitions")
      .takes_value(true)
      .requires("command")
    )
    .get_matches();

  let remote_name = matches.value_of("remote").unwrap();
  let command = if let Some(command) = matches.value_of("command") {
     Some(command.parse()?)
  } else {
    None
  };
  let repetitions = value_t!(matches.value_of("repetitions"), usize).unwrap_or(0);

  let mut storage = Storage::default();
  storage.load()?;

  let gpio = Gpio::new()?;

  let mut transmitter = gpio.get(TRANSMITTER_PIN)?.into_output();
  transmitter.set_low();

  let mut sender = Sender {
    transmitter,
    delay: Delay,
  };

  if let Some(command) = command {
    let remote_result = storage.with_remote(&remote_name, |remote| {
      log::info!("Sending command “{:?}” with remote “{}”.", command, remote_name);
      remote.send_repeat(&mut sender, command, repetitions)
    });

    if let Some(remote_result) = remote_result {
      remote_result?;
    } else {
      eprintln!("No remote with name “{}” found.", remote_name);
      exit(1);
    }
  }

  Ok(())
}
