use diesel::sql_types::{Text, Varchar};

diesel::table! {
    speaker (id) {
        id -> Integer,
        #[max_length = 100]
        first_name -> Varchar,
        #[max_length = 100]
        last_name -> Varchar,
        party -> Integer,
        age -> Integer,
        gender -> Integer,
        riding -> Integer,
        elected -> Integer,
        total_words -> Integer,
    }
}

diesel::table! {
    party (id) {
        id -> Integer,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 6]
        colour -> Varchar,
        total_words -> Integer,
    }
}

diesel::table! {
    gender (id) {
        id -> Integer,
        #[max_length = 100]
        name -> Varchar,
        colour -> Varchar,
        total_words -> Integer,
    }
}

diesel::table! {
    speech (id) {
        id -> Integer,
        speaker -> Integer,
        text -> Text,
        start -> Datetime,
        end -> Datetime,
    }
}

diesel::table! {
    speech_clean (id) {
        id -> Integer,
        speaker -> Integer,
        text -> Text,
        start -> Datetime,
        end -> Datetime,
    }
}

diesel::table! {
    word (id) {
        id -> Integer,
        #[max_length = 100]
        text -> Varchar,
    }
}

diesel::table! {
    word_count (speech, word) {
        speech -> Integer,
        word -> Integer,
        count -> Integer,
    }
}

diesel::define_sql_function!(fn count_words(x: Text, y: Varchar) -> Integer);
diesel::define_sql_function!(fn concat(x: Varchar, y: Varchar, z: Varchar) -> Varchar);

diesel::joinable!(speech_clean -> speaker (speaker));
diesel::joinable!(speaker -> party (party));
diesel::joinable!(speaker -> gender (gender));

diesel::allow_tables_to_appear_in_same_query!(
    speech_clean,
    speaker,
    party,
    gender,
);

diesel::allow_columns_to_appear_in_same_group_by_clause!(
    speaker::first_name,
    speaker::last_name,
    speaker::total_words,
    party::colour,
);
