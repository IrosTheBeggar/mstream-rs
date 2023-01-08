use toml;
use std::fs::File;
use std::fs;
use home;
use std::path::Path;
use std::process::exit;
use serde::Deserialize;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;


#[derive(Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Config {
  #[serde(default="default_port")]
  pub port: u16,
  #[serde(default="default_db_file")]
  pub dbFile: String,
  #[serde(default="default_folder")]
  pub musicFolder: String,
  #[serde(default="default_secret")]
  pub secret: String,
}

fn default_secret() -> String {
  thread_rng()
    .sample_iter(&Alphanumeric)
    .take(30)
    .map(char::from)
    .collect()
}

fn default_folder() -> String {
  let home = home::home_dir().unwrap();
  home.join("music").into_os_string().into_string().unwrap()
}

fn default_port() -> u16 {
  1111
}

fn default_db_file() -> String {
  "mstream.sqlite".to_string()
}

pub trait Load {
  fn init () -> Self;
}

impl Load for Config {
  fn init () -> Config {
    let filename = "./mstream-conf.toml";
    if !Path::new(filename).exists() {
      match File::create(filename) {
        Ok(_) => (),
        Err(_) => {
          // Write `msg` to `stderr`.
          eprintln!("Could not create file `{}`", filename);
          // Exit the program with exit code `1`.
          exit(1);
        }
      }
    }

    let contents = match fs::read_to_string(filename) {
      // If successful return the files text as `contents`.
      // `c` is a local variable.
      Ok(c) => c,
      // Handle the `error` case.
      Err(_) => {
          // Write `msg` to `stderr`.
          eprintln!("Could not read file `{}`", filename);
          // Exit the program with exit code `1`.
          exit(1);
      }
    };

    // load config file 
    let config: Config = toml::from_str(&contents).unwrap();

    config
  }
}