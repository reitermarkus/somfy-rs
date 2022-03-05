use std::collections::BTreeMap;
use std::io;
use std::fs::File;
use std::str;
use std::path::{Path, PathBuf};

use ux::u24;

use somfy::{Remote, RollingCodeStorage};

#[derive(Debug)]
pub struct Storage {
  path: PathBuf,
  address_map: BTreeMap<u24, String>,
  remotes: BTreeMap<String, Remote>,
}

impl Storage {
  pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
    let mut file = File::open(&path)?;

    match serde_yaml::from_reader::<_, BTreeMap<String, Remote>>(&mut file) {
      Ok(remotes) => {
        let address_map = remotes.iter().map(|(k, v)| {
          (v.address(), k.to_owned())
        }).collect();

        Ok(Self {
          path: path.as_ref().into(),
          address_map,
          remotes,
        })
      },
      Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
    }
  }

  pub fn remote(&self, name: &str) -> Option<&Remote> {
    self.remotes.get(name)
  }

  #[allow(unused)]
  pub fn remotes(&self) -> &BTreeMap<String, Remote> {
    &self.remotes
  }
}

impl RollingCodeStorage for Storage {
  type Error = io::Error;

  fn persist(&mut self, remote: &Remote) -> Result<(), Self::Error> {
    log::info!("Persisting config for remote {}.", remote.address());

    if let Some(remote_name) = self.address_map.get(&remote.address()) {
      if let Some(old_remote) = self.remotes.get_mut(remote_name) {
        *old_remote = remote.clone();

        let mut file = File::create(&self.path)?;

        if let Err(err) = serde_yaml::to_writer(&mut file, &self.remotes) {
          return Err(io::Error::new(io::ErrorKind::Other, err))
        }

        return Ok(())
      }
    }

    Err(io::Error::new(
      io::ErrorKind::NotFound,
      format!("No entry found for remote {}.", remote.address())
    ))
  }
}

#[test]
fn test_storage() {
  let dir = tempfile::tempdir().unwrap();

  let mut s = Storage::new(dir.path().join("config.yaml"));

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
