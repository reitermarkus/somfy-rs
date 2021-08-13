use std::str;
use std::collections::HashMap;
use std::io::Write;
use std::fs::OpenOptions;

use serde::{Serialize, Deserialize, Serializer};

use ux::u24;

const CONFIG_FILE_PATH: &'static str = "./config.yaml";

#[derive(Debug)]
pub struct Storage {
  path: String,
  remotes: HashMap<String, u16>,
}

impl Default for Storage {
  fn default() -> Self {
    Self {
      path: CONFIG_FILE_PATH.to_string(),
      remotes: HashMap::new(),
    }
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
  pub fn nextRollingCode(&mut self, address: &u24) -> Option<u16> {
    let rolling_code = self.remotes.get_mut(format!("{:#X}", address))?;
    *rolling_code = *rolling_code + 1;
    Some(*rolling_code)
  }

  pub fn serialize(&self) -> std::io::Result<()> {
    let file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .truncate(true)
      .open(&self.path)?;

    let yaml_string = serde_yaml::to_string(&self).unwrap();
    file.write(yaml_string.as_bytes())?;

    Ok(())
  }

}
