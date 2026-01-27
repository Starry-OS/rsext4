//! # rsext4
//!
//! 一个用 Rust 实现的 ext4 文件系统库，提供对 ext4 文件系统的读写操作支持。
//!
//! ## 功能特点
//!
//! - 支持 ext4 文件系统的挂载和卸载
//! - 提供文件和目录的基本操作
//! - 支持块设备的抽象和操作
//! - 提供完整的错误处理机制

#![no_std]
#![deny(unused)]
#![deny(dead_code)]
#![deny(warnings)]

extern crate alloc;
pub mod ext4_backend;
pub use ext4_backend::api::*;
pub use ext4_backend::blockdev::*;
pub use ext4_backend::config::*;
pub use ext4_backend::dir::*;
pub use ext4_backend::error::*;
pub use ext4_backend::ext4::*;
pub use ext4_backend::file::*;
