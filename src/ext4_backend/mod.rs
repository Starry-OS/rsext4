//! # ext4_backend
//!
//! ext4 文件系统的核心实现模块，提供对 ext4 文件系统的底层操作支持。
//! 
//! 该模块包含文件系统的主要组件：
//! - 文件系统挂载和卸载（api, ext4）
//! - 块设备管理和缓存（blockdev, loopfile）
//! - 块组管理和位图操作（blockgroup_description, bitmap, bitmap_cache）
//! - 文件和目录操作（file, dir, entries）
//! - 数据结构管理（superblock, inodetable_cache, datablock_cache）
//! - 辅助工具和配置（tool, config, endian）
//! - 日志系统（jbd2）

pub mod api;
pub mod bitmap;
pub mod bitmap_cache;
pub mod blockdev;
pub mod blockgroup_description;
pub mod bmalloc;
pub mod config;
pub mod datablock_cache;
pub mod dir;
pub mod disknode;
pub mod endian;
pub mod entries;
pub mod ext4;
pub mod extents_tree;
pub mod file;
pub mod hashtree;
pub mod error;
pub mod inodetable_cache;
pub mod jbd2;
pub mod loopfile;
pub mod superblock;
pub mod tool;
