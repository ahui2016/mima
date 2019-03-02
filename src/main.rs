#![feature(proc_macro_hygiene, decl_macro)]

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

extern crate base64;
extern crate chrono;
extern crate dotenv;
extern crate sodiumoxide;
extern crate uuid;

use std::env;
use std::str;
use std::sync::Mutex;

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use diesel::prelude::*;
// use diesel::sql_types::{Binary, Nullable, Text};
use dotenv::dotenv;
use rocket::fairing::{self, Fairing};
use rocket::http::Method;
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket::{Data, Request, State};
use rocket_contrib::templates::Template;
use sodiumoxide::crypto::hash::sha256;
use sodiumoxide::crypto::secretbox;
use uuid::Uuid;

mod schema;
use schema::allmima;

const EPOCH: &str = "1970-01-01T00:00:00Z";
// static EPOCH_UTC: DateTime<Utc> = DateTime::parse_from_rfc3339(EPOCH)
//     .unwrap()
//     .with_timezone(&Utc);

// sql_function!(pgp_sym_encrypt, T_pgp_sym_encrypt, (x: Text, y:Text) -> Nullable<Binary>);
// sql_function!(pgp_sym_decrypt, T_pgp_sym_decrypt, (x: Nullable<Binary>, y:Text) -> Text);

#[database("mimadb")]
pub struct DbConn(diesel::PgConnection);

/// - `Login` 是一个全局变量, 非常重要.
/// - 每一个请求进来时, 都通过 `LoginFairing` 检查 `Login` 的状态 (如: 密码是否为空, 有没有超时).
/// - 在 `main` 中对 `Login` 进行初始化.
struct Login {
    key: Mutex<secretbox::Key>,
    datetime: Mutex<DateTime<Utc>>,
    period: Duration, // datetime + period == expired time
}

fn main() {
    dotenv().ok();
    let _rocket_databases = env::var("ROCKET_DATABASES").expect("ROCKET_DATABASES must be set");

    let password_state = Login {
        key: Mutex::new(secretbox::Key::new()),
        datetime: Mutex::new(Utc::now()),
        period: Duration::minutes(30), // 默认有效时长
    };

    sodiumoxide::init().unwrap();

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

    let key = form.pwd_to_key();
    let p_nonce = secretbox::gen_nonce();
    let encrypted = secretbox::seal(b"TODO: random bytes here.", &p_nonce, &key);

    diesel::insert_into(allmima::table)
        .values((
            allmima::id.eq(uuid_simple()),
            allmima::password.eq(encrypted),
            allmima::p_nonce.eq(p_nonce.as_ref()),
            allmima::created.eq(now_string()),
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
    let try_key = form.pwd_to_key();
    let (encrypted, p_nonce) = allmima::table
        .filter(allmima::title.eq("").and(allmima::username.eq("")))
        .select((allmima::password, allmima::p_nonce))
        .get_result::<(Option<Vec<u8>>, Option<Vec<u8>>)>(&conn as &PgConnection)
        .unwrap();
    let encrypted = encrypted.unwrap();
    let p_nonce = p_nonce.unwrap();
    let p_nonce = secretbox::Nonce::from_slice(&p_nonce).unwrap();
    let decrypted = secretbox::open(&encrypted, &p_nonce, &try_key);
    match decrypted {
        Ok(text_bytes) => {
            let mut key = state.key.lock().unwrap();
            let mut dt = state.datetime.lock().unwrap();
            *key = try_key;
            *dt = Utc::now();
            Flash::success(
                Redirect::to(uri!(get_password)),
                str::from_utf8(&text_bytes).unwrap(),
            )
        }
        Err(_) => Flash::error(Redirect::to(uri!(login_page)), "密码错误。"),
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
///
/// 这里调用 `Login` 的密码, 由于在登入时已验证过密码, 可以认为这是正确该密码.
#[post("/add", data = "<form>")]
fn add(
    form: Form<AddForm>,
    state: State<Login>,
    conn: DbConn,
) -> Result<Flash<Redirect>, Template> {
    let key = state.key.lock().unwrap();
    let form_data = MimaItem::from_add_form(form.into_inner(), &key);
    let add_form = form_data.to_add_form(&key);

    if form_data.title.is_empty() {
        return Err(Template::render(
            "add",
            &AddContext {
                msg: Some("title不能为空。"),
                form_data: Some(&add_form),
            },
        ));
    }

    let result = form_data.insert(&conn);

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
                    form_data: Some(&add_form),
                },
            ))
        }
        // TODO: 成功后跳转到主页
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
        "Flash: {}\nPassword is {:?}\n生效时间：{}",
        msg,
        state.key.lock().unwrap(),
        state
            .datetime
            .lock()
            .unwrap()
            .to_rfc3339_opts(SecondsFormat::Secs, true),
    )
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
        let mut key = state.key.lock().unwrap();
        match result {
            // Not Found (数据表为空，需要创建新账户。)
            Err(err) => {
                if err != diesel::NotFound {
                    panic!(err);
                }
                if request.uri().path() == "/new-account" {
                    return;
                }
                *key = secretbox::Key::new();
                request.set_method(Method::Get);
                request.set_uri(uri!(new_account));
            }
            Ok(_) => {
                // 数据表里有数据（非新用户），但没有密码（未登入）
                if key.is_empty() {
                    if request.uri().path() == "/login" {
                        return;
                    }
                    request.set_method(Method::Get);
                    request.set_uri(uri!(login_page));
                    return;
                }

                let dt = state.datetime.lock().unwrap();
                let expired = *dt + state.period;

                // 有密码（已登入），但过期（超时）
                if Utc::now() > expired {
                    if request.uri().path() == "/login" {
                        return;
                    }
                    request.set_method(Method::Get);
                    request.set_uri(uri!(timeout));
                    return;
                }

                // 上述特殊情况皆不成立（已成功登入）
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

/// 与数据表 `allmima` 的结构一一对应.
#[table_name = "allmima"]
#[derive(Serialize, Insertable, Queryable, Identifiable, Debug, Clone)]
pub struct MimaItem {
    pub id: String,
    pub title: String,
    pub username: String,
    pub password: Option<Vec<u8>>,
    pub p_nonce: Option<Vec<u8>>,
    pub notes: Option<Vec<u8>>,
    pub n_nonce: Option<Vec<u8>>,
    pub favorite: bool,
    pub created: String,
    pub deleted: String,
}

impl MimaItem {
    /// 对 MimaItem 里的 password 或 notes 进行加密
    fn encrypt(plaintext: &str, key: &secretbox::Key) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
        match plaintext.is_empty() {
            true => (None, None),
            false => {
                let nonce = secretbox::gen_nonce();
                let encrypted = secretbox::seal(plaintext.as_bytes(), &nonce, key);
                (Some(encrypted), Some(nonce.to_vec()))
            }
        }
    }

    /// 用于处理 `add` 页面的表单.
    fn from_add_form(form: AddForm, key: &secretbox::Key) -> MimaItem {
        let (password, p_nonce) = Self::encrypt(form.password.as_str(), key);
        let (notes, n_nonce) = Self::encrypt(form.notes.trim(), key);
        MimaItem {
            id: uuid_simple(),
            title: form.title.trim().into(),
            username: form.username.trim().into(),
            password,
            p_nonce,
            notes,
            n_nonce,
            favorite: false,
            created: now_string(),
            deleted: EPOCH.into(),
        }
    }

    /// 当添加数据失败时, 为了避免用户原本输入的数据丢失, 需要向页面返回表单内容.
    fn to_add_form(&self, key: &secretbox::Key) -> AddForm {
        AddForm {
            title: self.title.clone(),
            username: self.username.clone(),
            password: String::new(),
            notes: self.notes_decrypt(key).to_string(),
        }
    }

    /// 把 Some(Vec<u8>) 转换为 secretbox::Nonce
    fn get_nonce(vec: Option<&Vec<u8>>) -> secretbox::Nonce {
        secretbox::Nonce::from_slice(vec.unwrap()).unwrap()
    }

    /// 对 MimaItem 里的 password 或 notes 进行解密, 返回字符串.
    ///
    /// 如果被解密参数为 None, 则返回空字符串.
    fn decrypt(
        encrypted: Option<&Vec<u8>>,
        nonce: Option<&Vec<u8>>,
        key: &secretbox::Key,
    ) -> String {
        match encrypted {
            Some(vec) => {
                let nonce = Self::get_nonce(nonce);
                let decrypted = secretbox::open(vec, &nonce, key).unwrap();
                String::from_utf8(decrypted).unwrap()
            }
            None => String::new(),
        }
    }

    /// 获取解密后的 MimaItem.password
    fn pwd_decrypt(&self, key: &secretbox::Key) -> String {
        Self::decrypt(self.password.as_ref(), self.p_nonce.as_ref(), key)
    }

    /// 获取解密后的 MimaItem.notes
    fn notes_decrypt(&self, key: &secretbox::Key) -> String {
        Self::decrypt(self.notes.as_ref(), self.n_nonce.as_ref(), key)
    }

    /// 向数据库中插入一条新项目.
    fn insert(&self, conn: &PgConnection) -> diesel::result::QueryResult<usize> {
        diesel::insert_into(allmima::table)
            .values(self)
            .execute(conn)
    }
}

/// 创建新账户或登入时使用.
#[derive(FromForm)]
struct LoginForm {
    pub password: String,
}

impl LoginForm {
    /// 把字符串密码转换为 secretbox::Key
    fn pwd_to_key(&self) -> secretbox::Key {
        let pwd = sha256::hash(self.password.as_bytes());
        secretbox::Key::from_slice(pwd.as_ref()).unwrap()
    }
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

/// 给 secretbox::Key 增加几个方法, 方便使用.
trait MySecretKey {
    fn new() -> secretbox::Key;
    fn is_empty(&self) -> bool;
}

impl MySecretKey for secretbox::Key {
    /// 创建一个空的 key.
    fn new() -> secretbox::Key {
        secretbox::Key::from_slice(&[0u8; secretbox::KEYBYTES]).unwrap()
    }

    /// 判断 key 是否为空
    fn is_empty(&self) -> bool {
        self == &Self::new()
    }
}

/// 当前时间的固定格式的字符串
fn now_string() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// simple格式的uuid
fn uuid_simple() -> String {
    Uuid::new_v4().to_simple().to_string()
}

/// 为了方便对 secretbox::Nonce 进行类型转换
trait NonceToVec {
    fn to_vec(self) -> Vec<u8>;
}

impl NonceToVec for secretbox::Nonce {
    /// 类型转换
    fn to_vec(self) -> Vec<u8> {
        self.0.to_vec()
    }
}
