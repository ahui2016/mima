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
                logout,
                timeout,
                logged_in,
                add_page,
                add,
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
    let msg = match flash {
        Some(ref f) => Some(f.msg()),
        None => None,
    };
    Template::render("new-account", &FlashContext { msg })
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
        Some(ref f) => FlashContext { msg: Some(f.msg()) },
        None => FlashContext { msg: None },
    };
    Template::render("login", &context)
}

#[post("/login", data = "<form>")]
fn login(form: Form<LoginForm>, state: State<Login>, conn: DbConn) -> Flash<Redirect> {
    let pwd = form.password.to_owned();
    let result = allmima::table
        .filter(allmima::title.eq("").and(allmima::username.eq("")))
        .select(pgp_sym_decrypt(allmima::password, &pwd))
        .get_result::<String>(&conn as &PgConnection);
    match result {
        Ok(decrypted) => {
            let mut l_pwd = state.password.lock().unwrap();
            let mut dt = state.datetime.lock().unwrap();
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

#[get("/logout")]
fn logout(state: State<Login>) -> Flash<Redirect> {
    let mut dt = state.datetime.lock().unwrap();
    *dt = *dt - state.period;
    Flash::success(Redirect::to(uri!(login)), "logged out.")
}

#[get("/timeout")]
fn timeout() -> Template {
    Template::render("timeout", FlashContext::new())
}

#[get("/logged-in")]
fn logged_in() -> Template {
    Template::render("logged-in", FlashContext::new())
}

#[get("/add")]
fn add_page() -> Template {
    Template::render("add", AddContext::new())
}

#[post("/add", data = "<form>")]
fn add(
    form: Form<AddForm>,
    state: State<Login>,
    conn: DbConn,
) -> Result<Flash<Redirect>, Template> {
    let form_data = AddForm {
        title: form.title.trim().into(),
        username: form.username.trim().into(),
        password: form.password.to_owned(),
        notes: form.notes.trim().into(),
    };

    if form_data.title.is_empty() {
        return Err(Template::render(
            "add",
            &AddContext {
                msg: Some("title不能为空。"),
                form_data: Some(&form_data),
            },
        ));
    }

    let pwd = state.password.lock().unwrap().clone();
    let result = diesel::insert_into(allmima::table)
        .values((
            allmima::id.eq(Uuid::new_v4().to_simple().to_string()),
            allmima::title.eq(&form_data.title),
            allmima::username.eq(&form_data.username),
            allmima::created.eq(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
        ))
        .execute(&conn as &PgConnection);

    let resp = match result {
        Err(err) => {
            let err_info = format!("{}", err);
            if !err_info.contains("allmima_title_username_deleted_key") {
                panic!(err);
            }
            Err(Template::render(
                "add",
                &AddContext {
                    msg: Some("冲突：数据库中已有相同的 title, username。"),
                    form_data: Some(&form_data),
                },
            ))
        }
        Ok(_) => Ok(Flash::success(Redirect::to(uri!(get_password)), "ok")),
    };

    if !form_data.password.is_empty() {
        diesel::insert_into(allmima::table)
            .values(allmima::password.eq(pgp_sym_encrypt(&form_data.password, &pwd)))
            .execute(&conn as &PgConnection)
            .unwrap();
    }
    if !form_data.notes.is_empty() {
        diesel::insert_into(allmima::table)
            .values(allmima::notes.eq(pgp_sym_encrypt(&form_data.notes, &pwd)))
            .execute(&conn as &PgConnection)
            .unwrap();
    }
    resp
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
        if request.uri().path() == "/logout" {
            return;
        }
        let conn = request.guard::<DbConn>().unwrap();
        let result = allmima::table
            .select(allmima::id)
            .first::<String>(&conn as &PgConnection);

        let state = request.guard::<State<Login>>().unwrap();
        let mut pwd = state.password.lock().unwrap();
        match result {
            Err(err) => {
                if err != diesel::NotFound {
                    panic!(err);
                }
                // Not Found (数据表为空，需要创建新账户。)
                if request.uri().path() == "/new-account" {
                    return;
                }
                *pwd = String::new();
                request.set_method(Method::Get);
                request.set_uri(uri!(new_account));
            }
            Ok(_) => {
                let dt = state.datetime.lock().unwrap();
                let expired = *dt + state.period;
                if pwd.is_empty() {
                    if request.uri().path() == "/login" {
                        return;
                    }
                    request.set_method(Method::Get);
                    request.set_uri(uri!(login_page));
                    return;
                }
                if Utc::now() > expired {
                    if request.uri().path() == "/login" {
                        return;
                    }
                    request.set_method(Method::Get);
                    request.set_uri(uri!(timeout));
                    return;
                }
                // Logged in successfully.
                if vec!["/login", "/new-account", "/timeout"]
                    .iter()
                    .find(|&&x| x == request.uri().path())
                    .is_some()
                {
                    request.set_method(Method::Get);
                    request.set_uri(uri!(logged_in));
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
// struct FlashContext {
//     parent: &'static str,
// }

#[derive(FromForm)]
struct LoginForm {
    pub password: String,
}

#[derive(Debug, Serialize)]
struct FlashContext<'a> {
    msg: Option<&'a str>,
}
impl<'a> FlashContext<'a> {
    fn new() -> FlashContext<'a> {
        FlashContext { msg: None }
    }
}

#[derive(FromForm, Serialize)]
pub struct AddForm {
    pub title: String,
    pub username: String,
    pub password: String,
    pub notes: String,
}

#[derive(Serialize)]
struct AddContext<'a, 'b> {
    msg: Option<&'a str>,
    form_data: Option<&'b AddForm>,
}
impl<'a, 'b> AddContext<'a, 'b> {
    fn new() -> AddContext<'a, 'b> {
        AddContext {
            msg: None,
            form_data: None,
        }
    }
}
