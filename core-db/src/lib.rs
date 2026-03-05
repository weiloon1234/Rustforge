extern crate self as core_db;

pub mod commands;
pub mod common;
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/framework_generated.rs"));
}
pub mod infra;
pub mod platform;
pub mod seeder;
pub mod ts_exports;
