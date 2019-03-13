use chrono::{SecondsFormat, Utc};
use diesel::result::{self, QueryResult};
use diesel::{self, prelude::*};
use sodiumoxide::crypto::secretbox;
use uuid::Uuid;

use super::forms::{AddForm, EditForm};
use super::schema::{allmima, history};
use super::{EPOCH, FIRST_ID};

use auto_getter::{AutoGetter, Decryptable};
use auto_getter_derive::AutoGetter;

/// 与数据表 `allmima` 的结构一一对应.
#[table_name = "allmima"]
#[derive(Serialize, Insertable, Queryable, Identifiable, AsChangeset, Debug, Clone, AutoGetter)]
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

    /// 用于处理 `edit` 页面的表单.
    pub fn from_edit_form(form: EditForm, conn: &PgConnection, key: &secretbox::Key) -> MimaItem {
        let item = allmima::table
            .filter(allmima::id.eq(&form.id))
            .get_result::<MimaItem>(conn)
            .unwrap();
        let (password, p_nonce) = if item.pwd_decrypt(key) == form.password {
            (item.password.clone(), item.p_nonce.clone())
        } else {
            Self::encrypt(form.password.as_str(), key)
        };
        let (notes, n_nonce) = if item.notes_decrypt(key) == form.notes {
            (item.notes, item.n_nonce)
        } else {
            Self::encrypt(form.notes.trim(), key)
        };
        MimaItem {
            title: form.title.trim().into(),
            username: form.username.trim().into(),
            password,
            p_nonce,
            notes,
            n_nonce,
            ..item
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

    /// 转换为 HistoryItem 以便插入到 history 数据表.
    pub fn into_history_item(self) -> HistoryItem {
        HistoryItem {
            id: uuid_simple(),
            mima_id: self.id,
            title: self.title,
            username: self.username,
            password: self.password,
            p_nonce: self.p_nonce,
            notes: self.notes,
            n_nonce: self.n_nonce,
            deleted: now_string(),
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
    fn to_edit_form(&self, key: &secretbox::Key) -> EditForm {
        EditForm {
            id: self.id.clone(),
            title: self.title.clone(),
            username: self.username.clone(),
            password: self.pwd_decrypt(key),
            notes: self.notes_decrypt(key),
            favorite: self.favorite,
        }
    }

    /// 向数据库中插入一条新项目.
    pub fn insert(&self, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(allmima::table)
            .values(self)
            .execute(conn)
    }

    /// 更新数据
    pub fn update(&self, conn: &PgConnection) -> QueryResult<usize> {
        diesel::update(self).set(self).execute(conn)
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
            .map(|item| item.to_edit_form(key))
            .collect()
    }

    /// 通过 id 获取一条记录 (已解密)
    pub fn get_by_id(
        id: &str,
        conn: &PgConnection,
        key: &secretbox::Key,
    ) -> Result<EditForm, result::Error> {
        allmima::table
            .filter(allmima::id.eq(id))
            .get_result::<MimaItem>(conn)
            .map(|item| item.to_edit_form(key))
    }

    /// 根据 id 把一条记录标记为已删除
    pub fn mark_as_deleted(id: &str, conn: &PgConnection) -> QueryResult<usize> {
        let target = allmima::table.filter(allmima::id.eq(id));
        diesel::update(target)
            .set(allmima::deleted.eq(now_string()))
            .execute(conn)
    }

    /// 把一条已标记为删除的记录改为未删除, 同时修改其 title 和创建日期.
    ///
    /// 修改 title 是为了避免 title 重复冲突.
    /// 修改创建日期是为了排序 (由于没有修改日期).
    pub fn recover(id: &str, conn: &PgConnection) -> QueryResult<usize> {
        let title = allmima::table
            .filter(allmima::id.eq(id))
            .select(allmima::title)
            .get_result::<String>(conn)
            .unwrap();
        let title = format!("{} ({})", title, now_string());
        let target = allmima::table.filter(allmima::id.eq(id));
        diesel::update(target)
            .set((
                allmima::deleted.eq(EPOCH),
                allmima::title.eq(title),
                allmima::created.eq(now_string()),
            ))
            .execute(conn)
    }

    /// 通过 id 彻底删除一条记录 (不可恢复)
    pub fn delete_forever(id: &str, conn: &PgConnection) -> QueryResult<usize> {
        let target = allmima::table.filter(allmima::id.eq(id));
        diesel::delete(target).execute(conn)
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

/// 与数据表 `history` 的结构一一对应
#[table_name = "history"]
#[derive(Serialize, Insertable, Queryable, Identifiable, Debug, Clone, AutoGetter)]
pub struct HistoryItem {
    pub id: String,
    pub mima_id: String,
    pub title: String,
    pub username: String,
    pub password: Option<Vec<u8>>,
    pub p_nonce: Option<Vec<u8>>,
    pub notes: Option<Vec<u8>>,
    pub n_nonce: Option<Vec<u8>>,
    pub deleted: String,
}

impl HistoryItem {
    /// 通过 id 获取一条历史记录 (已解密)
    pub fn get_by_id(
        id: &str,
        conn: &PgConnection,
        key: &secretbox::Key,
    ) -> Result<EditForm, result::Error> {
        history::table
            .filter(history::id.eq(id))
            .get_result::<HistoryItem>(conn)
            .map(|item| item.to_edit_form(key))
    }

    /// 通过 history_id 查找 mima_id.
    pub fn get_mima_id(history_id: &str, conn: &PgConnection) -> String {
        history::table
            .filter(history::id.eq(history_id))
            .select(history::mima_id)
            .get_result::<String>(conn)
            .unwrap()
    }

    /// 通过 mima_id 获取相关的全部记录 (并且解密).
    pub fn get_by_mima_id(id: &str, conn: &PgConnection, key: &secretbox::Key) -> Vec<EditForm> {
        history::table
            .filter(history::mima_id.eq(id))
            .order(history::deleted.desc())
            .load::<HistoryItem>(conn)
            .unwrap()
            .iter()
            .map(|item| item.to_edit_form(key))
            .collect()
    }

    /// 返回适用于展示的内容, 已解密.
    fn to_edit_form(&self, key: &secretbox::Key) -> EditForm {
        EditForm {
            id: self.id.clone(),
            title: self.title.clone(),
            username: self.username.clone(),
            password: self.pwd_decrypt(key),
            notes: self.notes_decrypt(key),
            favorite: false,
        }
    }

    /// 向插入一条新的 history.
    pub fn insert(&self, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(history::table)
            .values(self)
            .execute(conn)
    }

    /// 通过 id 彻底删除一条历史记录 (不可恢复)
    pub fn delete_forever(id: &str, conn: &PgConnection) -> QueryResult<usize> {
        let target = history::table.filter(history::id.eq(id));
        diesel::delete(target).execute(conn)
    }
}
