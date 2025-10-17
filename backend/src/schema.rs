// @generated automatically by Diesel CLI.

diesel::table! {
    settings (key) {
        key -> Text,
        value -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        #[max_length = 97]
        password -> Bpchar,
        first_name -> Text,
        affix -> Nullable<Text>,
        last_name -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    settings,
    users,
);
