#![cfg(feature = "server")]

use std::cmp::Ordering;
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::thread;

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::OutputPin;
use serde_json::json;
use webthing::{Action, BaseAction, BaseThing, BaseProperty, Thing};
use webthing::server::ActionGenerator;
use uuid::Uuid;

use crate::{Command, Remote, Sender};

struct Generator<T, D> {
  sender: Arc<Mutex<Sender<T, D>>>,
  remotes: HashMap<String, Arc<RwLock<Remote>>>,
}

impl<T, D, E> ActionGenerator for Generator<T, D>
where
  T: OutputPin<Error = E> + Send + 'static,
  D: DelayUs<u32, Error = E> + Send + 'static,
  E: Debug,
{
    fn generate(
        &self,
        thing: Weak<RwLock<Box<dyn Thing>>>,
        name: String,
        input: Option<&serde_json::Value>,
    ) -> Option<Box<dyn Action>> {
        let input = input.and_then(|v| v.as_object()).cloned();
        let thing_id = thing.upgrade()?.write().unwrap().get_id();
        let remote = self.remotes.get(&thing_id);

        match name.as_ref() {
          "move" => Some(Box::new(MoveAction::new(input, thing, self.sender.clone(), remote?.clone()))),
          _ => None,
        }
    }
}

pub struct MoveAction<T, D> {
  action: BaseAction,
  sender: Arc<Mutex<Sender<T, D>>>,
  remote: Arc<RwLock<Remote>>,
}

impl<T, D> MoveAction<T, D> {
    fn new(
        input: Option<serde_json::Map<String, serde_json::Value>>,
        thing: Weak<RwLock<Box<dyn Thing>>>,
        sender: Arc<Mutex<Sender<T, D>>>,
        remote: Arc<RwLock<Remote>>,
    ) -> Self {
        Self {
          action: BaseAction::new(
            Uuid::new_v4().to_string(),
            "move".to_owned(),
            input,
            thing,
          ),
          sender: sender.clone(),
          remote: remote.clone(),
        }
    }
}

impl<T, D, E> Action for MoveAction<T, D>
where
  T: OutputPin<Error = E> + Send + 'static,
  D: DelayUs<u32, Error = E> + Send + 'static,
  E: Debug,
{
    fn set_href_prefix(&mut self, prefix: String) {
      self.action.set_href_prefix(prefix)
    }

    fn get_id(&self) -> String {
      self.action.get_id()
    }

    fn get_name(&self) -> String {
      self.action.get_name()
    }

    fn get_href(&self) -> String {
      self.action.get_href()
    }

    fn get_status(&self) -> String {
      self.action.get_status()
    }

    fn get_time_requested(&self) -> String {
      self.action.get_time_requested()
    }

    fn get_time_completed(&self) -> Option<String> {
      self.action.get_time_completed()
    }

    fn get_input(&self) -> Option<serde_json::Map<String, serde_json::Value>> {
      self.action.get_input()
    }

    fn get_thing(&self) -> Option<Arc<RwLock<Box<dyn Thing>>>> {
      self.action.get_thing()
    }

    fn set_status(&mut self, status: String) {
      self.action.set_status(status)
    }

    fn start(&mut self) {
      self.action.start()
    }

    fn perform_action(&mut self) {
      let thing = self.get_thing();
      if thing.is_none() {
          return;
      }

      let thing = thing.unwrap();
      let input = self.get_input().unwrap().clone();
      let name = self.get_name();
      let id = self.get_id();

      let sender = self.sender.clone();
      let remote = self.remote.clone();

      thread::spawn(move || {
        let thing = thing.clone();
        let mut thing = thing.write().unwrap();

        let current_position = thing.find_property(&"position".to_owned()).unwrap().get_value().as_u64().unwrap();
        let target_position_value = input.get("position").unwrap();
        let target_position = target_position_value.as_u64().unwrap();

        let command = match target_position.cmp(&current_position) {
          Ordering::Less => Command::Down,
          Ordering::Equal => return,
          Ordering::Greater => Command::Up,
        };

        let mut sender = sender.lock().unwrap();
        let mut remote = remote.write().unwrap();
        remote.send_repeat(&mut *sender, command, 2).unwrap();

        thing.set_property(
          "position".to_owned(),
          target_position_value.clone(),
        ).unwrap();

        thing.finish_action(name, id);
      });
    }

    fn cancel(&mut self) {
        self.action.cancel()
    }

    fn finish(&mut self) {
        self.action.finish()
    }
}

fn make_remote(name: &str, remote: &Remote) -> Arc<RwLock<Box<BaseThing>>> {
  let mut thing = BaseThing::new(
    format!("urn:dev:ops:somfy-rts-{}", remote.address()),
      name.to_owned(),
      Some(vec!["MultiLevelSwitch".to_owned(), "Blind".to_owned()]),
      Some("A Somfy RTS blind".to_owned()),
  );

  let position_description = json!({
      "@type": "LevelProperty",
      "title": "Position",
      "type": "integer",
      "description": "The current position of the blind from 0-100",
      "minimum": 0,
      "maximum": 100,
      "unit": "percent"
  });
  let position_description = position_description.as_object().unwrap().clone();
  thing.add_property(Box::new(BaseProperty::new(
      "position".to_owned(),
      json!(50),
      None,
      Some(position_description),
  )));

  let move_metadata = json!({
      "title": "Move",
      "description": "Move the blind to a given position",
      "input": {
          "type": "object",
          "required": [
              "position"
          ],
          "properties": {
              "position": {
                  "type": "integer",
                  "minimum": 0,
                  "maximum": 100,
                  "unit": "percent"
              }
          }
      }
  });
  let move_metadata = move_metadata.as_object().unwrap().clone();
  thing.add_available_action("move".to_owned(), move_metadata);

  Arc::new(RwLock::new(Box::new(thing)))
}
