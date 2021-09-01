use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};

use clap::{Arg, App, value_t};

use rppal::{gpio::Gpio, hal::Delay};

use somfy::*;

mod storage;
use storage::Storage;

#[cfg(feature = "server")]
mod thing;

#[cfg(feature = "server")]
use webthing::{Thing, ThingsType, WebThingServer};

const TRANSMITTER_PIN: u8 = 4;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let matches = App::new("somfy")
    .arg(Arg::with_name("remote")
      .help("The remote name")
      .requires("command")
    )
    .arg(Arg::with_name("config")
      .short("f")
      .long("config")
      .value_name("FILE")
      .help("The path to the config file")
      .takes_value(true)
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
    .arg(Arg::with_name("server")
      .short("s")
      .long("server")
      .help("Start API server")
      .conflicts_with_all(&["remote", "command"])
    )
    .get_matches();

  let gpio = Gpio::new()?;

  let mut transmitter = gpio.get(TRANSMITTER_PIN)?.into_output();
  transmitter.set_low();

  let mut sender = Sender {
    transmitter,
    delay: Delay,
  };

  let mut storage = value_t!(matches.value_of("config"), PathBuf)
    .map(|path| Storage::new(path))
    .unwrap_or_default();
  storage.load()?;

  let repetitions = value_t!(matches.value_of("repetitions"), usize).unwrap_or(0);

  #[cfg(feature = "server")]
  if matches.is_present("server") {
    let mut remotes = HashMap::new();

    let mut things = Vec::<Arc<RwLock<Box<dyn Thing + 'static>>>>::new();

    for (name, remote) in storage.remotes() {
      let thing = thing::make_remote(name, remote);
      remotes.insert(thing.get_id().clone(), Arc::new(RwLock::new(remote.clone())));
      things.push(Arc::new(RwLock::new(Box::new(thing))));
    }

    let generator = thing::Generator {
      sender: Arc::new(Mutex::new(sender)),
      storage: Arc::new(RwLock::new(storage)),
      remotes,
    };

    log::info!("Starting server.");
    let mut server = WebThingServer::new(
        ThingsType::Multiple(things, "Somfy RTS Blinds".to_owned()),
        Some(8888),
        None,
        None,
        Box::new(generator),
        None,
        Some(true),
    );
    server.start(None).await?;

    return Ok(())
  }

  let remote_name = matches.value_of("remote").unwrap();
  let command = if let Some(command) = matches.value_of("command") {
    Some(command.parse()?)
  } else {
    None
  };

  if let Some(command) = command {
    if let Some(remote) = storage.remote(&remote_name) {
      log::info!("Sending command “{:?}” with remote “{}”.", command, remote_name);
      remote.clone().send_repeat(&mut sender, &mut storage, command, repetitions)?;
    } else {
      eprintln!("No remote with name “{}” found.", remote_name);
      exit(1);
    }
  }

  Ok(())
}
