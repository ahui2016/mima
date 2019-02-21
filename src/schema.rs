table! {
    allmima (id) {
        id -> Varchar,
        title -> Varchar,
        username -> Varchar,
        password -> Nullable<Bytea>,
        notes -> Nullable<Bytea>,
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
        notes -> Nullable<Bytea>,
    }
}

joinable!(history -> allmima (mima_id));

allow_tables_to_appear_in_same_query!(allmima, history,);
