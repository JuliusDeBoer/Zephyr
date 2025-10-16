// @generated automatically by Diesel CLI.

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
