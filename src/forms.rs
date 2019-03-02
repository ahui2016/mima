use sodiumoxide::crypto::hash::sha256;
use sodiumoxide::crypto::secretbox;

/// 创建新账户或登入时使用.
#[derive(FromForm)]
pub struct LoginForm {
    pub password: String,
}

impl LoginForm {
    /// 把字符串密码转换为 secretbox::Key
    pub fn pwd_to_key(&self) -> secretbox::Key {
        let pwd = sha256::hash(self.password.as_bytes());
        secretbox::Key::from_slice(pwd.as_ref()).unwrap()
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

/// 用于向网页返回 Flash 信息.
#[derive(Debug, Serialize)]
pub struct FlashContext<'a> {
    pub msg: Option<&'a str>,
}

impl<'a> FlashContext<'a> {
    /// 创建一个空的实例, 里面的 msg 为 None.
    pub fn new() -> FlashContext<'a> {
        FlashContext { msg: None }
    }
}

/// 网页模板数据, 用于 `add.html.tera`
#[derive(Serialize)]
pub struct AddContext<'a, 'b> {
    pub msg: Option<&'a str>,
    pub form_data: Option<&'b AddForm>,
}

impl<'a, 'b> AddContext<'a, 'b> {
    /// 创建一个空的实例, 里面的 msg 和 form_data 均为 None.
    pub fn new() -> AddContext<'a, 'b> {
        AddContext {
            msg: None,
            form_data: None,
        }
    }
}
