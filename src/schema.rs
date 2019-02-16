table! {
    allmima (id) {
        id -> Uuid,
        title -> Varchar,
        username -> Varchar,
        password -> Nullable<Bytea>,
        notes -> Nullable<Bytea>,
        favorite -> Bool,
        created -> Timestamp,
        deleted -> Timestamp,
    }
}

table! {
    history (id) {
        id -> Uuid,
        mima_id -> Uuid,
        title -> Varchar,
        username -> Varchar,
        password -> Nullable<Bytea>,
        notes -> Nullable<Bytea>,
    }
}

joinable!(history -> allmima (mima_id));

allow_tables_to_appear_in_same_query!(
    allmima,
    history,
);
