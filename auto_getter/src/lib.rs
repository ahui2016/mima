use mima_forms::EditForm;
use sodiumoxide::crypto::secretbox;

/// 用于获取解密后的数据, MimaItem 与 HistoryItem 的通用部分.
pub trait AutoGetter {
    fn password_and_nonce(&self) -> (Option<& Vec<u8>>, Option<& Vec<u8>>);
    fn notes_and_nonce(&self) -> (Option<& Vec<u8>>, Option<& Vec<u8>>);
    fn pwd_decrypt(&self, key: &secretbox::Key) -> String;
    fn notes_decrypt(&self, key: &secretbox::Key) -> String;
    fn to_edit_form(&self, key: &secretbox::Key) -> EditForm;
}

/// 用于解密, MimaItem 与 HistoryItem 的通用部分.
pub trait Decryptable {
    /// 对 password 或 notes 进行解密, 返回字符串.
    ///
    /// 如果被解密参数为 None, 则返回空字符串.
    fn decrypt(
        encrypted: Option<&Vec<u8>>,
        nonce: Option<&Vec<u8>>,
        key: &secretbox::Key,
    ) -> String {
        match encrypted {
            Some(vec) => {
                let nonce = nonce.to_nonce();
                let decrypted = secretbox::open(vec, &nonce, key).unwrap();
                String::from_utf8(decrypted).unwrap()
            }
            None => String::new(),
        }
    }
}

/// 为了方便把 Option<&Vec<u8>> 转换为 secretbox::Nonce
pub trait VecToNonce {
    fn to_nonce(self) -> secretbox::Nonce;
}

impl VecToNonce for Option<&Vec<u8>> {
    /// 类型转换
    fn to_nonce(self) -> secretbox::Nonce {
        secretbox::Nonce::from_slice(self.unwrap()).unwrap()
    }
}
