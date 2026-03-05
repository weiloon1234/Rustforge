pub mod config;
pub mod gen_auth;
pub mod gen_datatables;
pub mod gen_enums;
pub mod gen_localized;
pub mod gen_models;
pub mod gen_permissions;
pub mod permissions;
pub mod schema;

pub use config::ConfigsFile;
pub use gen_auth::generate_auth;
pub use gen_datatables::generate_datatable_skeletons;
pub use gen_enums::{
    generate_enums, generate_enums_with_options, generate_enum_with_options, GenerateEnumsOptions,
};
pub use gen_localized::generate_localized;
pub use gen_models::{generate_models, generate_models_with_options, GenerateModelsOptions};
pub use gen_permissions::generate_permissions;
pub use permissions::load_permissions;
pub use schema::{load_framework, load_with_framework, Schema};
