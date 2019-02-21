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

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Binary, Nullable, Text};
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
use schema::allmima;

const EPOCH: &str = "1970-01-01T00:00:00Z";
static PASSWORD: &'static str = "abcabc";
sql_function!(pgp_sym_encrypt, T_pgp_sym_encrypt, (x: Text, y:Text) -> Nullable<Binary>);
sql_function!(pgp_sym_decrypt, T_pgp_sym_decrypt, (x: Binary, y:Text) -> Text);

#[database("mimadb")]
pub struct DbConn(diesel::PgConnection);

struct Login {
    password: Mutex<String>,
    datetime: Mutex<DateTime<Utc>>,
    period: Duration, // datetime + period == expired time
}

fn main() {
    dotenv().ok();
    let _rocket_databases = env::var("ROCKET_DATABASES").expect("ROCKET_DATABASES must be set");

    let password_state = Login {
        password: Mutex::new(String::new()),
        datetime: Mutex::new(Utc::now()),
        period: Duration::minutes(30), // the default life of the password
    };

    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(LoginFairing)
        .mount(
            "/",
            routes![new_account, create_account, get_password, change_password],
        )
        .attach(Template::fairing())
        .manage(password_state)
        .launch();
}

#[get("/new-account")]
fn new_account(flash: Option<FlashMessage>) -> Template {
    let context = match flash {
        Some(ref f) => TemplateContext { msg: Some(f.msg()) },
        None => TemplateContext {
            msg: Some("no flash"),
        },
    };
    Template::render("new-account", &context)
}
#[post("/new-account", data = "<form>")]
fn create_account(form: Form<NewAccount>, state: State<Login>, conn: DbConn) -> Flash<Redirect> {
    if form.password.is_empty() {
        return Flash::error(Redirect::to(uri!(new_account)), "password cannot be empty.");
    }
    diesel::insert_into(allmima::table)
        .values((
            allmima::id.eq(Uuid::new_v4().to_simple().to_string()),
            allmima::password.eq(pgp_sym_encrypt("TODO: random password here", PASSWORD)),
            allmima::created.eq(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
        ))
        .execute(&conn as &PgConnection)
        .unwrap();

    let mut pwd = state.password.lock().unwrap();
    *pwd = form.password.to_owned();
    let mut dt = state.datetime.lock().unwrap();
    *dt = Utc::now();
    Flash::success(Redirect::to(uri!(get_password)), "")
}

#[get("/get-password")]
fn get_password(_flash: Option<FlashMessage>, state: State<Login>) -> String {
    format!(
        "Password is {}\n生效时间：{}",
        state.password.lock().unwrap(),
        state.datetime.lock().unwrap().to_rfc3339_opts(SecondsFormat::Secs, true),
    )
}

#[get("/change-password")]
fn change_password(state: State<Login>) -> Redirect {
    let mut pwd = state.password.lock().unwrap();
    *pwd = "new password".to_string();
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
        let conn = request.guard::<DbConn>().unwrap();
        //         let database_url = env::var("DATABASE_URL").unwrap();
        //         let conn = PgConnection::establish(&database_url).unwrap();
        let result = allmima::table
            .select(allmima::id)
            .first::<String>(&conn as &PgConnection);
        if result.is_err() && result.unwrap_err() == diesel::NotFound {
            if request.uri().path() == "/new-account" {
                return;
            }
            request.set_method(Method::Get);
            request.set_uri(Origin::parse("/new-account").unwrap());
        } else {
            request.set_uri(Origin::parse("/get-password").unwrap());
        }
        //         let password = allmima
        //             .filter(
        //                 allmima::title
        //                     .eq("")
        //                     .and(allmima::username.eq("")),
        //             )
        //             .select(pgp_sym_decrypt(allmima::password, PASSWORD))
        //             .get_result();
    }
}

// #[derive(Serialize, Insertable, Queryable, Debug, Clone)]
// pub struct Mima {
//     pub id: String,
//     pub title: String,
//     pub username: String,
//     pub passowrd: Option<Vec<u8>>,
//     pub notes: Option<Vec<u8>>,
//     pub favorite: bool,
//     pub created: String,
//     pub deleted: String,
// }

// #[derive(Serialize)]
// struct TemplateContext {
//     parent: &'static str,
// }

#[derive(FromForm)]
struct NewAccount {
    pub password: String,
}

#[derive(Debug, Serialize)]
struct TemplateContext<'a> {
    msg: Option<&'a str>,
}
