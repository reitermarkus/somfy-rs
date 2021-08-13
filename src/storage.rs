use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::str;

use serde::{Serialize, Serializer, Deserialize};

use ux::u24;

const CONFIG_FILE_PATH: &'static str = "./config.yaml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RemoteInfo {
  address: u32,
  rolling_code: u16,
}

#[derive(Debug)]
pub struct Storage {
  path: String,
  remotes: HashMap<String, RemoteInfo>,
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
  pub fn next_rolling_code(&mut self, name: &String) -> Option<u16> {
    let remote_info = self.remotes.get_mut(name)?;
    let rolling_code = remote_info.next_rolling_code();
    self.persist().ok()?;
    Some(rolling_code)
  }

  pub fn address(&self, name: &String) -> Option<u24> {
    let remote_info = self.remotes.get(name)?;
    Some(u24::new(remote_info.address))
  }

  pub fn add_remote(&mut self, name: String, address: u24, rolling_code: u16) {
    self.remotes.insert(name, RemoteInfo {
      address: address.into(),
      rolling_code
    });
  }

  pub fn remove_remote(&mut self, name: String) {
    self.remotes.remove(&name);
  }

  pub fn persist(&self) -> std::io::Result<()> {
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

  pub fn load(&mut self) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
      .read(true)
      .open(&self.path)?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    self.remotes = serde_yaml::from_str::<HashMap<String, RemoteInfo>>(&buf).unwrap();

    Ok(())
  }
}

impl RemoteInfo {
  fn next_rolling_code(&mut self) -> u16 {
    self.rolling_code = self.rolling_code + 1;
    self.rolling_code
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

  s.remotes = serde_yaml::from_str::<HashMap<String, RemoteInfo>>(&yaml_string).unwrap();

  println!("{:?}", s);
  assert_eq!(s.remotes.len(), 2);
  assert_eq!(s.address(&String::from("Remote A")), Some(u24::new(0xAA)));

  let remote_a = s.remotes.get_mut("Remote A").unwrap();
  assert_eq!(remote_a.next_rolling_code(), 0xA7 + 1);
  assert_eq!(remote_a.next_rolling_code(), 0xA7 + 2);
}
