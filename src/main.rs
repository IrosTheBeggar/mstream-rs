use serde::{Deserialize, Serialize};
use sqlx::{
  sqlite::{SqlitePool, SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use std::{time::{Duration}, io::ErrorKind};
use std::{ str::FromStr };

use tide::utils::After;
use tide::Response;
use tide::StatusCode;
use tide::Request;
use tide::Body;

use crate::conf::Config;
use crate::conf::Load;
mod conf;

mod api;

#[derive(Clone)]
pub struct State {
  db: SqlitePool,
  conf: Config
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MusicFile {
  pub id: String,
  pub filepath: String,
  // pub metadata: Option<String>
}

#[async_std::main]
async fn main() -> Result<(),Box<dyn std::error::Error>> {
  let conf = Config::init();
  let state = State { db: setup_db(&conf.dbFile).await?, conf };
  let port = state.conf.port.clone();
  let lol = state.clone();

  sqlx::migrate!("./migrations").run(&state.db).await?;

  tide::log::start();

  let mut app = tide::with_state(state);
  app.with(tide::log::LogMiddleware::new());

  // Error Handling
  app.with(After(|mut res: Response| async {
    if let Some(err) = res.downcast_error::<async_std::io::Error>() {
      if let ErrorKind::NotFound = err.kind() {
        let msg = format!("Error: {:?}", err);
        res.set_status(StatusCode::NotFound);

        // NOTE: You may want to avoid sending error messages in a production server.
        res.set_body(msg);
      }
    }
    Ok(res)
  }));
  
  app.at("/test-error")
  .get(|_req: Request<_>| async { Ok(Body::from_file("./does-not-exist").await?) });

  api::public::public_api(&mut app);
  api::auth::auth_api(&mut app);

  app.at("/admin").nest({
    let mut admin_routes = tide::with_state(lol);
    api::admin::admin_api(&mut admin_routes);
    admin_routes
  });


  app.listen(format!("127.0.0.1:{}", &port.to_string())).await?;
  Ok(())
}

async fn setup_db(db_file: &String) -> Result<SqlitePool, Box<dyn std::error::Error>> {
  let database_url = format!("sqlite://{}", db_file);

  let pool_timeout = Duration::from_secs(30);
  let connection_options = SqliteConnectOptions::from_str(&database_url)?
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)
    .synchronous(SqliteSynchronous::Normal)
    .busy_timeout(pool_timeout);

  let pool = SqlitePoolOptions::new()
    .connect_with(connection_options)
    .await?;

  Ok(pool)
}