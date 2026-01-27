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
//! 
//! ## 基本使用
//! 
//! ```rust
//! use rsext4::*;
//! 
//! // 挂载文件系统
//! let mut fs = fs_mount(&mut device)?;
//! 
//! // 打开文件
//! let mut file = open(&mut fs, &mut device, "/path/to/file")?;
//! 
//! // 读取数据
//! let mut buffer = vec![0u8; 1024];
//! read(&mut fs, &mut device, &mut file, &mut buffer)?;
//! 
//! // 卸载文件系统
//! fs_umount(fs, &mut device)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

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
pub use ext4_backend::ext4::*;
pub use ext4_backend::file::*;
pub use ext4_backend::error::*;
