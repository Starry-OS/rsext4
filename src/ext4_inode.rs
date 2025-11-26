//! EXT4 inode 操作
//! 参考 `lwext4/src/ext4_inode.c`

#![allow(dead_code)]

use core::mem::offset_of;

use crate::{
    ext4_errno::{EINVAL, EOK},
    ext4_get16, ext4_get32,
    ext4_misc::{to_le16, to_le32},
    ext4_super::{ext4_sb_feature_ro_com, ext4_sb_get_block_size},
    ext4_types::{
        EXT4_FRO_COM_HUGE_FILE, EXT4_GOOD_OLD_INODE_SIZE, EXT4_INODE_FLAG_APPEND, EXT4_INODE_FLAG_HUGE_FILE, EXT4_INODE_FLAG_IMMUTABLE, EXT4_INODE_INDIRECT_BLOCK, EXT4_INODE_MODE_DIRECTORY, EXT4_INODE_MODE_FILE, EXT4_INODE_MODE_SOFTLINK, EXT4_INODE_MODE_TYPE_MASK, EXT4_SUPERBLOCK_OS_HURD, EXT4_SUPERBLOCK_OS_LINUX, ext4_extent_header, ext4_inode, ext4_sblock
    }, offsetof,
};

// ============================================================================
// 内部辅助函数
// ============================================================================

/// 计算 inode 块计数字段所需的位数。
///
/// # 参数
/// - `block_size`: 文件系统块大小（字节）。
///
/// # 返回
/// - 需要用于块计数的位数。
#[inline]
fn ext4_inode_block_bits_count(block_size: u32) -> u32 {
    let mut bits: u32 = 8;
    let mut size = block_size;

    while size > 256 {
        bits += 1;
        size >>= 1;
    }

    bits
}

// ============================================================================
// 基础元数据访问
// ============================================================================

/// 获取 inode 的 `mode` 值（必要时包含 Hurd 扩展位）。
pub fn ext4_inode_get_mode(sb: &ext4_sblock, inode: &ext4_inode) -> u32 {
    let mut v = to_le16(inode.mode) as u32;

    if ext4_get32!(sb, creator_os) == EXT4_SUPERBLOCK_OS_HURD {
        v |= (to_le16(unsafe { inode.osd2.hurd2.mode_high }) as u32) << 16;
    }

    v
}

/// 设置 inode 的 `mode` 值（必要时更新 Hurd 扩展位）。
pub fn ext4_inode_set_mode(sb: &ext4_sblock, inode: &mut ext4_inode, mode: u32) {
    inode.mode = to_le16(((mode << 16) >> 16) as u16);

    if ext4_get32!(sb, creator_os) == EXT4_SUPERBLOCK_OS_HURD {
        unsafe {
            inode.osd2.hurd2.mode_high = to_le16((mode >> 16) as u16);
        }
    }
}

/// 获取 inode 中存储的用户 ID（低 16 位 + 扩展位）。
pub fn ext4_inode_get_uid(inode: &ext4_inode) -> u32 {
    to_le32(inode.uid as u32)
}

/// 设置 inode 中的用户 ID。
pub fn ext4_inode_set_uid(inode: &mut ext4_inode, uid: u32) {
    inode.uid = to_le16((uid & 0xFFFF) as u16);
}

/// 获取 inode 记录的逻辑文件大小（组合高低位字段）。
pub fn ext4_inode_get_size(sb: &ext4_sblock, inode: &ext4_inode) -> u64 {
    let mut v = to_le32(inode.size_lo) as u64;

    if (ext4_get32!(sb, rev_level) > 0)
        && ext4_inode_is_type(sb, inode, EXT4_INODE_MODE_FILE as u32)
    {
        v |= (to_le32(inode.size_hi) as u64) << 32;
    }

    v
}

/// 设置 inode 记录的逻辑文件大小（拆分到高低位）。
pub fn ext4_inode_set_size(inode: &mut ext4_inode, size: u64) {
    inode.size_lo = to_le32(((size << 32) >> 32) as u32);
    inode.size_hi = to_le32((size >> 32) as u32);
}

/// 获取 inode 的 CRC32 校验值（支持 large inode）。
pub fn ext4_inode_get_csum(sb: &ext4_sblock, inode: &ext4_inode) -> u32 {
    let inode_size = ext4_get16!(sb, inode_size);
    let mut v = to_le16(unsafe { inode.osd2.linux2.checksum_lo }) as u32;

    if inode_size > EXT4_GOOD_OLD_INODE_SIZE as u16 {
        v |= (to_le16(inode.checksum_hi) as u32) << 16;
    }

    v
}

/// 设置 inode 的 CRC32 校验值。
pub fn ext4_inode_set_csum(sb: &ext4_sblock, inode: &mut ext4_inode, checksum: u32) {
    let inode_size = ext4_get16!(sb, inode_size);
    unsafe {
        inode.osd2.linux2.checksum_lo = to_le16(((checksum << 16) >> 16) as u16);
    }

    if inode_size > EXT4_GOOD_OLD_INODE_SIZE as u16 {
        inode.checksum_hi = to_le16((checksum >> 16) as u16);
    }
}

/// 获取 inode 的访问时间（atime）。
pub fn ext4_inode_get_access_time(inode: &ext4_inode) -> u32 {
    to_le32(inode.access_time)
}

/// 设置 inode 的访问时间（atime）。
pub fn ext4_inode_set_access_time(inode: &mut ext4_inode, time: u32) {
    inode.access_time = to_le32(time);
}

/// 获取 inode 的状态变更时间（ctime）。
pub fn ext4_inode_get_change_inode_time(inode: &ext4_inode) -> u32 {
    to_le32(inode.change_inode_time)
}

/// 设置 inode 的状态变更时间（ctime）。
pub fn ext4_inode_set_change_inode_time(inode: &mut ext4_inode, time: u32) {
    inode.change_inode_time = to_le32(time);
}

/// 获取 inode 的修改时间（mtime）。
pub fn ext4_inode_get_modif_time(inode: &ext4_inode) -> u32 {
    to_le32(inode.modification_time)
}

/// 设置 inode 的修改时间（mtime）。
pub fn ext4_inode_set_modif_time(inode: &mut ext4_inode, time: u32) {
    inode.modification_time = to_le32(time);
}

/// 获取 inode 的删除时间（dtime）。
pub fn ext4_inode_get_del_time(inode: &ext4_inode) -> u32 {
    to_le32(inode.deletion_time)
}

/// 设置 inode 的删除时间（dtime）。
pub fn ext4_inode_set_del_time(inode: &mut ext4_inode, time: u32) {
    inode.deletion_time = to_le32(time);
}

/// 获取 inode 中存储的组 ID（低 16 位 + 扩展位）。
pub fn ext4_inode_get_gid(inode: &ext4_inode) -> u32 {
    to_le32(inode.gid as u32)
}

/// 设置 inode 中的组 ID。
pub fn ext4_inode_set_gid(inode: &mut ext4_inode, gid: u32) {
    inode.gid = to_le16((gid & 0xFFFF) as u16);
}

/// 获取 inode 的硬链接数量。
pub fn ext4_inode_get_links_cnt(inode: &ext4_inode) -> u16 {
    to_le16(inode.links_count)
}

/// 设置 inode 的硬链接数量。
pub fn ext4_inode_set_links_cnt(inode: &mut ext4_inode, cnt: u16) {
    inode.links_count = to_le16(cnt);
}

/// 获取 inode 分配的磁盘块总数（支持 huge/64 位）。
pub fn ext4_inode_get_blocks_count(sb: &ext4_sblock, inode: &ext4_inode) -> u64 {
    let mut cnt = to_le32(inode.blocks_count_lo) as u64;

    if ext4_sb_feature_ro_com(sb, EXT4_FRO_COM_HUGE_FILE) {
        // 48-bit field
        cnt |= (to_le16(unsafe { inode.osd2.linux2.blocks_high }) as u64) << 32;

        if ext4_inode_has_flag(inode, EXT4_INODE_FLAG_HUGE_FILE) {
            let block_count = ext4_sb_get_block_size(sb);
            let b = ext4_inode_block_bits_count(block_count);
            return cnt << (b - 9);
        }
    }

    cnt
}

/// 设置 inode 分配的磁盘块总数；返回 errno。
pub fn ext4_inode_set_blocks_count(
    sb: &ext4_sblock,
    inode: &mut ext4_inode,
    count: u64,
) -> i32 {
    // 32-bit maximum
    let max_32: u64 = 0xFFFFFFFF;

    if count <= max_32 {
        inode.blocks_count_lo = to_le32(count as u32);
        unsafe {
            inode.osd2.linux2.blocks_high = 0;
        }
        ext4_inode_clear_flag(inode, EXT4_INODE_FLAG_HUGE_FILE);
        return EOK;
    }

    // Check if there can be used huge files (many blocks)
    if !ext4_sb_feature_ro_com(sb, EXT4_FRO_COM_HUGE_FILE) {
        return EINVAL;
    }

    // 48-bit maximum
    let max_48: u64 = 0xFFFFFFFFFFFF;

    if count <= max_48 {
        inode.blocks_count_lo = to_le32(count as u32);
        unsafe {
            inode.osd2.linux2.blocks_high = to_le16((count >> 32) as u16);
        }
        ext4_inode_clear_flag(inode, EXT4_INODE_FLAG_HUGE_FILE);
    } else {
        let block_count = ext4_sb_get_block_size(sb);
        let block_bits = ext4_inode_block_bits_count(block_count);

        ext4_inode_set_flag(inode, EXT4_INODE_FLAG_HUGE_FILE);
        let shifted_count = count >> (block_bits - 9);
        inode.blocks_count_lo = to_le32(shifted_count as u32);
        unsafe {
            inode.osd2.linux2.blocks_high = to_le16((shifted_count >> 32) as u16);
        }
    }

    EOK
}

/// 获取 inode 的标志字段。
pub fn ext4_inode_get_flags(inode: &ext4_inode) -> u32 {
    to_le32(inode.flags)
}

/// 设置 inode 的标志字段。
pub fn ext4_inode_set_flags(inode: &mut ext4_inode, flags: u32) {
    inode.flags = to_le32(flags);
}

/// 获取 inode 的 generation（供 NFS 等使用）。
pub fn ext4_inode_get_generation(inode: &ext4_inode) -> u32 {
    to_le32(inode.generation)
}

/// 设置 inode 的 generation。
pub fn ext4_inode_set_generation(inode: &mut ext4_inode, generation: u32) {
    inode.generation = to_le32(generation);
}

/// 获取 inode 的扩展尺寸（只对 large inode 有效）。
pub fn ext4_inode_get_extra_isize(sb: &ext4_sblock, inode: &ext4_inode) -> u16 {
    let inode_size = ext4_get16!(sb, inode_size);
    if inode_size > EXT4_GOOD_OLD_INODE_SIZE as u16 {
        to_le16(inode.extra_isize)
    } else {
        0
    }
}

/// 设置 inode 的扩展尺寸（仅当 inode size 大于传统尺寸时）。
pub fn ext4_inode_set_extra_isize(sb: &ext4_sblock, inode: &mut ext4_inode, size: u16) {
    let inode_size = ext4_get16!(sb, inode_size);
    if inode_size > EXT4_GOOD_OLD_INODE_SIZE as u16 {
        inode.extra_isize = to_le16(size);
    }
}

/// 获取 inode 中记录的 file ACL 块号。
pub fn ext4_inode_get_file_acl(inode: &ext4_inode, sb: &ext4_sblock) -> u64 {
    let mut v = to_le32(inode.file_acl_lo) as u64;

    if ext4_get32!(sb, creator_os) == EXT4_SUPERBLOCK_OS_LINUX {
        let high = to_le16(unsafe { inode.osd2.linux2.file_acl_high }) as u32;
        v |= (high as u64) << 32;
    }

    v
}

/// 设置 inode 中的 file ACL 块号。
pub fn ext4_inode_set_file_acl(inode: &mut ext4_inode, sb: &ext4_sblock, acl: u64) {
    inode.file_acl_lo = to_le32(((acl << 32) >> 32) as u32);

    if ext4_get32!(sb, creator_os) == EXT4_SUPERBLOCK_OS_LINUX {
        unsafe {
            inode.osd2.linux2.file_acl_high = to_le16(((acl >> 32)) as u16);
        }
    }
}

// ============================================================================
// 数据块指针访问
// ============================================================================

/// 读取第 `idx` 个直接块指针的物理块地址。
pub fn ext4_inode_get_direct_block(inode: &ext4_inode, idx: u32) -> u32 {
    to_le32(inode.blocks[idx as usize])
}

/// 将物理块地址写入第 `idx` 个直接块指针。
pub fn ext4_inode_set_direct_block(inode: &mut ext4_inode, idx: u32, block: u32) {
    inode.blocks[idx as usize] = to_le32(block);
}

/// 读取间接/二级/三级指针的物理块地址。
pub fn ext4_inode_get_indirect_block(inode: &ext4_inode, idx: u32) -> u32 {
    to_le32(inode.blocks[idx as usize + EXT4_INODE_INDIRECT_BLOCK])
}

/// 将物理块地址写入间接/二级/三级指针。
pub fn ext4_inode_set_indirect_block(inode: &mut ext4_inode, idx: u32, block: u32) {
    inode.blocks[idx as usize + EXT4_INODE_INDIRECT_BLOCK] = to_le32(block);
}

/// 获取特殊 inode 中编码的设备号。
pub fn ext4_inode_get_dev(inode: &ext4_inode) -> u32 {
    let dev_0 = ext4_inode_get_direct_block(inode, 0);
    let dev_1 = ext4_inode_get_direct_block(inode, 1);

    if dev_0 != 0 {
        dev_0
    } else {
        dev_1
    }
}

/// 将设备号写入前两个直接块槽位。
pub fn ext4_inode_set_dev(inode: &mut ext4_inode, dev: u32) {
    if dev & !0xFFFF != 0 {
        // 大设备号：使用 slot 1，清除 slot 0
        ext4_inode_set_direct_block(inode, 0, 0);
        ext4_inode_set_direct_block(inode, 1, dev);
    } else {
        // 小设备号：使用 slot 0，清除 slot 1
        ext4_inode_set_direct_block(inode, 0, dev);
        ext4_inode_set_direct_block(inode, 1, 0);
    }
}

// ============================================================================
// 类型、标志与权限辅助
// ============================================================================

/// 返回 inode 的类型掩码（由 mode 推导）。
pub fn ext4_inode_type(sb: &ext4_sblock, inode: &ext4_inode) -> u32 {
    ext4_inode_get_mode(sb, inode) & (EXT4_INODE_MODE_TYPE_MASK as u32)
}

/// 判断 inode 是否为指定类型。
pub fn ext4_inode_is_type(sb: &ext4_sblock, inode: &ext4_inode, ty: u32) -> bool {
    ext4_inode_type(sb, inode) == ty
}

/// 判断 inode 的标志字段是否包含给定标志。
pub fn ext4_inode_has_flag(inode: &ext4_inode, flag: u32) -> bool {
    (ext4_inode_get_flags(inode) & flag) != 0
}

/// 清除 inode 标志字段中的某个标志。
pub fn ext4_inode_clear_flag(inode: &mut ext4_inode, flag: u32) {
    let flags = ext4_inode_get_flags(inode);
    ext4_inode_set_flags(inode, flags & !flag);
}

/// 设置 inode 标志字段中的某个标志。
pub fn ext4_inode_set_flag(inode: &mut ext4_inode, flag: u32) {
    let flags = ext4_inode_get_flags(inode);
    ext4_inode_set_flags(inode, flags | flag);
}

/// 判断 inode 是否允许截断（需排除 append/immutable 等情况）。
pub fn ext4_inode_can_truncate(sb: &ext4_sblock, inode: &ext4_inode) -> bool {
    if ext4_inode_has_flag(inode, EXT4_INODE_FLAG_APPEND)
        || ext4_inode_has_flag(inode, EXT4_INODE_FLAG_IMMUTABLE)
    {
        return false;
    }

    if ext4_inode_is_type(sb, inode, EXT4_INODE_MODE_FILE as u32)
        || ext4_inode_is_type(sb, inode, EXT4_INODE_MODE_DIRECTORY as u32)
        || ext4_inode_is_type(sb, inode, EXT4_INODE_MODE_SOFTLINK as u32)
    {
        return true;
    }

    false
}

// ============================================================================
// Extent 访问
// ============================================================================

/// 获取 inode 中 extent 树根节点的头部指针。
pub fn ext4_inode_get_extent_header(inode: &mut ext4_inode) -> *mut ext4_extent_header {
    unsafe {
        let raw_ptr = inode as *mut ext4_inode as *mut u8;
        let block_ptr = raw_ptr.add(offset_of!(ext4_inode,blocks));
        block_ptr as *mut ext4_extent_header
    }   
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext4_types::{
        ext4_inode, ext4_sblock, EXT4_FRO_COM_HUGE_FILE, EXT4_GOOD_OLD_INODE_SIZE,
        EXT4_INODE_FLAG_APPEND, EXT4_INODE_FLAG_HUGE_FILE, EXT4_INODE_FLAG_IMMUTABLE,
        EXT4_INODE_MODE_DIRECTORY, EXT4_INODE_MODE_FILE, EXT4_INODE_MODE_SOFTLINK,
        EXT4_SUPERBLOCK_OS_HURD, EXT4_SUPERBLOCK_OS_LINUX,
    };

    /// 创建一个测试用的超级块
    fn create_test_sblock() -> ext4_sblock {
        let mut sb: ext4_sblock = unsafe { core::mem::zeroed() };
        sb.creator_os = crate::ext4_misc::to_le32(EXT4_SUPERBLOCK_OS_LINUX);
        sb.rev_level = crate::ext4_misc::to_le32(1);
        sb.inode_size = crate::ext4_misc::to_le16(EXT4_GOOD_OLD_INODE_SIZE as u16);
        sb.log_block_size = crate::ext4_misc::to_le32(0); // 1024 bytes
        sb
    }

    /// 创建一个测试用的 inode
    fn create_test_inode() -> ext4_inode {
        unsafe { core::mem::zeroed() }
    }

    #[test]
    fn test_block_bits_count() {
        assert_eq!(ext4_inode_block_bits_count(256), 8);
        assert_eq!(ext4_inode_block_bits_count(512), 9);
        assert_eq!(ext4_inode_block_bits_count(1024), 10);
        assert_eq!(ext4_inode_block_bits_count(2048), 11);
        assert_eq!(ext4_inode_block_bits_count(4096), 12);
    }

    #[test]
    fn test_mode_get_set() {
        let mut sb = create_test_sblock();
        let mut inode = create_test_inode();

        // 测试普通模式
        ext4_inode_set_mode(&sb, &mut inode, 0o755);
        assert_eq!(ext4_inode_get_mode(&sb, &inode), 0o755);

        // 测试 Hurd 扩展模式
        sb.creator_os = crate::ext4_misc::to_le32(EXT4_SUPERBLOCK_OS_HURD);
        ext4_inode_set_mode(&sb, &mut inode, 0x12345678);
        assert_eq!(ext4_inode_get_mode(&sb, &inode), 0x12345678);
    }

    #[test]
    fn test_uid_gid_get_set() {
        let mut inode = create_test_inode();

        ext4_inode_set_uid(&mut inode, 1000);
        assert_eq!(ext4_inode_get_uid(&inode), 1000);

        ext4_inode_set_gid(&mut inode, 2000);
        assert_eq!(ext4_inode_get_gid(&inode), 2000);

        // 测试高位截断
        ext4_inode_set_uid(&mut inode, 0x12345678);
        assert_eq!(ext4_inode_get_uid(&inode), 0x5678);
    }

    #[test]
    fn test_size_get_set() {
        let mut sb = create_test_sblock();
        let mut inode = create_test_inode();

        // 测试小文件（32位）
        ext4_inode_set_size(&mut inode, 1024);
        assert_eq!(ext4_inode_get_size(&sb, &inode), 1024);

        // 测试大文件（64位，需要是文件类型）
        ext4_inode_set_mode(&sb, &mut inode, EXT4_INODE_MODE_FILE as u32 | 0o644);
        ext4_inode_set_size(&mut inode, 0x123456789ABCDEF0);
        assert_eq!(ext4_inode_get_size(&sb, &inode), 0x123456789ABCDEF0);

        // 测试非文件类型（只返回32位）
        ext4_inode_set_mode(&sb, &mut inode, EXT4_INODE_MODE_DIRECTORY as u32 | 0o755);
        ext4_inode_set_size(&mut inode, 0x123456789ABCDEF0);
        assert_eq!(ext4_inode_get_size(&sb, &inode), 0x9ABCDEF0);
    }

    #[test]
    fn test_csum_get_set() {
        let mut sb = create_test_sblock();
        let mut inode = create_test_inode();

        // 测试小 inode
        sb.inode_size = crate::ext4_misc::to_le16(EXT4_GOOD_OLD_INODE_SIZE as u16);
        ext4_inode_set_csum(&sb, &mut inode, 0x12345678);
        assert_eq!(ext4_inode_get_csum(&sb, &inode), 0x5678);

        // 测试大 inode
        sb.inode_size = crate::ext4_misc::to_le16(256);
        ext4_inode_set_csum(&sb, &mut inode, 0x12345678);
        assert_eq!(ext4_inode_get_csum(&sb, &inode), 0x12345678);
    }

    #[test]
    fn test_time_get_set() {
        let mut inode = create_test_inode();

        let time1 = 1234567890;
        let time2 = 987654321;

        ext4_inode_set_access_time(&mut inode, time1);
        assert_eq!(ext4_inode_get_access_time(&inode), time1);

        ext4_inode_set_modif_time(&mut inode, time2);
        assert_eq!(ext4_inode_get_modif_time(&inode), time2);

        ext4_inode_set_change_inode_time(&mut inode, time1);
        assert_eq!(ext4_inode_get_change_inode_time(&inode), time1);

        ext4_inode_set_del_time(&mut inode, time2);
        assert_eq!(ext4_inode_get_del_time(&inode), time2);
    }

    #[test]
    fn test_links_count_get_set() {
        let mut inode = create_test_inode();

        ext4_inode_set_links_cnt(&mut inode, 5);
        assert_eq!(ext4_inode_get_links_cnt(&inode), 5);

        ext4_inode_set_links_cnt(&mut inode, 0);
        assert_eq!(ext4_inode_get_links_cnt(&inode), 0);
    }

    #[test]
    fn test_blocks_count_get_set() {
        let mut sb = create_test_sblock();
        let mut inode = create_test_inode();

        // 测试32位块数
        let result = ext4_inode_set_blocks_count(&sb, &mut inode, 1000);
        assert_eq!(result, EOK);
        assert_eq!(ext4_inode_get_blocks_count(&sb, &inode), 1000);
        assert!(!ext4_inode_has_flag(&inode, EXT4_INODE_FLAG_HUGE_FILE));

        // 测试48位块数（需要 HUGE_FILE 特性）
        sb.features_read_only = crate::ext4_misc::to_le32(EXT4_FRO_COM_HUGE_FILE);
        let result = ext4_inode_set_blocks_count(&sb, &mut inode, 0x123456789ABC);
        assert_eq!(result, EOK);
        assert_eq!(ext4_inode_get_blocks_count(&sb, &inode), 0x123456789ABC);
        assert!(!ext4_inode_has_flag(&inode, EXT4_INODE_FLAG_HUGE_FILE));

        // 测试超大文件（需要设置 HUGE_FILE 标志）
        let huge_count = 0x123456789ABCDEF0;
        let result = ext4_inode_set_blocks_count(&sb, &mut inode, huge_count);
        assert_eq!(result, EOK);
        assert!(ext4_inode_has_flag(&inode, EXT4_INODE_FLAG_HUGE_FILE));

        // 测试不支持 HUGE_FILE 特性时的错误
        sb.features_read_only = crate::ext4_misc::to_le32(0);
        let result = ext4_inode_set_blocks_count(&sb, &mut inode, 0x100000000);
        assert_eq!(result, EINVAL);
    }

    #[test]
    fn test_flags_get_set() {
        let mut inode = create_test_inode();

        ext4_inode_set_flags(&mut inode, 0x12345678);
        assert_eq!(ext4_inode_get_flags(&inode), 0x12345678);

        // 测试设置标志
        ext4_inode_set_flag(&mut inode, EXT4_INODE_FLAG_APPEND);
        assert!(ext4_inode_has_flag(&inode, EXT4_INODE_FLAG_APPEND));

        // 测试清除标志
        ext4_inode_clear_flag(&mut inode, EXT4_INODE_FLAG_APPEND);
        assert!(!ext4_inode_has_flag(&inode, EXT4_INODE_FLAG_APPEND));
    }

    #[test]
    fn test_generation_get_set() {
        let mut inode = create_test_inode();

        ext4_inode_set_generation(&mut inode, 12345);
        assert_eq!(ext4_inode_get_generation(&inode), 12345);
    }

    #[test]
    fn test_extra_isize_get_set() {
        let mut sb = create_test_sblock();
        let mut inode = create_test_inode();

        // 测试小 inode（无效）
        sb.inode_size = crate::ext4_misc::to_le16(EXT4_GOOD_OLD_INODE_SIZE as u16);
        ext4_inode_set_extra_isize(&sb, &mut inode, 100);
        assert_eq!(ext4_inode_get_extra_isize(&sb, &inode), 0);

        // 测试大 inode（有效）
        sb.inode_size = crate::ext4_misc::to_le16(256);
        ext4_inode_set_extra_isize(&sb, &mut inode, 100);
        assert_eq!(ext4_inode_get_extra_isize(&sb, &inode), 100);
    }

    #[test]
    fn test_file_acl_get_set() {
        let mut sb = create_test_sblock();
        let mut inode = create_test_inode();

        // 测试 Linux
        sb.creator_os = crate::ext4_misc::to_le32(EXT4_SUPERBLOCK_OS_LINUX);
        ext4_inode_set_file_acl(&mut inode, &sb, 0x123456789ABC);
        assert_eq!(ext4_inode_get_file_acl(&inode, &sb), 0x123456789ABC);

        // 测试非 Linux（只使用低32位）
        sb.creator_os = crate::ext4_misc::to_le32(EXT4_SUPERBLOCK_OS_HURD);
        ext4_inode_set_file_acl(&mut inode, &sb, 0x123456789ABC);
        // 非 Linux 系统只使用 file_acl_lo，所以是 0x56789ABC
        assert_eq!(ext4_inode_get_file_acl(&inode, &sb), 0x56789ABC);
    }

    #[test]
    fn test_direct_block_get_set() {
        let mut inode = create_test_inode();

        ext4_inode_set_direct_block(&mut inode, 0, 100);
        assert_eq!(ext4_inode_get_direct_block(&inode, 0), 100);

        ext4_inode_set_direct_block(&mut inode, 11, 200);
        assert_eq!(ext4_inode_get_direct_block(&inode, 11), 200);
    }

    #[test]
    fn test_indirect_block_get_set() {
        let mut inode = create_test_inode();

        ext4_inode_set_indirect_block(&mut inode, 0, 300);
        assert_eq!(ext4_inode_get_indirect_block(&inode, 0), 300);

        ext4_inode_set_indirect_block(&mut inode, 1, 400);
        assert_eq!(ext4_inode_get_indirect_block(&inode, 1), 400);
    }

    #[test]
    fn test_dev_get_set() {
        let mut inode = create_test_inode();

        // 测试小设备号（使用 slot 0）
        ext4_inode_set_dev(&mut inode, 0x1234);
        assert_eq!(ext4_inode_get_dev(&inode), 0x1234);
        assert_eq!(ext4_inode_get_direct_block(&inode, 0), 0x1234);
        assert_eq!(ext4_inode_get_direct_block(&inode, 1), 0);

        // 测试大设备号（使用 slot 1）
        ext4_inode_set_dev(&mut inode, 0x12345678);
        assert_eq!(ext4_inode_get_dev(&inode), 0x12345678);
        assert_eq!(ext4_inode_get_direct_block(&inode, 0), 0);
        assert_eq!(ext4_inode_get_direct_block(&inode, 1), 0x12345678);

        // 测试 slot 0 优先
        ext4_inode_set_direct_block(&mut inode, 0, 100);
        ext4_inode_set_direct_block(&mut inode, 1, 200);
        assert_eq!(ext4_inode_get_dev(&inode), 100);
    }

    #[test]
    fn test_inode_type() {
        let mut sb = create_test_sblock();
        let mut inode = create_test_inode();

        ext4_inode_set_mode(&sb, &mut inode, EXT4_INODE_MODE_FILE as u32 | 0o644);
        assert_eq!(
            ext4_inode_type(&sb, &inode),
            EXT4_INODE_MODE_FILE as u32
        );
        assert!(ext4_inode_is_type(&sb, &inode, EXT4_INODE_MODE_FILE as u32));

        ext4_inode_set_mode(&sb, &mut inode, EXT4_INODE_MODE_DIRECTORY as u32 | 0o755);
        assert_eq!(
            ext4_inode_type(&sb, &inode),
            EXT4_INODE_MODE_DIRECTORY as u32
        );
        assert!(ext4_inode_is_type(
            &sb,
            &inode,
            EXT4_INODE_MODE_DIRECTORY as u32
        ));
    }

    #[test]
    fn test_can_truncate() {
        let mut sb = create_test_sblock();
        let mut inode = create_test_inode();

        // 测试普通文件（可以截断）
        ext4_inode_set_mode(&sb, &mut inode, EXT4_INODE_MODE_FILE as u32 | 0o644);
        assert!(ext4_inode_can_truncate(&sb, &inode));

        // 测试目录（可以截断）
        ext4_inode_set_mode(&sb, &mut inode, EXT4_INODE_MODE_DIRECTORY as u32 | 0o755);
        assert!(ext4_inode_can_truncate(&sb, &inode));

        // 测试软链接（可以截断）
        ext4_inode_set_mode(&sb, &mut inode, EXT4_INODE_MODE_SOFTLINK as u32 | 0o777);
        assert!(ext4_inode_can_truncate(&sb, &inode));

        // 测试 APPEND 标志（不能截断）
        ext4_inode_set_mode(&sb, &mut inode, EXT4_INODE_MODE_FILE as u32 | 0o644);
        ext4_inode_set_flag(&mut inode, EXT4_INODE_FLAG_APPEND);
        assert!(!ext4_inode_can_truncate(&sb, &inode));

        // 测试 IMMUTABLE 标志（不能截断）
        ext4_inode_clear_flag(&mut inode, EXT4_INODE_FLAG_APPEND);
        ext4_inode_set_flag(&mut inode, EXT4_INODE_FLAG_IMMUTABLE);
        assert!(!ext4_inode_can_truncate(&sb, &inode));
    }

    #[test]
    fn test_extent_header() {
        let mut inode = create_test_inode();

        // 测试 extent header 指针不为空
        let header = ext4_inode_get_extent_header(&mut inode);
        assert!(!header.is_null());
    }

    #[test]
    fn test_edge_cases() {
        let mut sb = create_test_sblock();
        let mut inode = create_test_inode();

        // 测试最大值
        ext4_inode_set_size(&mut inode, u64::MAX);
        ext4_inode_set_mode(&sb, &mut inode, EXT4_INODE_MODE_FILE as u32 | 0o644);
        assert_eq!(ext4_inode_get_size(&sb, &inode), u64::MAX);

        // 测试零值
        ext4_inode_set_size(&mut inode, 0);
        assert_eq!(ext4_inode_get_size(&sb, &inode), 0);

        // 测试所有标志位
        ext4_inode_set_flags(&mut inode, u32::MAX);
        assert_eq!(ext4_inode_get_flags(&inode), u32::MAX);

        // 测试清除所有标志
        ext4_inode_clear_flag(&mut inode, u32::MAX);
        assert_eq!(ext4_inode_get_flags(&inode), 0);
    }
}

