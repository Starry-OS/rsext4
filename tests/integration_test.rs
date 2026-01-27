//! # 集成测试
//!
//! 测试 ext4 文件系统的基本功能

use rsext4::error::{BlockDevError, BlockDevResult};
use rsext4::*;

/// 创建一个简单的测试块设备
struct TestBlockDevice {
    data: Vec<u8>,
    block_size: u32,
    is_open: bool,
}

impl TestBlockDevice {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            block_size: rsext4::BLOCK_SIZE as u32, // 使用与 ext4 相同的块大小
            is_open: false,
        }
    }
}

impl BlockDevice for TestBlockDevice {
    fn read(&mut self, buffer: &mut [u8], block_id: u32, _count: u32) -> BlockDevResult<()> {
        let start = (block_id as u64 * self.block_size as u64) as usize;
        let end = start + buffer.len();
        if end > self.data.len() {
            return Err(BlockDevError::BlockOutOfRange {
                block_id,
                max_blocks: (self.data.len() / self.block_size as usize) as u64,
            });
        }
        buffer.copy_from_slice(&self.data[start..end]);
        Ok(())
    }

    fn write(&mut self, buffer: &[u8], block_id: u32, _count: u32) -> BlockDevResult<()> {
        let start = (block_id as u64 * self.block_size as u64) as usize;
        let end = start + buffer.len();
        if end > self.data.len() {
            return Err(BlockDevError::BlockOutOfRange {
                block_id,
                max_blocks: (self.data.len() / self.block_size as usize) as u64,
            });
        }
        self.data[start..end].copy_from_slice(buffer);
        Ok(())
    }

    fn open(&mut self) -> BlockDevResult<()> {
        self.is_open = true;
        Ok(())
    }

    fn close(&mut self) -> BlockDevResult<()> {
        self.is_open = false;
        Ok(())
    }

    fn total_blocks(&self) -> u64 {
        (self.data.len() / self.block_size as usize) as u64
    }

    fn block_size(&self) -> u32 {
        self.block_size
    }
}

#[test]
fn test_basic_mount_mkfs() {
    let device = TestBlockDevice::new(100 * 1024 * 1024); // 100MB
    let mut jbd2_dev = Jbd2Dev::initial_jbd2dev(0, device, true);

    // 格式化文件系统
    mkfs(&mut jbd2_dev).expect("mkfs failed");

    // 挂载文件系统
    let mut fs = mount(&mut jbd2_dev).expect("mount failed");

    // 创建目录
    mkdir(&mut jbd2_dev, &mut fs, "/test").expect("mkdir failed");

    // 创建文件
    let data = b"Hello, world!";
    mkfile(&mut jbd2_dev, &mut fs, "/test/hello.txt", Some(data), None).expect("mkfile failed");

    // 读取文件
    let read_data = read_file(&mut jbd2_dev, &mut fs, "/test/hello.txt").expect("read_file failed");
    assert_eq!(read_data, Some(data.to_vec()));

    // 卸载文件系统
    umount(fs, &mut jbd2_dev).expect("umount failed");
}

#[test]
fn test_file_operations() {
    let device = TestBlockDevice::new(100 * 1024 * 1024); // 100MB
    let mut jbd2_dev = Jbd2Dev::initial_jbd2dev(0, device, true);

    mkfs(&mut jbd2_dev).expect("mkfs failed");
    let mut fs = mount(&mut jbd2_dev).expect("mount failed");

    mkdir(&mut jbd2_dev, &mut fs, "/filetest").expect("mkdir failed");

    // 创建一个空文件
    mkfile(&mut jbd2_dev, &mut fs, "/filetest/empty.txt", None, None).expect("mkfile failed");

    // 写入数据
    write_file(
        &mut jbd2_dev,
        &mut fs,
        "/filetest/empty.txt",
        0,
        b"First line",
    )
    .expect("write_file failed");

    // 追加数据
    let file_len = read_file(&mut jbd2_dev, &mut fs, "/filetest/empty.txt")
        .expect("read_file failed")
        .unwrap_or_default()
        .len();
    write_file(
        &mut jbd2_dev,
        &mut fs,
        "/filetest/empty.txt",
        file_len as u64,
        b"\nSecond line",
    )
    .expect("write_file failed");

    // 读取并验证
    let data = read_file(&mut jbd2_dev, &mut fs, "/filetest/empty.txt").expect("read_file failed");
    assert_eq!(data, Some(b"First line\nSecond line".to_vec()));

    // 使用 API 进行文件操作
    let mut file = open(&mut jbd2_dev, &mut fs, "/filetest/api.txt", true).expect("open failed");

    write_at(&mut jbd2_dev, &mut fs, &mut file, b"API test").expect("write_at failed");
    assert!(lseek(&mut file, 0));

    let bytes_read = read_at(&mut jbd2_dev, &mut fs, &mut file, 8).expect("read_at failed");
    assert_eq!(bytes_read, b"API test");

    umount(fs, &mut jbd2_dev).expect("umount failed");
}
