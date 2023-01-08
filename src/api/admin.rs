use std::{thread, collections::HashSet};
use futures::executor;
use tide::{Body, Next, Request, Response, Result, StatusCode, Server};
use std::time::Duration;
use sqlx::{ sqlite::{SqlitePool} };
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;
use std::path::Path;
use argon2::{
  password_hash::{
      rand_core::OsRng,
      PasswordHasher, SaltString
  },
  Argon2
};
use jsonwebtoken::{ decode, DecodingKey, Validation, Algorithm };
use std::future::Future;
use std::pin::Pin;

use crate::State;

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct AddUserBody {
  username: String,
  password: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  user: String,
  exp: Option<usize>
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct User {
  pub username: String,
  pub id: String
}

fn user_loader<'a>(
  mut request: Request<State>, // wtf
  next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
  Box::pin(async {
    // decode JWT
    let token = request.header("x-token").unwrap().last().to_string();
    eprintln!("LOL TOKEN `{}`", &token);
    eprintln!("LOL DECODE `{}`", &request.state().conf.secret);

    let mut valid = Validation::new(Algorithm::HS256);
    valid.validate_exp = false;

    let books: HashSet<String> = HashSet::new();
    valid.required_spec_claims = books;
    let token_message = decode::<Claims>(&token, &DecodingKey::from_secret(&request.state().conf.secret.as_bytes()), &valid);
    let username = token_message.unwrap().claims.user;


    let row: User = sqlx::query_as!(User, r#"SELECT id, username FROM users WHERE username = ?"#, username)
    // .bind(&req_body.username)
    .fetch_one(&request.state().db).await?;
    request.set_ext(row);


    Ok(next.run(request).await)
  })
}

pub fn admin_api(app: &mut Server<State>) {
  app.at("/user").put(|mut req: Request<State>| async move {    
    let req_body: AddUserBody = req.body_json().await?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(&req_body.password.as_bytes(), &salt).unwrap().to_string();

    let sql_query = "INSERT INTO users(username,id,password,salt) VALUES(?,?,?,?);";
    sqlx::query(sql_query)
      .bind(&req_body.username)
      .bind(Uuid::new_v4().to_string())
      .bind(&password_hash)
      .bind(&salt.to_string())
      .execute(&req.state().db).await?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(Body::from_json(&req_body)?);
    Ok(res)
  });

  app.with(user_loader);

  app.at("/scan").get(|req: Request<State>| async move {  
    let user: &User = req.ext().unwrap();
    eprintln!("LOL USER `{}`", user.username);
    
    
    scan_on_new_thread(req.state().conf.musicFolder.clone().to_string(), req.state().db.clone());

    Ok("{}")
  });
}

fn scan_on_new_thread(folder: String, sql: SqlitePool) {
  thread::spawn(move || {
    let scan_id = Uuid::new_v4().to_string();
    recursive_scan(&folder, &sql, &scan_id);

    // Delete files that no longer exist
    let delete_sql = "DELETE FROM files WHERE scan_id NOT IN (?)";
    executor::block_on(sqlx::query(delete_sql)
      .bind(&scan_id)
      .execute(&sql));

    thread::sleep(Duration::from_millis(1000));
  });
}

fn recursive_scan(folder: &String, sql: &SqlitePool, scan_id: &String) {
  let paths = fs::read_dir(&folder).unwrap();
  for path in paths {
    let foo = path.unwrap();

    let p = &foo.file_name().to_string_lossy().into_owned();

    if foo.path().is_dir() == true {
      let joined: String = Path::new(&folder).join(&p).into_os_string().into_string().unwrap();
      recursive_scan(&joined, sql, scan_id);
      continue;
    }

    let x = "INSERT INTO files(filepath,id,scan_id) VALUES(?,?,?)
    ON CONFLICT(filepath) DO UPDATE SET scan_id= ?;";
    executor::block_on(sqlx::query(x)
      .bind(p)
      .bind(Uuid::new_v4().to_string())
      .bind(scan_id)
      .bind(scan_id)
      .execute(sql));
  }
}