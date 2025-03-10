//!
//! [<img alt="github" src="https://img.shields.io/badge/github-workflow--rs-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github" height="20">](https://github.com/workflow-rs/workflow-rs)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/workflow-core.svg?maxAge=2592000&style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/workflow-core)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-workflow--core-56c2a5?maxAge=2592000&style=for-the-badge&logo=docs.rs" height="20">](https://docs.rs/workflow-core)
//! <img alt="license" src="https://img.shields.io/crates/l/workflow-core.svg?maxAge=2592000&color=6ac&style=for-the-badge&logoColor=fff" height="20">
//! <img src="https://img.shields.io/badge/platform- native-informational?style=for-the-badge&color=50a0f0" height="20">
//! <img src="https://img.shields.io/badge/platform- wasm32/browser -informational?style=for-the-badge&color=50a0f0" height="20">
//! <img src="https://img.shields.io/badge/platform- wasm32/node.js -informational?style=for-the-badge&color=50a0f0" height="20">
//! <img src="https://img.shields.io/badge/platform- solana_os/ignored-informational?style=for-the-badge&color=777787" height="20">
//!
//! [`workflow_core`] is a part of the [`workflow-rs`](https://crates.io/workflow-rs)
//! framework, subset of which is designed to function uniformally across multiple
//! environments including native Rust, WASM-browser and Solana OS targets.
//!
//! This is a general-purpose crate that provides platform-uniform (native and WASM) abstractions for:
//! - async channels
//! - task spawn and sleep functions
//! - random identifiers
//! - async-friendly and threadsafe event triggers
//! - time (Instant and Duration) structs
//! - dynamic async_trait attribute macros
//!

extern crate self as workflow_core;

pub mod abortable;
pub mod enums;
pub mod prelude;
pub mod runtime;
pub mod sendable;
pub mod traits;
pub mod utils;

mod native;
mod wasm;

// pub use workflow_core_macros::describe_enum;
// pub use workflow_core_macros::Describe;
pub use workflow_core_macros::seal;

cfg_if::cfg_if! {
    if #[cfg(not(target_os = "solana"))] {
        // Generic 8-byte identifier
        pub mod id;
        // task re-exports and shims
        pub mod task;
        // channel re-exports and shims
        pub mod channel;
        // async object lookup combinator
        pub mod lookup;

        // time functions and utilities
        pub mod time;

        // environment variable access (native and Node.js abstraction)
        pub mod env;

        // directory access (home folder, data folder) (native and Node.js abstraction)
        pub mod dirs;

        /// trigger re-exports and shims
        pub mod trigger;

        /// re-export of [`mod@cfg_if`] crate
        pub use ::cfg_if::cfg_if;

        /// dynamically configured re-export of async_trait as workflow_async_trait
        /// that imposes `Send` restriction in native (non-WASM) and removes `Send`
        /// restriction in WASM builds.
        #[cfg(target_arch = "wasm32")]
        pub use workflow_async_trait::async_trait_without_send as workflow_async_trait;
        /// dynamically configured re-export of async_trait as workflow_async_trait
        /// that imposes `Send` restriction in native (non-WASM) and removes `Send`
        /// restriction in WASM builds.
        #[cfg(not(target_arch = "wasm32"))]
        pub use workflow_async_trait::async_trait_with_send as workflow_async_trait;

    }
}
