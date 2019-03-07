table! {
    allmima (id) {
        id -> Varchar,
        title -> Varchar,
        username -> Varchar,
        password -> Nullable<Bytea>,
        p_nonce -> Nullable<Bytea>,
        notes -> Nullable<Bytea>,
        n_nonce -> Nullable<Bytea>,
        favorite -> Bool,
        created -> Varchar,
        deleted -> Varchar,
    }
}

table! {
    history (id) {
        id -> Varchar,
        mima_id -> Varchar,
        title -> Varchar,
        username -> Varchar,
        password -> Nullable<Bytea>,
        p_nonce -> Nullable<Bytea>,
        notes -> Nullable<Bytea>,
        n_nonce -> Nullable<Bytea>,
        deleted -> Varchar,
    }
}

joinable!(history -> allmima (mima_id));

allow_tables_to_appear_in_same_query!(allmima, history,);
