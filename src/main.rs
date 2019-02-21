#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

extern crate chrono;
extern crate dotenv;
extern crate uuid;
// extern crate time;

use std::borrow::BorrowMut;
use std::env;
use std::sync::Mutex;

use chrono::{Duration, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Binary, Text};
use dotenv::dotenv;
use rocket::fairing::{self, Fairing};
use rocket::http::uri::Origin;
use rocket::http::Method;
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket::{Data, Request, State};
use rocket_contrib::templates::Template;
use uuid::Uuid;

mod schema;

static PASSWORD: &'static str = "abcabc";
sql_function!(pgp_sym_encrypt, T_pgp_sym_encrypt, (x: Text, y:Text) -> Binary);
sql_function!(pgp_sym_decrypt, T_pgp_sym_decrypt, (x: Binary, y:Text) -> Text);

#[database("mimadb")]
pub struct DbConn(diesel::PgConnection);

struct Login {
    password: Mutex<&'static str>,
    datetime: NaiveDateTime,
    period: Duration, // datetime + period == expired time
}

fn main() {
    dotenv().ok();
    let _rocket_databases = env::var("ROCKET_DATABASES").expect("ROCKET_DATABASES must be set");

    let password_state = Login {
        password: Mutex::new(""),
        datetime: Utc::now().naive_utc(),
        period: Duration::minutes(30), // the default life of the password
    };

    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/", routes![new_account, get_password, change_password])
        .attach(Template::fairing())
        .manage(password_state)
        .attach(LoginFairing)
        .launch();
}

#[get("/new_account")]
fn new_account(state: State<Login>, conn: DbConn) -> String {
    "It's a new account.".into()
}

#[get("/get_password")]
fn get_password(state: State<Login>) -> String {
    format!("Password is {}", state.password.lock().unwrap())
}

#[get("/change_password")]
fn change_password(state: State<Login>) -> Redirect {
    let mut pwd = state.password.lock().unwrap();
    *pwd = "new password";
    Redirect::to(uri!(get_password))
}

pub struct LoginFairing;
impl Fairing for LoginFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "GET/Post Login",
            kind: fairing::Kind::Request,
        }
    }
    fn on_request(&self, request: &mut Request, _: &Data) {
        let database_url = env::var("DATABASE_URL").unwrap();
        let conn = PgConnection::establish(&database_url).unwrap();
        let result = schema::allmima::table
            .select(schema::allmima::id)
            .first::<String>(&conn);
        if result.unwrap_err() == diesel::NotFound {
            request.set_method(Method::Get);
            request.set_uri(Origin::parse("/new_account").unwrap());
        } else {
            request.set_uri(Origin::parse("/get_password").unwrap());
        }
        //         let password = schema::allmima
        //             .filter(
        //                 schema::allmima::title
        //                     .eq("")
        //                     .and(schema::allmima::username.eq("")),
        //             )
        //             .select(pgp_sym_decrypt(schema::allmima::password, PASSWORD))
        //             .get_result();
    }
}

#[derive(Serialize, Queryable, Debug, Clone)]
pub struct Mima {
    pub id: String,
    pub title: String,
    pub username: String,
    pub passowrd: Option<Vec<u8>>,
    pub notes: Option<Vec<u8>>,
    pub favorite: bool,
    pub created: String,
    pub deleted: String,
}
