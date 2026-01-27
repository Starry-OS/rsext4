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

#![no_std]

extern crate alloc;

// 导出配置常量，供其他模块使用
pub use config::{
    BITMAP_CACHE_MAX, BLOCK_SIZE, BLOCK_SIZE_U32, DATABLOCK_CACHE_MAX, DEFAULT_FEATURE_COMPAT,
    DEFAULT_FEATURE_INCOMPAT, DEFAULT_FEATURE_RO_COMPAT, DEFAULT_INODE_SIZE, DIRNAME_LEN,
    EXT4_MAJOR_VERSION, EXT4_MINOR_VERSION, EXT4_SUPER_MAGIC, GROUP_DESC_SIZE, GROUP_DESC_SIZE_OLD,
    INODE_CACHE_MAX, JBD2_BUFFER_MAX, LOG_BLOCK_SIZE, RESERVED_GDT_BLOCKS, RESERVED_INODES,
    SUPERBLOCK_OFFSET, SUPERBLOCK_SIZE,
};

// 重新导出核心模块
pub use error::{Ext4Result, RSEXT4Error};

// 重新导出常用类型和函数
pub use api::{lseek, open, read_at, write_at};
pub use blockdev::{BlockDevice, Jbd2Dev};
pub use dir::mkdir;
pub use ext4::{Ext4FileSystem, find_file, mkfs, mount, umount};
pub use file::{
    create_symbol_link, delete_dir, delete_file, link, mkfile, mv, read_file, rename, truncate,
    unlink, write_file,
};

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
pub mod error;
pub mod ext4;
pub mod extents_tree;
pub mod file;
pub mod hashtree;
pub mod inodetable_cache;
pub mod jbd2;
pub mod loopfile;
pub mod superblock;
pub mod tool;
