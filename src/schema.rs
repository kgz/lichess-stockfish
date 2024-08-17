// @generated automatically by Diesel CLI.

diesel::table! {
    message (id) {
        id -> Integer,
        message_id -> Text,
        lc_channel -> Text,
        created_at -> Timestamp,
    }
}
