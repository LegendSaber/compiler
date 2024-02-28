use failure;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};

const BUF_LEN: usize = 20;

pub struct Scanner {
    file_name: String,                  // 文件名
    file: Option<File>,                 // 文件指针

    // 内部状态
    line: [u8; BUF_LEN],                // 文件内容
    line_len: usize,                    // 当前行的长度
    read_pos: usize,                    // 读取的位置
    last_char: char,                    // 上一个字符，主要用于判断行位置
    need_read: bool,                      // 保存是否读取到缓冲区最后一个字符

    //读取状态
    line_num: usize,                    // 记录行号
    col_num: usize,                     // 记录列号
}

impl Scanner {
    pub fn new(file_name: String) -> Result<Scanner, failure::Error> {
        let file = File::open(&file_name)?;

        Ok(Scanner {
            file_name,
            file: Some(file),
            line: [0; BUF_LEN],
            line_len: 0,
            read_pos: 0,
            last_char: char::from(0),
            need_read: true,
            line_num: 1,
            col_num: 0,
        })
    }

    pub fn scan(&mut self) -> Option<char> {
        if let Some(file) = self.file.as_mut() {
            if self.need_read {
                self.line_len = file.read(&mut self.line[..]).unwrap_or_else(|_| 0);

                if self.line_len == 0 {     // 数据读取完毕
                    self.file = None;
                    return None;
                    // return Err(Error::new(ErrorKind::Other, "文件数据读取完毕").into())
                }
                self.need_read = false;
            }

            // 从缓冲区中读取一个字符
            let ch = self.line[self.read_pos] as char;
            self.read_pos += 1;
            if self.read_pos == self.line_len { // 读取到最后一个字符了，则记录
                self.read_pos = 0;
                self.need_read = true;
            }

            if self.last_char == '\n' {         // 新行
                self.line_num += 1;             // 行号累加
                self.col_num = 0;               // 列号清空
            }

            if ch != '\n' {                     // 不是换行
                self.col_num += 1;              // 列号新增
            }
            self.last_char = ch;                // 记录上一个字符

            return Some(ch);
        }

        return None;
        // Err(Error::new(ErrorKind::Other, "文件指针不存在").into())
    }

    pub fn file_name(&self) -> String {
        self.file_name.clone()
    }

    pub fn line_num(&self) -> usize {
        self.line_num
    }

    pub fn col_num(&self) -> usize {
        self.col_num
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::Scanner;

    #[test]
    fn test_scan() {
        let test_str = "int val_name\r\nadd\r\nchar\r\nfunc_tests\r\n+-*/";
        let mut scanner = Scanner::new("./test_file/scanner.txt".to_string()).unwrap();

        for c in test_str.chars() {
            let ch = scanner.scan();
            if let None = ch {
                assert!(false);
                break;
            }
            assert_eq!(ch.unwrap(), c);
        }

        if let None = scanner.scan() {
            assert!(true)
        }
    }
}
