#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

extern crate dotenv;

use std::borrow::BorrowMut;
use std::env;
use std::sync::Mutex;

use dotenv::dotenv;
use rocket::fairing::{self, Fairing};
use rocket::http::uri::Origin;
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket::{Request, State, Data};
use rocket_contrib::templates::Template;

mod schema;

#[database("mimadb")]
pub struct DbConn(diesel::PgConnection);

struct Login {
    password: Mutex<&'static str>,
    // TODO: 增加一个时间戳
    status: LoginStatus,
}
enum LoginStatus {
    NewAccount,
    LoggedOut,
    LoggedIn,
    Expired,
}

fn main() {
    dotenv().ok();
    let _rocket_databases = env::var("ROCKET_DATABASES").expect("ROCKET_DATABASES must be set");

    let password_state = Login {
        password: Mutex::new(""),
        status: LoginStatus::NewAccount,
    };

    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/", routes![index, get_password, change_password])
        .attach(Template::fairing())
        .manage(password_state)
        .attach(LoginFairing)
        .launch();
}

#[get("/")]
fn index(state: State<Login>, conn: DbConn) -> String {
    "It's index.".into()
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
        if request.uri().path() != "/get_password" {
            request.set_uri(Origin::parse("/").unwrap());
        }
    }
}
