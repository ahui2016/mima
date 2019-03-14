#![recursion_limit="128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

/// 用于获取解密后的数据, MimaItem 与 HistoryItem 的通用部分.
#[proc_macro_derive(AutoGetter)]
pub fn auto_getter_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_auto_getter(&ast)
}

fn impl_auto_getter(ast: &syn::DeriveInput) -> TokenStream {
    let item = &ast.ident;
    let gen = quote! {
        impl Decryptable for #item {}

        impl AutoGetter for #item {
            fn password_and_nonce(&self) -> (Option<& Vec<u8>>, Option<& Vec<u8>>) {
                (self.password.as_ref(), self.p_nonce.as_ref())
            }
            fn notes_and_nonce(&self) -> (Option<& Vec<u8>>, Option<& Vec<u8>>) {
                (self.notes.as_ref(), self.n_nonce.as_ref())
            }
            /// 获取解密后的 password
            fn pwd_decrypt(&self, key: &secretbox::Key) -> String {
                let (pwd, nonce) = self.password_and_nonce();
                Self::decrypt(pwd, nonce, key)
            }
            /// 获取解密后的 notes
            fn notes_decrypt(&self, key: &secretbox::Key) -> String {
                let (notes, nonce) = self.notes_and_nonce();
                Self::decrypt(notes, nonce, key)
            }
            /// 返回适用于展示的内容, 已解密.
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
        }
    };
    gen.into()
}
