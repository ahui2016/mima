#![feature(proc_macro_hygiene, decl_macro)]
// #![deny(missing_docs)]

//! - It's a password manager using Rust, Rocket, Diesel and PostgreSQL.
//! - 这是一个密码管理器，采用技术：Rust, Rocket, Diesel, PostgreSQL。
//!
//! - 本软件虽然采用了网站框架来制作，但只是为了方便而已。
//! - 制作时只考虑了在本地使用的情形，未考虑联网安全，不宜联网使用。
//! - 这是一个单用户系统，使用时只需要输入密码，不需要输入用户名，也无法新建第二个用户。
//!
//! - 安装时, 先参照 `create_role_and_database.md` 进行操作.
//! - 由于采用了 sodiumoxide, 因此需要设定相关的环境变量 https://crates.io/crates/sodiumoxide

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
extern crate sodiumoxide;

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

/// - `Login` 是一个全局变量, 非常重要.
/// - 每一个请求进来时, 都通过 `LoginFairing` 检查 `Login` 的状态 (如: 密码是否为空, 有没有超时).
/// - 在 `main` 中对 `Login` 进行初始化.
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

/// 创建新用户，输入新密码的页面。
#[get("/new-account")]
fn new_account(flash: Option<FlashMessage>) -> Template {
    let msg = match flash {
        Some(ref f) => Some(f.msg()),
        None => None,
    };
    Template::render("new-account", &FlashContext { msg })
}

/// **POST** 创建新用户, 服务器端处理.
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

/// 登入页面.
#[get("/login")]
fn login_page(flash: Option<FlashMessage>) -> Template {
    let context = match flash {
        Some(ref f) => FlashContext { msg: Some(f.msg()) },
        None => FlashContext { msg: None },
    };
    Template::render("login", &context)
}

/// **POST** 登入, 服务器端处理.
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

/// 暂时假装超时, 需要修改为清空密码.
#[get("/logout")]
fn logout(state: State<Login>) -> Flash<Redirect> {
    let mut dt = state.datetime.lock().unwrap();
    *dt = *dt - state.period;
    Flash::success(Redirect::to(uri!(login)), "logged out.")
}

/// 处于超时状态时, 任何请求都跳转到该页面.
#[get("/timeout")]
fn timeout() -> Template {
    Template::render("timeout", FlashContext::new())
}

/// 成功登入后, 访问 `/login`, `/new-account`, `/timeout` 时跳转到该页面.
#[get("/logged-in")]
fn logged_in() -> Template {
    Template::render("logged-in", FlashContext::new())
}

/// 添加项目的表单.
#[get("/add")]
fn add_page() -> Template {
    Template::render("add", AddContext::new())
}

/// **POST** 添加项目, 服务器端对表单进行处理.
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

/// _临时使用_, 以后要删除.
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

/// _临时使用_, 以后要删除.
#[get("/change-password")]
fn change_password(state: State<Login>) -> Redirect {
    let mut pwd = state.password.lock().unwrap();
    *pwd = "new password".to_string();
    Redirect::to(uri!(get_password))
}

/// **中间件** Kind::Request
/// 
/// 对于每一个请求, 检查 `Login` 的状态 (如: 密码是否为空, 有没有超时), 并跳转到相关页面.
pub struct LoginFairing;
impl Fairing for LoginFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "GET/Post Login",
            kind: fairing::Kind::Request,
        }
    }
    
    /// 对每一个请求进行处理.
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

/// 反映数据表 `allmima` 的结构.
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

    /// 用于处理 `add` 页面的表单.
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

    /// 当添加数据失败时, 为了避免用户原本输入的数据丢失, 需要向页面返回表单内容.
    fn to_add_form(&self) -> AddForm {
        AddForm {
            title: self.title.to_string(),
            username: self.username.clone(),
            password: String::new(),
            notes: self.str_notes().to_string(),
        }
    }

    /// 由于 password 的类型在数据库中是 Binary, 因此必要时需要转换为字符串.
    fn str_password(&self) -> &str {
        // 注意这里与 str_notes 写法不同，但达到相同的效果。
        match self.password.as_ref() {
            Some(vec) => str::from_utf8(vec).unwrap(),
            None => "",
        }
    }

    /// 由于 notes 的类型在数据库中是 Binary, 因此必要时需要转换为字符串.
    fn str_notes(&self) -> &str {
        // 注意这里与 str_password 写法不同，但本质一样，都是为了避免所有权移出。
        match self.notes {
            Some(ref vec) => str::from_utf8(vec).unwrap(),
            None => "",
        }
    }

    /// 向数据库中插入一条新项目.
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

/// 创建新账户或登入时使用.
#[derive(FromForm)]
struct LoginForm {
    pub password: String,
}

/// 用于向网页返回 Flash 信息.
#[derive(Debug, Serialize)]
struct FlashContext<'a> {
    msg: Option<&'a str>,
}

impl<'a> FlashContext<'a> {
    /// 创建一个空的实例, 里面的 msg 为 None.
    fn new() -> FlashContext<'a> {
        FlashContext { msg: None }
    }
}

/// 与 `add` 页面的表单对应.
#[derive(FromForm, Serialize)]
pub struct AddForm {
    pub title: String,
    pub username: String,
    pub password: String,
    pub notes: String,
}

/// 网页模板数据, 用于 `add.html.tera`
#[derive(Serialize)]
struct AddContext<'a, 'b> {
    msg: Option<&'a str>,
    form_data: Option<&'b AddForm>,
}

impl<'a, 'b> AddContext<'a, 'b> {

    /// 创建一个空的实例, 里面的 msg 和 form_data 均为 None.
    fn new() -> AddContext<'a, 'b> {
        AddContext {
            msg: None,
            form_data: None,
        }
    }
}
