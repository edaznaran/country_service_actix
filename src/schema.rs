// @generated automatically by Diesel CLI.

diesel::table! {
    audit_log (id) {
        id -> Int4,
        action -> Varchar,
        details -> Jsonb,
        createdAt -> Timestamp,
    }
}

diesel::table! {
    country (id) {
        id -> Int4,
        name -> Varchar,
        code -> Varchar,
        dial_code -> Varchar,
        createdAt -> Timestamp,
        updateAt -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(audit_log, country,);
