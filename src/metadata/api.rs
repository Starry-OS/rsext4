//! Public path-based metadata APIs.

use crate::blockdev::{BlockDevice, Jbd2Dev};
use crate::dir::{get_inode_with_num, split_paren_child_and_tranlatevalid};
use crate::disknode::Ext4TimeSpec;
use crate::error::{Ext4Error, Ext4Result};
use crate::ext4::Ext4FileSystem;

use super::Ext4InodeMetadataUpdate;

/// Updates the permission bits of the inode referenced by `path`.
pub fn chmod<B: BlockDevice>(
    device: &mut Jbd2Dev<B>,
    fs: &mut Ext4FileSystem,
    path: &str,
    mode: u16,
) -> Ext4Result<()> {
    let path = split_paren_child_and_tranlatevalid(path);
    let (inode_num, _) = get_inode_with_num(fs, device, &path)?.ok_or(Ext4Error::invalid_input())?;
    let _ = fs.apply_inode_metadata(device, inode_num, Ext4InodeMetadataUpdate::chmod(mode))?;
    Ok(())
}

/// Updates the owner and group of the inode referenced by `path`.
pub fn chown<B: BlockDevice>(
    device: &mut Jbd2Dev<B>,
    fs: &mut Ext4FileSystem,
    path: &str,
    uid: Option<u32>,
    gid: Option<u32>,
) -> Ext4Result<()> {
    let path = split_paren_child_and_tranlatevalid(path);
    let (inode_num, _) = get_inode_with_num(fs, device, &path)?.ok_or(Ext4Error::invalid_input())?;
    let _ = fs.apply_inode_metadata(device, inode_num, Ext4InodeMetadataUpdate::chown(uid, gid))?;
    Ok(())
}

/// Updates the access and modification times of the inode referenced by `path`.
pub fn utimens<B: BlockDevice>(
    device: &mut Jbd2Dev<B>,
    fs: &mut Ext4FileSystem,
    path: &str,
    atime: Ext4TimeSpec,
    mtime: Ext4TimeSpec,
) -> Ext4Result<()> {
    let path = split_paren_child_and_tranlatevalid(path);
    let (inode_num, _) = get_inode_with_num(fs, device, &path)?.ok_or(Ext4Error::invalid_input())?;
    let _ = fs.apply_inode_metadata(
        device,
        inode_num,
        Ext4InodeMetadataUpdate::utimens(atime, mtime),
    )?;
    Ok(())
}

/// Updates the project identifier of the inode referenced by `path`.
pub fn set_project<B: BlockDevice>(
    device: &mut Jbd2Dev<B>,
    fs: &mut Ext4FileSystem,
    path: &str,
    projid: u32,
) -> Ext4Result<()> {
    let path = split_paren_child_and_tranlatevalid(path);
    let (inode_num, _) = get_inode_with_num(fs, device, &path)?.ok_or(Ext4Error::invalid_input())?;
    let _ = fs.apply_inode_project(device, inode_num, projid)?;
    Ok(())
}

/// Updates the user-visible inode flags of the inode referenced by `path`.
pub fn set_flags<B: BlockDevice>(
    device: &mut Jbd2Dev<B>,
    fs: &mut Ext4FileSystem,
    path: &str,
    flags: u32,
) -> Ext4Result<()> {
    let path = split_paren_child_and_tranlatevalid(path);
    let (inode_num, _) = get_inode_with_num(fs, device, &path)?.ok_or(Ext4Error::invalid_input())?;
    let _ = fs.apply_inode_flags(device, inode_num, flags)?;
    Ok(())
}
