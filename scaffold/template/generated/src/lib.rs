#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/generated_root.rs"));
pub mod ts_exports;
pub mod generated {
    pub use crate::*;
}
