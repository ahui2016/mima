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
use std::str;
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
    let form_data = MimaItem::from_add_form(form.into_inner());

    if form_data.title.is_empty() {
        return Err(Template::render(
            "add",
            &AddContext {
                msg: Some("title不能为空。"),
                form_data: Some(&form_data.to_add_form()),
            },
        ));
    }

    let pwd = state.password.lock().unwrap();

    let result = form_data.insert(&conn, &pwd);

    match result {
        Err(err) => {
            let err_info = format!("{}", err);
            if !err_info.contains("allmima_title_username_deleted_key") {
                panic!(err);
            }
            Err(Template::render(
                "add",
                &AddContext {
                    msg: Some("冲突：数据库中已有相同的 title, username。"),
                    form_data: Some(&form_data.to_add_form()),
                },
            ))
        }
        Ok(_) => Ok(Flash::success(Redirect::to(uri!(get_password)), "ok")),
    }
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

#[table_name = "allmima"]
#[derive(Serialize, Insertable, Queryable, Identifiable, Debug, Clone)]
pub struct MimaItem {
    pub id: String,
    pub title: String,
    pub username: String,
    pub password: Option<Vec<u8>>,
    pub notes: Option<Vec<u8>>,
    pub favorite: bool,
    pub created: String,
    pub deleted: String,
}

impl MimaItem {
    fn from_add_form(form: AddForm) -> MimaItem {
        let password = match form.password.is_empty() {
            true => None,
            false => Some(form.password.into_bytes()),
        };
        let notes = form.notes.trim();
        let notes = match notes.is_empty() {
            true => None,
            false => Some(notes.as_bytes().to_owned()),
        };
        MimaItem {
            id: Uuid::new_v4().to_simple().to_string(),
            title: form.title.trim().into(),
            username: form.username.trim().into(),
            password,
            notes,
            favorite: false,
            created: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            deleted: EPOCH.into(),
        }
    }

    fn to_add_form(&self) -> AddForm {
        AddForm {
            title: self.title.clone(),
            username: self.username.clone(),
            password: String::new(),
            notes: self.str_notes().to_string(),
        }
    }

    fn str_password(&self) -> &str {
        match self.password.as_ref() {
            Some(vec) => str::from_utf8(vec).unwrap(),
            None => "",
        }
    }

    fn str_notes(&self) -> &str {
        match self.notes {
            Some(ref vec) => str::from_utf8(vec).unwrap(),
            None => "",
        }
    }

    fn insert(&self, conn: &PgConnection, pwd: &str) -> diesel::result::QueryResult<usize> {
        let result = diesel::insert_into(allmima::table)
            .values(self)
            .execute(conn);
        if self.password.is_some() {
            diesel::update(self)
                .set(allmima::password.eq(pgp_sym_encrypt(self.str_password(), pwd)))
                .execute(conn)
                .unwrap();
        }
        if self.notes.is_some() {
            diesel::update(self)
                .set(allmima::notes.eq(pgp_sym_encrypt(self.str_notes(), pwd)))
                .execute(conn)
                .unwrap();
        }
        result
    }
}

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
