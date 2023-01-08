use tide::Server;
use tide::{Body, Request, Response, StatusCode};

use crate::State;
use crate::MusicFile;

pub fn public_api(app: &mut Server<State>) {
  app.at("/files").get(|req: Request<State>| async move {
    let pool = &req.state().db;

    let rows: Vec<MusicFile> =
      sqlx::query_as!(MusicFile, r#"SELECT id, filepath FROM files"#).fetch_all(pool).await?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(Body::from_json(&rows)?);
    Ok(res)
  });
}