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

/// 与 `Edit` 页面的表单对应, 也用于首页.
#[derive(FromForm, Serialize)]
pub struct EditForm {
    pub id: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub notes: String,
    pub favorite: bool,
}

/// 只有一个 id 的表单, 用于删除等.
#[derive(FromForm, Serialize)]
pub struct IdForm {
    pub id: String,
}

/// 用于向网页返回 Flash 信息.
#[derive(Debug, Serialize, Default)]
pub struct FlashContext<'a> {
    pub msg: Option<&'a str>,
}

/// 网页模板数据, 用于 `add.html.tera`
#[derive(Serialize, Default)]
pub struct AddContext<'a, 'b> {
    pub msg: Option<&'a str>,
    pub form_data: Option<&'b AddForm>,
}

/// 向前端返回搜索结果，结果可能为空，也可能包含一条或多条数据。
#[derive(Serialize, Default)]
pub struct ResultContext<'a> {
    pub msg: Option<&'a str>,
    pub result: Vec<EditForm>,
}
