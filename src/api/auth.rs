use tide::Server;
use tide::{Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use argon2::{
  password_hash::{
      rand_core::OsRng,
      PasswordHasher, SaltString
  },
  Argon2
};
use jsonwebtoken::{encode, Header, EncodingKey};

use crate::State;

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct LoginBody {
  username: String,
  password: String
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct User {
  pub username: String,
  pub password: String,
  pub salt: String,
  pub id: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  user: String
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct TestJson {
  req: bool,
  opt: Option<String>,
  any: Value,
  #[serde(default="test_default")]
  def: String,
  enu: String,
  rec: Option<Vec<RecursiveJson>>,
  recx: Option<Box<RecursiveJson>>
}

#[derive(Debug, Deserialize, Serialize)]
struct RecursiveJson {
  foo: String,
  rec: Option<Vec<RecursiveJson>>,
  recx: Option<Box<RecursiveJson>>
}

fn test_default() -> String {
  "test".to_string()
}

pub fn auth_api(app: &mut Server<State>) {
  app.at("/login").post(|mut req: Request<State>| async move {
    let req_body: LoginBody = req.body_json().await?;
    let username = req_body.username.clone();

    let row: User = sqlx::query_as!(User, r#"SELECT username, id, password, salt FROM users WHERE username = ?"#, username)
    // .bind(&req_body.username)
    .fetch_one(&req.state().db).await?;

    let salt: SaltString = SaltString::new(&row.salt).unwrap();
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(&req_body.password.as_bytes(), &salt).unwrap().to_string();

    if password_hash != row.password {
      let bad_res = Response::new(StatusCode::Forbidden);
      return Ok(bad_res);
    }

    let my_claims = Claims {
      user: username
    };

    let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret(&req.state().conf.secret.as_bytes())).unwrap();

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(Body::from_string(token));
    Ok(res)
  });

  app.at("/test-json").post(|mut req: Request<State>| async move {
    let req_body: TestJson = req.body_json().await?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(Body::from_json(&req_body)?);
    Ok(res)
  });
}