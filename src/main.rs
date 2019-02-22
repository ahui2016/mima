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
// static EPOCH_UTC: DateTime<Utc> = DateTime::parse_from_rfc3339(EPOCH)
//     .unwrap()
//     .with_timezone(&Utc);
sql_function!(pgp_sym_encrypt, T_pgp_sym_encrypt, (x: Text, y:Text) -> Nullable<Binary>);
sql_function!(pgp_sym_decrypt, T_pgp_sym_decrypt, (x: Nullable<Binary>, y:Text) -> Text);

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
        period: Duration::minutes(30), // 默认有效时长
    };

    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(LoginFairing)
        .mount(
            "/",
            routes![
                new_account,
                create_account,
                login_page,
                login,
                timeout,
                logged_in,
                get_password,
                change_password
            ],
        )
        .attach(Template::fairing())
        .manage(password_state)
        .launch();
}

#[get("/new-account")]
fn new_account(flash: Option<FlashMessage>) -> Template {
    let context = match flash {
        Some(ref f) => TemplateContext { msg: Some(f.msg()) },
        None => TemplateContext { msg: None },
    };
    Template::render("new-account", &context)
}

#[post("/new-account", data = "<form>")]
fn create_account(form: Form<LoginForm>, conn: DbConn) -> Flash<Redirect> {
    if form.password.is_empty() {
        return Flash::error(Redirect::to(uri!(new_account)), "密码不能为空。");
    }
    diesel::insert_into(allmima::table)
        .values((
            allmima::id.eq(Uuid::new_v4().to_simple().to_string()),
            allmima::password.eq(pgp_sym_encrypt(
                "TODO: random password here",
                form.password.to_owned(),
            )),
            allmima::created.eq(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
        ))
        .execute(&conn as &PgConnection)
        .unwrap();

    Flash::success(
        Redirect::to(uri!(login_page)),
        "创建新账户成功！请登入。",
    )
}

#[get("/login")]
fn login_page(flash: Option<FlashMessage>) -> Template {
    let context = match flash {
        Some(ref f) => TemplateContext { msg: Some(f.msg()) },
        None => TemplateContext { msg: None },
    };
    Template::render("login", &context)
}

#[post("/login", data = "<form>")]
fn login(form: Form<LoginForm>, login: State<Login>, conn: DbConn) -> Flash<Redirect> {
    let pwd = form.password.to_owned();
    let result = allmima::table
        .filter(allmima::title.eq("").and(allmima::username.eq("")))
        .select(pgp_sym_decrypt(allmima::password, &pwd))
        .get_result::<String>(&conn as &PgConnection);
    match result {
        Ok(decrypted) => {
            let mut l_pwd = login.password.lock().unwrap();
            let mut dt = login.datetime.lock().unwrap();
            *l_pwd = pwd;
            *dt = Utc::now();
            Flash::success(Redirect::to(uri!(get_password)), decrypted)
        }
        Err(err) => {
            let err_info = format!("{}", err);
            if err_info.contains("Wrong key or corrupt data") {
                return Flash::error(Redirect::to(uri!(login_page)), "密码错误。");
            }
            panic!(err)
        }
    }
}

#[get("/timeout")]
fn timeout() -> Template {
    Template::render("timeout", "")
}

#[get("/logged-in")]
fn logged_in() -> Template {
    Template::render("logged-in", "")
}

#[get("/get-password")]
fn get_password(flash: Option<FlashMessage>, state: State<Login>) -> String {
    let msg = match flash {
        Some(ref f) => f.msg(),
        None => "No flash message.",
    };
    format!(
        "Flash: {}\nPassword is {}\n生效时间：{}",
        msg,
        state.password.lock().unwrap(),
        state
            .datetime
            .lock()
            .unwrap()
            .to_rfc3339_opts(SecondsFormat::Secs, true),
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
        let result = allmima::table
            .select(allmima::id)
            .first::<String>(&conn as &PgConnection);

        match result {
            Err(err) => {
                if err != diesel::NotFound {
                    panic!(err);
                }
                // Not Found (数据表为空，需要创建新账户。)
                if request.uri().path() == "/new-account" {
                    return;
                }
                request.set_method(Method::Get);
                request.set_uri(Origin::parse("/new-account").unwrap());
            }
            Ok(_) => {
                let login = request.guard::<State<Login>>().unwrap();
                let pwd = login.password.lock().unwrap();
                let dt = login.datetime.lock().unwrap();
                let expired = *dt + login.period;
                if pwd.is_empty() {
                    if request.uri().path() == "/login" {
                        return;
                    }
                    request.set_method(Method::Get);
                    request.set_uri(Origin::parse("/login").unwrap());
                    return;
                }
                if Utc::now() > expired {
                    if request.uri().path() == "/login" {
                        return;
                    }
                    request.set_method(Method::Get);
                    request.set_uri(Origin::parse("/timeout").unwrap());
                    return;
                }
                // Logged in successfully.
                if vec!["/login", "/new-account", "/timeout"]
                    .iter()
                    .find(|&&x| x == request.uri().path())
                    .is_some()
                {
                    request.set_method(Method::Get);
                    request.set_uri(Origin::parse("/logged-in").unwrap());
                }
            }
        };
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
struct LoginForm {
    pub password: String,
}

#[derive(Debug, Serialize)]
struct TemplateContext<'a> {
    msg: Option<&'a str>,
}
