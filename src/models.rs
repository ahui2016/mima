use chrono::{SecondsFormat, Utc};
use diesel::result;
use diesel::{self, prelude::*};
use sodiumoxide::crypto::secretbox;
use uuid::Uuid;

use super::forms::{AddForm, EditForm};
use super::schema::allmima;
use super::{EPOCH, FIRST_ID};

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
    /// 用于处理 `add` 页面的表单.
    pub fn from_add_form(form: AddForm, key: &secretbox::Key) -> MimaItem {
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
    pub fn to_add_form(&self, key: &secretbox::Key) -> AddForm {
        AddForm {
            title: self.title.clone(),
            username: self.username.clone(),
            password: String::new(),
            notes: self.notes_decrypt(key).to_string(),
        }
    }

    /// 返回精简的内容 (剔除了 p_nonce, n_nonce, created, deleted)
    ///
    /// password 和 notes 不解密, 适用于首页或搜索结果等.
    pub fn to_simple(&self) -> EditForm {
        let password = self
            .password
            .clone()
            .map(|_| "******".into())
            .unwrap_or_else(|| "".into());
        // 下面 notes 与 上面 password 目的一样, 只是写法不同.
        let notes = match self.notes {
            Some(_) => "******".into(),
            None => "".into(),
        };
        EditForm {
            id: self.id.clone(),
            title: self.title.clone(),
            username: self.username.clone(),
            password,
            notes,
            favorite: self.favorite,
        }
    }

    /// 返回精简的内容, 并且机密内容也解密.
    ///
    /// 适用于编辑, 删除, 回收站等页面.
    pub fn to_edit_delete(&self, key: &secretbox::Key) -> EditForm {
        EditForm {
            id: self.id.clone(),
            title: self.title.clone(),
            username: self.username.clone(),
            password: self.pwd_decrypt(key),
            notes: self.notes_decrypt(key),
            favorite: self.favorite,
        }
    }

    /// 获取解密后的 MimaItem.password
    pub fn pwd_decrypt(&self, key: &secretbox::Key) -> String {
        Self::decrypt(self.password.as_ref(), self.p_nonce.as_ref(), key)
    }

    /// 获取解密后的 MimaItem.notes
    pub fn notes_decrypt(&self, key: &secretbox::Key) -> String {
        Self::decrypt(self.notes.as_ref(), self.n_nonce.as_ref(), key)
    }

    /// 向数据库中插入一条新项目.
    pub fn insert(&self, conn: &PgConnection) -> diesel::result::QueryResult<usize> {
        diesel::insert_into(allmima::table)
            .values(self)
            .execute(conn)
    }

    /// 从数据库提取全部记录, 输出时, 机密内容不解密.
    pub fn all(conn: &PgConnection) -> Vec<EditForm> {
        allmima::table
            .filter(allmima::id.ne(FIRST_ID).and(allmima::deleted.eq(EPOCH)))
            .order(allmima::created.desc())
            .load::<MimaItem>(conn)
            .unwrap()
            .iter()
            .map(MimaItem::to_simple)
            .collect()
    }

    /// 从数据库中获取全部标记为已删除的记录 (用于"回收站")
    pub fn recyclebin(conn: &PgConnection, key: &secretbox::Key) -> Vec<EditForm> {
        allmima::table
            .filter(allmima::id.ne(FIRST_ID).and(allmima::deleted.ne(EPOCH)))
            .order(allmima::deleted.desc())
            .load::<MimaItem>(conn)
            .unwrap()
            .iter()
            .map(|item| item.to_edit_delete(key))
            .collect()
    }

    /// 通过 id 获取一条记录
    pub fn get_by_id(
        id: &str,
        conn: &PgConnection,
        key: &secretbox::Key,
    ) -> Result<EditForm, result::Error> {
        allmima::table
            .filter(allmima::id.eq(id))
            .get_result::<MimaItem>(conn)
            .map(|item| item.to_edit_delete(key))
    }

    /// 通过 id 把一条记录标记为已删除
    pub fn mark_as_deleted(id: &str, conn: &PgConnection) {
        let target = allmima::table.filter(allmima::id.eq(id));
        diesel::update(target)
            .set(allmima::deleted.eq(now_string()))
            .execute(conn)
            .unwrap(); // BUG
    }

    /// 通过 id 彻底删除一条记录 (不可恢复)
    pub fn delete_forever(id: &str, conn: &PgConnection) {
        let target = allmima::table.filter(allmima::id.eq(id));
        diesel::delete(target).execute(conn).unwrap(); // BUG
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

    /// 对 MimaItem 里的 password 或 notes 进行加密
    fn encrypt(plaintext: &str, key: &secretbox::Key) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
        if plaintext.is_empty() {
            (None, None)
        } else {
            let nonce = secretbox::gen_nonce();
            let encrypted = secretbox::seal(plaintext.as_bytes(), &nonce, key);
            (Some(encrypted), Some(nonce.to_vec()))
        }
    }
}

/// 为了方便对 secretbox::Nonce 进行类型转换
pub trait NonceToVec {
    fn to_vec(self) -> Vec<u8>;
}

impl NonceToVec for secretbox::Nonce {
    /// 类型转换
    fn to_vec(self) -> Vec<u8> {
        self.0.to_vec()
    }
}

/// 给 secretbox::Key 增加几个方法, 方便使用.
pub trait MySecretKey {
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
pub fn now_string() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// simple格式的uuid
pub fn uuid_simple() -> String {
    Uuid::new_v4().to_simple().to_string()
}
