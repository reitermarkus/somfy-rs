use std::collections::{BTreeMap, HashMap};
use std::io;
use std::fs::File;
use std::str;
use std::path::{Path, PathBuf};

use ux::u24;

use somfy::{Remote, RollingCodeStorage};

const CONFIG_FILE_PATH: &'static str = "./config.yaml";

#[derive(Debug)]
pub struct Storage {
  path: PathBuf,
  address_map: BTreeMap<u24, String>,
  remotes: HashMap<String, Remote>,
}

impl Default for Storage {
  fn default() -> Self {
    Self::new(CONFIG_FILE_PATH)
  }
}

impl Storage {
  pub fn new(path: impl AsRef<Path>) -> Self {
    Self { path: PathBuf::from(path.as_ref()), address_map: Default::default(), remotes: HashMap::new() }
  }

  pub fn remote(&self, name: &str) -> Option<&Remote> {
    self.remotes.get(name)
  }

  pub fn remotes(&self) -> &HashMap<String, Remote> {
    &self.remotes
  }

  #[allow(unused)]
  pub fn add_remote(&mut self, name: String, address: u24, rolling_code: u16) {
    self.remotes.insert(name, Remote::new(address, rolling_code));
  }

  #[allow(unused)]
  pub fn remove_remote(&mut self, name: String) {
    self.remotes.remove(&name);
  }

  pub fn load(&mut self) -> io::Result<()> {
    let mut file = File::open(&self.path)?;

    match serde_yaml::from_reader(&mut file) {
      Ok(ok) => {
        self.remotes = ok;

        self.address_map = self.remotes.iter().map(|(k, v)| {
          (v.address(), k.to_owned())
        }).collect();

        Ok(())
      },
      Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
    }
  }
}

impl RollingCodeStorage for Storage {
  type Error = io::Error;

  fn persist(&mut self, remote: &Remote) -> Result<(), Self::Error> {
    log::info!("Persisting config for remote 0x{:2X}.", remote.address());

    if let Some(remote_name) = self.address_map.get(&remote.address()) {
      if let Some(old_remote) = self.remotes.get_mut(remote_name) {
        *old_remote = remote.clone();

        let mut file = File::create(&self.path)?;

        if let Err(err) = serde_yaml::to_writer(&mut file, &self.remotes) {
          return Err(io::Error::new(io::ErrorKind::Other, err))
        }
      }
    }

    Ok(())
  }
}

#[test]
fn test_storage() {
  let mut s = Storage::default();

  s.add_remote(String::from("Remote A"), u24::new(0xAA), 0xA7);
  s.add_remote(String::from("Remote B"), u24::new(0xAF), 0xA7);

  let yaml_string = serde_yaml::to_string(&s).unwrap();
  println!("Config file:\n{:?}", yaml_string);

  s.remove_remote(String::from("Remote A"));
  s.remove_remote(String::from("Remote B"));

  assert_eq!(s.remotes.len(), 0);

  s.remotes = serde_yaml::from_str::<HashMap<String, Remote>>(&yaml_string).unwrap();

  println!("{:?}", s);
  assert_eq!(s.remotes.len(), 2);
  assert_eq!(s.address(&String::from("Remote A")), Some(u24::new(0xAA)));
}
