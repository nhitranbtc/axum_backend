// @generated automatically by Diesel CLI.

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        token_hash -> Text,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        revoked_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        password_hash -> Nullable<Text>,
        #[max_length = 20]
        role -> Varchar,
        is_active -> Bool,
        last_login -> Nullable<Timestamptz>,
        #[max_length = 20]
        confirmation_code -> Nullable<Varchar>,
        confirmation_code_expires_at -> Nullable<Timestamptz>,
        email_verified -> Bool,
    }
}

diesel::joinable!(refresh_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(refresh_tokens, users,);
