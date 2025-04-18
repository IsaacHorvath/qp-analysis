use diesel::sql_types::{BigInt, Integer, Mediumtext, Nullable, Varchar};

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
        province -> Integer,
        class -> Integer,
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
    province (id) {
        id -> Integer,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 6]
        colour -> Varchar,
        total_words -> Integer,
    }
}

diesel::table! {
    class (id) {
        id -> Integer,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 6]
        colour -> Varchar,
        total_words -> Integer,
    }
}

diesel::table! {
    riding (id) {
        id -> Integer,
        #[max_length = 100]
        name -> Varchar,
        population -> Integer,
        electors -> Integer,
        area -> Double,
    }
}

diesel::table! {
    speech (id) {
        id -> Integer,
        speaker -> Integer,
        transcript -> Integer,
        text -> Mediumtext,
        start -> Datetime,
        end -> Datetime,
    }
}

diesel::table! {
    speech_clean (speech) {
        speech -> Integer,
        text -> Mediumtext,
    }
}

diesel::table! {
    transcript (id) {
        id -> Integer,
        #[max_length = 500]
        link -> Varchar,
    }
}

diesel::define_sql_function!(fn last_insert_id() -> Integer);
diesel::define_sql_function!(fn concat(x: Varchar, y: Varchar, z: Varchar) -> Varchar);
diesel::define_sql_function!(fn count_words(x: Mediumtext, y: Varchar) -> Integer);
diesel::define_sql_function!(fn score(x: Integer, y: Nullable<BigInt>) -> Nullable<Double>);

diesel::joinable!(speech -> speaker (speaker));
diesel::joinable!(speech -> speech_clean (id));
diesel::joinable!(speech -> transcript (transcript));
diesel::joinable!(speaker -> party (party));
diesel::joinable!(speaker -> gender (gender));
diesel::joinable!(speaker -> province (province));
diesel::joinable!(speaker -> class (class));
diesel::joinable!(speaker -> riding (riding));

diesel::allow_tables_to_appear_in_same_query!(
    speech,
    speech_clean,
    speaker,
    party,
    gender,
    province,
    class,
    riding,
    transcript,
);

diesel::allow_columns_to_appear_in_same_group_by_clause!(
    speaker::id,
    speaker::first_name,
    speaker::last_name,
    speaker::total_words,
    party::colour,
    riding::name,
    riding::area,
    riding::population,
);
