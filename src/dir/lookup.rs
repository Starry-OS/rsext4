//! Directory path lookup helpers.

use crate::blockdev::*;
use crate::bmalloc::InodeNumber;
use crate::disknode::*;
use crate::error::*;
use crate::ext4::Ext4FileSystem;
use crate::loopfile::get_file_inode;

/// Resolves a path to its inode number and inode contents.
pub fn get_inode_with_num<B: BlockDevice>(
    fs: &mut Ext4FileSystem,
    device: &mut Jbd2Dev<B>,
    path: &str,
) -> Ext4Result<Option<(InodeNumber, Ext4Inode)>> {
    get_file_inode(fs, device, path)
}
