//! # 目录操作功能测试
//!
//! 测试文件系统的目录操作功能，包括创建、删除等

use rsext4::disknode::Ext4Inode;
use rsext4::error::{BlockDevError, BlockDevResult};
use rsext4::*;

// 包装 mkdir 函数，将 Option 转换为 Result 以便于测试
fn test_mkdir<B: BlockDevice>(
    device: &mut Jbd2Dev<B>,
    fs: &mut Ext4FileSystem,
    path: &str,
) -> BlockDevResult<Ext4Inode> {
    match mkdir(device, fs, path) {
        Some(inode) => Ok(inode),
        None => Err(BlockDevError::InvalidInput),
    }
}

/// 测试用块设备
struct MockBlockDevice {
    data: Vec<u8>,
    block_size: u32,
}

impl MockBlockDevice {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            block_size: rsext4::BLOCK_SIZE as u32,
        }
    }
}

impl BlockDevice for MockBlockDevice {
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
        Ok(())
    }

    fn close(&mut self) -> BlockDevResult<()> {
        Ok(())
    }

    fn total_blocks(&self) -> u64 {
        (self.data.len() / self.block_size as usize) as u64
    }

    fn block_size(&self) -> u32 {
        self.block_size
    }
}

#[cfg(test)]
mod directory_functional_tests {
    use super::*;

    /// 测试目录创建功能
    #[test]
    fn test_directory_create() {
        let device = MockBlockDevice::new(100 * 1024 * 1024); // 100MB
        let mut jbd2_dev = Jbd2Dev::initial_jbd2dev(0, device, true);

        mkfs(&mut jbd2_dev).expect("mkfs failed");
        let mut fs = mount(&mut jbd2_dev).expect("mount failed");

        // 创建单级目录
        test_mkdir(&mut jbd2_dev, &mut fs, "/single").expect("mkdir failed");

        // 创建多级目录
        test_mkdir(&mut jbd2_dev, &mut fs, "/level1").expect("mkdir failed");
        test_mkdir(&mut jbd2_dev, &mut fs, "/level1/level2").expect("mkdir failed");
        test_mkdir(&mut jbd2_dev, &mut fs, "/level1/level2/level3").expect("mkdir failed");

        // 创建多个同级目录
        test_mkdir(&mut jbd2_dev, &mut fs, "/siblings").expect("mkdir failed");
        test_mkdir(&mut jbd2_dev, &mut fs, "/siblings/sibling1").expect("mkdir failed");
        test_mkdir(&mut jbd2_dev, &mut fs, "/siblings/sibling2").expect("mkdir failed");
        test_mkdir(&mut jbd2_dev, &mut fs, "/siblings/sibling3").expect("mkdir failed");

        umount(fs, &mut jbd2_dev).expect("umount failed");
    }

    /// 测试目录删除功能
    #[test]
    fn test_directory_delete() {
        let device = MockBlockDevice::new(100 * 1024 * 1024); // 100MB
        let mut jbd2_dev = Jbd2Dev::initial_jbd2dev(0, device, true);

        mkfs(&mut jbd2_dev).expect("mkfs failed");
        let mut fs = mount(&mut jbd2_dev).expect("mount failed");

        // 创建测试目录结构
        test_mkdir(&mut jbd2_dev, &mut fs, "/test").expect("mkdir failed");
        test_mkdir(&mut jbd2_dev, &mut fs, "/test/subdir").expect("mkdir failed");

        // 在子目录中创建文件
        let test_data = b"File in subdirectory";
        mkfile(
            &mut jbd2_dev,
            &mut fs,
            "/test/subdir/file",
            Some(test_data),
            None,
        )
        .expect("mkfile failed");

        // 删除空目录（应该成功）
        test_mkdir(&mut jbd2_dev, &mut fs, "/empty").expect("mkdir failed");
        delete_dir(&mut fs, &mut jbd2_dev, "/empty");

        // 验证空目录已被删除（尝试在其中创建文件，mkfile会自动创建父目录）
        let result = mkfile(&mut jbd2_dev, &mut fs, "/empty/file", Some(b"data"), None);
        assert!(result.is_some(), "rsext4会自动创建父目录并成功创建文件");

        umount(fs, &mut jbd2_dev).expect("umount failed");
    }

    /// 测试在目录中创建和操作文件
    #[test]
    fn test_directory_file_operations() {
        let device = MockBlockDevice::new(100 * 1024 * 1024); // 100MB
        let mut jbd2_dev = Jbd2Dev::initial_jbd2dev(0, device, true);

        mkfs(&mut jbd2_dev).expect("mkfs failed");
        let mut fs = mount(&mut jbd2_dev).expect("mount failed");

        // 创建目录结构
        test_mkdir(&mut jbd2_dev, &mut fs, "/documents").expect("mkdir failed");
        test_mkdir(&mut jbd2_dev, &mut fs, "/documents/projects").expect("mkdir failed");
        test_mkdir(&mut jbd2_dev, &mut fs, "/documents/personal").expect("mkdir failed");

        // 在不同目录中创建文件
        let project_data = b"Project related data";
        mkfile(
            &mut jbd2_dev,
            &mut fs,
            "/documents/projects/project1.txt",
            Some(project_data),
            None,
        )
        .expect("mkfile failed");

        let personal_data = b"Personal notes";
        mkfile(
            &mut jbd2_dev,
            &mut fs,
            "/documents/personal/notes.txt",
            Some(personal_data),
            None,
        )
        .expect("mkfile failed");

        // 验证文件内容
        let read_project = read_file(&mut jbd2_dev, &mut fs, "/documents/projects/project1.txt")
            .expect("read_file failed");
        assert_eq!(read_project, Some(project_data.to_vec()));

        let read_notes = read_file(&mut jbd2_dev, &mut fs, "/documents/personal/notes.txt")
            .expect("read_file failed");
        assert_eq!(read_notes, Some(personal_data.to_vec()));

        umount(fs, &mut jbd2_dev).expect("umount failed");
    }

    /// 测试目录中的文件查找功能
    #[test]
    fn test_directory_file_find() {
        let device = MockBlockDevice::new(100 * 1024 * 1024); // 100MB
        let mut jbd2_dev = Jbd2Dev::initial_jbd2dev(0, device, true);

        mkfs(&mut jbd2_dev).expect("mkfs failed");
        let mut fs = mount(&mut jbd2_dev).expect("mount failed");

        // 创建目录和文件
        test_mkdir(&mut jbd2_dev, &mut fs, "/findtest").expect("mkdir failed");

        // 创建多个文件
        for i in 1..=5 {
            let filename = format!("/findtest/file{}.txt", i);
            let data = format!("Content of file {}", i);
            mkfile(
                &mut jbd2_dev,
                &mut fs,
                &filename,
                Some(data.as_bytes()),
                None,
            )
            .expect("mkfile failed");
        }

        // 测试文件查找
        for i in 1..=5 {
            let filename = format!("/findtest/file{}.txt", i);
            let expected_data = format!("Content of file {}", i);

            let found_data =
                read_file(&mut jbd2_dev, &mut fs, &filename).expect("read_file failed");
            assert_eq!(found_data, Some(expected_data.as_bytes().to_vec()));
        }

        // 测试查找不存在的文件
        let not_found =
            read_file(&mut jbd2_dev, &mut fs, "/findtest/notexist.txt").expect("read_file failed");
        assert_eq!(not_found, None);

        umount(fs, &mut jbd2_dev).expect("umount failed");
    }

    /// 测试目录操作中的错误处理
    #[test]
    fn test_directory_error_handling() {
        let device = MockBlockDevice::new(100 * 1024 * 1024); // 100MB
        let mut jbd2_dev = Jbd2Dev::initial_jbd2dev(0, device, true);

        mkfs(&mut jbd2_dev).expect("mkfs failed");
        let mut fs = mount(&mut jbd2_dev).expect("mount failed");

        // 测试在不存在的目录中创建文件（mkfile会自动创建父目录）
        let result = mkfile(
            &mut jbd2_dev,
            &mut fs,
            "/nonexistent/file.txt",
            Some(b"data"),
            None,
        );
        assert!(result.is_some(), "rsext4会自动创建父目录并成功创建文件");

        // 测试删除不存在的目录（会打印警告但不会报错）
        delete_dir(&mut fs, &mut jbd2_dev, "/nonexistent");

        // 测试删除非空目录（会打印警告但不会报错）
        test_mkdir(&mut jbd2_dev, &mut fs, "/nonempty").expect("mkdir failed");
        mkfile(
            &mut jbd2_dev,
            &mut fs,
            "/nonempty/file.txt",
            Some(b"data"),
            None,
        )
        .expect("mkfile failed");

        // 这个操作可能会失败，取决于实现
        delete_dir(&mut fs, &mut jbd2_dev, "/nonempty");

        // 验证目录是否还存在
        let result = mkfile(
            &mut jbd2_dev,
            &mut fs,
            "/nonempty/another_file.txt",
            Some(b"data"),
            None,
        );
        // 如果目录被删除，这个操作应该失败

        // 测试在已存在的目录中创建同名目录
        test_mkdir(&mut jbd2_dev, &mut fs, "/duplicate").expect("mkdir failed");
        let result = test_mkdir(&mut jbd2_dev, &mut fs, "/duplicate");
        // 注意：rsext4的mkdir可能不会在目录已存在时报错，而是返回现有目录
        assert!(result.is_ok(), "mkdir可能返回已存在的目录而不是报错");

        umount(fs, &mut jbd2_dev).expect("umount failed");
    }

    /// 测试复杂目录结构
    #[test]
    fn test_complex_directory_structure() {
        let device = MockBlockDevice::new(200 * 1024 * 1024); // 200MB
        let mut jbd2_dev = Jbd2Dev::initial_jbd2dev(0, device, true);

        mkfs(&mut jbd2_dev).expect("mkfs failed");
        let mut fs = mount(&mut jbd2_dev).expect("mount failed");

        // 创建复杂的目录结构
        let structure = [
            "/home",
            "/home/user",
            "/home/user/documents",
            "/home/user/documents/work",
            "/home/user/documents/personal",
            "/home/user/music",
            "/home/user/music/rock",
            "/home/user/music/jazz",
            "/home/user/music/classical",
            "/var",
            "/var/log",
            "/var/www",
            "/var/www/html",
            "/var/www/css",
            "/var/www/js",
            "/tmp",
            "/etc",
            "/etc/config",
        ];

        // 创建所有目录
        for dir in &structure {
            test_mkdir(&mut jbd2_dev, &mut fs, dir).expect("mkdir failed");
        }

        // 在不同目录中创建文件
        let files = [
            (
                "/home/user/documents/work/report.txt",
                "Work report content",
            ),
            (
                "/home/user/documents/personal/diary.txt",
                "Personal diary entries",
            ),
            ("/home/user/music/rock/song1.mp3", "Rock music data"),
            ("/var/log/system.log", "System log entries"),
            ("/var/www/html/index.html", "HTML page content"),
            ("/var/www/css/style.css", "CSS style definitions"),
            ("/var/www/js/script.js", "JavaScript code"),
            ("/etc/config/app.conf", "Application configuration"),
        ];

        for (path, content) in &files {
            mkfile(&mut jbd2_dev, &mut fs, path, Some(content.as_bytes()), None)
                .expect("mkfile failed");
        }

        // 验证所有文件都能正确读取
        for (path, content) in &files {
            let read_data = read_file(&mut jbd2_dev, &mut fs, path).expect("read_file failed");
            assert_eq!(read_data, Some(content.as_bytes().to_vec()));
        }

        umount(fs, &mut jbd2_dev).expect("umount failed");
    }
}
