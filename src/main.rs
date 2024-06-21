use std::{error::Error, path::PathBuf, process::exit};

use clap::{value_parser, Arg, ArgAction, Command};

use rppal::{gpio::Gpio, hal::Delay};

use somfy::*;

mod storage;
use storage::Storage;

#[cfg(feature = "server")]
mod thing;

#[cfg(feature = "server")]
use webthing::{Thing, ThingsType, WebThingServer};

const TRANSMITTER_PIN: u8 = 4;

const DEFAULT_CONFIG_FILE_PATH: &str = "./config.yaml";

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let matches = Command::new("somfy")
    .arg(
      Arg::new("config")
        .short('f')
        .long("config")
        .value_name("FILE")
        .help("Path to the config file")
        .num_args(1)
        .default_value(DEFAULT_CONFIG_FILE_PATH)
        .action(ArgAction::Set)
        .value_parser(value_parser!(PathBuf)),
    )
    .subcommands(["my", "up", "myup", "down", "mydown", "updown", "myupdown", "prog", "sunflag", "flag"].map(
      |command| {
        Command::new(command)
          .about(format!("Send the {command} command"))
          .arg(Arg::new("remote").help("The remote name").requires("command").num_args(1).action(ArgAction::Set))
          .arg(
            Arg::new("repetitions")
              .short('r')
              .long("repeat")
              .value_name("REPETITIONS")
              .help("Number of command repetitions")
              .num_args(1)
              .requires("command")
              .value_parser(value_parser!(usize))
              .default_value("0")
              .action(ArgAction::Set),
          )
      },
    ))
    .subcommand(Command::new("server").long_flag("server").short_flag('s').about("Start API server"))
    .get_matches();

  let gpio = Gpio::new()?;

  let mut transmitter = gpio.get(TRANSMITTER_PIN)?.into_output();
  transmitter.set_low();

  let mut sender = Sender { transmitter, delay: Delay };

  let storage_path: &PathBuf = matches.get_one("config").unwrap();
  let mut storage = Storage::new(storage_path)?;

  match matches.subcommand_name() {
    #[cfg(feature = "server")]
    Some("server") => {
      use std::{
        collections::HashMap,
        sync::{Arc, Mutex, RwLock},
      };

      let mut remotes = HashMap::new();

      let mut things = Vec::<Arc<RwLock<Box<dyn Thing + 'static>>>>::new();

      for (name, remote) in storage.remotes() {
        let thing = thing::make_remote(name, remote);
        remotes.insert(thing.get_id().clone(), Arc::new(RwLock::new(remote.clone())));
        things.push(Arc::new(RwLock::new(Box::new(thing))));
      }

      let generator =
        thing::Generator { sender: Arc::new(Mutex::new(sender)), storage: Arc::new(RwLock::new(storage)), remotes };

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
    },
    Some(subcommand_name) => {
      let matches = matches.subcommand_matches(subcommand_name).unwrap();

      let command = subcommand_name.parse::<somfy::Command>().unwrap();
      let remote_name: &String = matches.get_one("remote").unwrap();
      let repetitions: usize = matches.get_one("repetitions").copied().unwrap();

      if let Some(remote) = storage.remote(remote_name) {
        log::info!("Sending command “{:?}” with remote “{}”.", command, remote_name);
        remote.clone().send_repeat(&mut sender, &mut storage, command, repetitions)?;
      } else {
        eprintln!("No remote with name “{}” found.", remote_name);
        exit(1);
      }
    },
    _ => unreachable!(),
  }

  Ok(())
}
