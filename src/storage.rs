use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::fs::File;
use std::str;
use std::path::{Path, PathBuf};

use ux::u24;

use somfy::Remote;

const CONFIG_FILE_PATH: &'static str = "./config.yaml";

#[derive(Debug)]
pub struct Storage {
  path: PathBuf,
  remotes: HashMap<String, Remote>,
}

impl Default for Storage {
  fn default() -> Self {
    Self::new(CONFIG_FILE_PATH)
  }
}

impl Storage {
  pub fn new(path: impl AsRef<Path>) -> Self {
    Self { path: PathBuf::from(path.as_ref()), remotes: HashMap::new() }
  }

  pub fn with_remote<C, T, E>(&mut self, name: &str, closure: C) -> Option<io::Result<T>>
  where
    E: Into<Box<dyn Error + Send + Sync>>,
    C: FnOnce(&mut Remote) -> Result<T, E>,
  {
    if let Some(remote) = self.remotes.get_mut(name) {
      let previous_rolling_code = remote.rolling_code();

      let value = closure(remote);

      if previous_rolling_code != remote.rolling_code() {
        if let Err(err) = self.persist() {
          return Some(Err(err))
        }
      }

      return Some(value.map_err(|err| io::Error::new(io::ErrorKind::Other, err)))
    }

    None
  }

  #[allow(unused)]
  pub fn add_remote(&mut self, name: String, address: u24, rolling_code: u16) {
    self.remotes.insert(name, Remote::new(address, rolling_code));
  }

  #[allow(unused)]
  pub fn remove_remote(&mut self, name: String) {
    self.remotes.remove(&name);
  }

  pub fn persist(&self) -> io::Result<()> {
    let mut file = File::create(&self.path)?;

    if let Err(err) = serde_yaml::to_writer(&mut file, &self.remotes) {
      return Err(io::Error::new(io::ErrorKind::Other, err))
    }

    Ok(())
  }

  pub fn load(&mut self) -> io::Result<()> {
    let mut file = File::open(&self.path)?;

    match serde_yaml::from_reader(&mut file) {
      Ok(ok) => {
        self.remotes = ok;
        Ok(())
      },
      Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
    }
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
