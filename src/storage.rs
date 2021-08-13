use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::str;

use serde::{Serialize, Serializer};

use ux::u24;

const CONFIG_FILE_PATH: &'static str = "./config.yaml";

macro_rules! format_address {
  ($address:ident) => {
    format!("{:#X}", $address)
  };
}

#[derive(Debug)]
pub struct Storage {
  path: String,
  remotes: HashMap<String, u16>,
}

impl Default for Storage {
  fn default() -> Self {
    Self { path: CONFIG_FILE_PATH.to_string(), remotes: HashMap::new() }
  }
}

impl Serialize for Storage {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_map(self.remotes.iter())
  }
}

impl Storage {
  pub fn next_rolling_code(&mut self, address: u24) -> Option<u16> {
    let rolling_code = self.remotes.get_mut(&format_address!(address))?;
    *rolling_code = *rolling_code + 1;
    Some(*rolling_code)
  }

  pub fn add_remote(&mut self, address: u24, rolling_code: u16) {
    self.remotes.insert(format_address!(address), rolling_code);
  }

  pub fn remove_remote(&mut self, address: u24) {
    self.remotes.remove(&format_address!(address));
  }

  pub fn serialize(&self) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .truncate(true)
      .open(&self.path)?;

    let yaml_string = serde_yaml::to_string(&self).unwrap();
    file.write(yaml_string.as_bytes())?;

    Ok(())
  }

  pub fn deserialize(&mut self) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
      .read(true)
      .create(true)
      .open(&self.path)?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    self.remotes = serde_yaml::from_str::<HashMap<String, u16>>(&buf).unwrap();

    Ok(())
  }
}

#[test]
fn test_storage() {
  let mut s = Storage::default();

  s.add_remote(u24::new(0xAA), 0xA7);
  s.add_remote(u24::new(0xAF), 0xA7);

  let yaml_string = serde_yaml::to_string(&s).unwrap();
  println!("Config file:\n{:?}", yaml_string);

  s.remove_remote(u24::new(0xAA));
  s.remove_remote(u24::new(0xAF));

  assert_eq!(s.remotes.len(), 0);

  s.remotes = serde_yaml::from_str::<HashMap<String, u16>>(&yaml_string).unwrap();

  println!("{:?}", s);
  assert_eq!(s.remotes.len(), 2);
}
