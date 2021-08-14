use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::fs::File;
use std::rc::Rc;
use std::str;
use std::path::PathBuf;

use serde::{Serialize, Serializer, Deserialize};

use ux::u24;

use super::Remote;

const CONFIG_FILE_PATH: &'static str = "./config.yaml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RemoteInfo {
  address: u24,
  rolling_code: u16,
}

#[derive(Debug)]
pub struct Storage {
  path: PathBuf,
  remotes: Rc<RefCell<HashMap<String, RemoteInfo>>>,
}

impl Default for Storage {
  fn default() -> Self {
    Self { path: PathBuf::from(CONFIG_FILE_PATH), remotes: Rc::new(RefCell::new(HashMap::new())) }
  }
}

impl Serialize for Storage {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_map(self.remotes.borrow().iter())
  }
}

impl Storage {
  pub fn remote(self: Rc<Self>, name: &str) -> Option<Remote<impl FnMut(u24, u16)>> {
    let remotes = self.remotes.borrow();
    let remote = remotes.get(name)?;
    let address = remote.address;
    let rolling_code = remote.rolling_code;

    let this = Rc::clone(&self);
    Some(Remote::new(address, rolling_code, move |address, rolling_code| {
      log::info!("New rolling code: {:?}", rolling_code);

      if let Ok(mut remotes) = this.remotes.try_borrow_mut() {
        for (_, ref mut remote_info) in remotes.iter_mut() {
          if remote_info.address == address {
            remote_info.rolling_code = rolling_code;
            break;
          }
        }
      }

      if let Err(err) = this.persist() {
        log::error!("{}", err);
      }
    }))
  }

  pub fn address(&self, name: &str) -> Option<u24> {
    let remotes = self.remotes.borrow();
    let remote_info = remotes.get(name)?;
    Some(remote_info.address)
  }

  pub fn add_remote(&mut self, name: String, address: u24, rolling_code: u16) {
    self.remotes.borrow_mut().insert(name, RemoteInfo {
      address: address.into(),
      rolling_code
    });
  }

  pub fn remove_remote(&mut self, name: String) {
    self.remotes.borrow_mut().remove(&name);
  }

  pub fn persist(&self) -> io::Result<()> {
    let mut file = File::create(&self.path)?;

    if let Err(err) = serde_yaml::to_writer(&mut file, &self) {
      return Err(io::Error::new(io::ErrorKind::Other, err))
    }

    Ok(())
  }

  pub fn load(&mut self) -> io::Result<()> {
    let mut file = File::open(&self.path)?;

    match serde_yaml::from_reader(&mut file) {
      Ok(ok) => {
        self.remotes = Rc::new(RefCell::new(ok));
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

  s.remotes = serde_yaml::from_str::<HashMap<String, RemoteInfo>>(&yaml_string).unwrap();

  println!("{:?}", s);
  assert_eq!(s.remotes.len(), 2);
  assert_eq!(s.address(&String::from("Remote A")), Some(u24::new(0xAA)));
}
