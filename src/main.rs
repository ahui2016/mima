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
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_contrib::templates::Template;

mod schema;

#[database("mimadb")]
pub struct DbConn(diesel::PgConnection);

struct Password {
    password: Mutex<&'static str>,
}

fn main() {
    dotenv().ok();
    let _rocket_databases = env::var("ROCKET_DATABASES")
        .expect("ROCKET_DATABASES must be set");

    let password_state = Password {
        password: Mutex::new(""),
    };

    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/", routes![get_password, change_password])
        .attach(Template::fairing())
        .manage(password_state)
        .launch();
}

#[get("/get_password")]
fn get_password(state: State<Password>) -> String {
    format!("Password is {}", state.password.lock().unwrap())
}

#[get("/change_password")]
fn change_password(state: State<Password>) -> Redirect {
    let mut pwd = state.password.lock().unwrap();
    *pwd = "new password";
    Redirect::to(uri!(get_password))
}
