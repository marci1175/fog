// @generated automatically by Diesel CLI.

diesel::table! {
    dependencies (dependency_name) {
        dependency_name -> Text,
        dependency_source_path -> Text,
        dependency_version -> Text,
        author -> Text,
        date_added -> Date,
        secret -> Text,
    }
}
