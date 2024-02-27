use crate::common::LexError::{CHAR_NO_DATA, CHAR_NO_R_QUTION, COMMENT_NO_END, NUM_BIN_TYPE, NUM_HEX_TYPE, STR_NO_R_QUTION, TOKEN_NO_EXIST};
use crate::common::Tag::{self, ADD, ASSIGN, DEC, END, ERR, GE, GT, ID, INC, LE, LT, MOD, MUL, SUB, EQU, LEA, AND, NEQU, NOT, COMMA, COLON, SEMICON, LPAREN, RPAREN, LBRACK, RBRACK, LBRACE, RBRACE, DIV};
use crate::scanner::Scanner;
use crate::keywords::Keywords;
use crate::token::{Char, Id, Num, Str, Token, TokenType};

fn lex_error(scanner: &mut Scanner, code: usize) {
    // 打印词法错误
    let lex_error_table = ["字符串丢失右引号",
        "二进制数没有实体数据",
        "十六进制数没有实体数据",
        "字符丢失右单引号",
        "不支持空字符",
        "错误的或运算符",
        "多行注释没有正常结束",
        "词法记号不存在"];

    println!("{}<{}行,{}列> 词法错误 : {}.\n", scanner.file_name(),
             scanner.line_num(), scanner.col_num(), lex_error_table[code]);
}

pub struct Lexer<'a> {
    scanner: &'a mut Scanner,
    ch: Option<char>,
    token: Option<TokenType>,
    keywords: Keywords,
}

impl<'a> Lexer<'a> {
    fn new(scanner: &'a mut Scanner) -> Self {
        Lexer {
            scanner,
            ch: None,
            token: None,
            keywords: Keywords::new(),
        }
    }

    fn scan(&mut self, need: Option<char>) -> bool {
        self.ch = self.scanner.scan();
        if need != None {
            if self.ch != need {
                return false;
            }

            self.ch = self.scanner.scan();
        }

        true
    }

    fn tokenize(&mut self) -> TokenType {
        let mut token: Option<TokenType> = None;
        loop {
            if let None = self.ch {
                break;
            }

            let mut ch = self.ch.unwrap();

            // 忽略空白符
            while ch == ' ' || ch == '\n' || ch == '\r' || ch == '\t' {
                self.scan(None);
                continue;
            }

            // 标识符，关键字
            if (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch == '_' {
                let mut name = String::from("");

                while (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch == '_' {
                    name.push(ch);              // 记录字符
                    self.scan(None);
                    if let None = self.ch {
                        break;
                    }
                    ch = self.ch.unwrap();
                }

                // 匹配结束
                let tag: Tag = self.keywords.get_tag(name.clone());
                if tag == ID {          // 正常的标识符
                    token = Some(TokenType::Id(Id::new(name)));
                } else {                // 关键字
                    token = Some(TokenType::Token(Token::new(tag)));
                }
            } else if ch == '"' {   // 字符串
                let mut str = String::from("");

                while !self.scan(Some('"')) {
                    match self.ch {
                        Some(c) => {
                            if c == '\\' {
                                self.scan(None);
                                match self.ch {
                                    Some(c) => {
                                        match c {
                                            'n' => str.push('\n'),
                                            '\\' => str.push('\\'),
                                            't' => str.push('\t'),
                                            '"' => str.push('"'),
                                            '0' => str.push('\0'),
                                            '\n' => {},
                                            _ => {
                                                str.push(c);
                                            }
                                        };
                                    }
                                    None => {
                                        lex_error(self.scanner, STR_NO_R_QUTION as usize);
                                        token = Some(TokenType::Token(Token::new(ERR)));
                                    }
                                }
                            } else if c == '\n' {
                                lex_error(self.scanner, STR_NO_R_QUTION as usize);
                                token = Some(TokenType::Token(Token::new(ERR)));
                            } else {
                                str.push(c);
                            }
                        },
                        None => {
                            lex_error(self.scanner, STR_NO_R_QUTION as usize);
                            token = Some(TokenType::Token(Token::new(ERR)));
                            break;
                        }
                    }
                }

                // 最终字符串
                if let None = token {
                    token = Some(TokenType::Str(Str::new(str)));
                }
            } else if ch >= '0' && ch <= '9' {  // 数字
                let mut val: isize = 0;

                if ch != '0' {  // 10进制
                    while ch > '0' && ch < '9' {
                        val = val * 10 + ch as i32 - '0' as i32;
                        self.scan(None);
                        if let None = self.ch {
                            break;
                        }
                        ch = self.ch.unwrap();
                    }
                } else {
                    self.scan(None);
                    if let None = self.ch {
                        ch = self.ch.unwrap();
                        if ch == 'x' {      // 16进制
                            self.scan(None);
                            match self.ch {
                                Some(c) => {
                                    ch = c;
                                    if (ch >= '0' && ch <= '9') || (ch >= 'A' && ch <= 'F') || (ch >= 'a' && ch <= 'f') {
                                        while (ch >= '0' && ch <= '9') || (ch >= 'A' && ch <= 'F') || (ch >= 'a' && ch <= 'f') {
                                            val = val * 16 + ch as isize;
                                            if ch >= '0' && ch <= '9' {
                                                val -= '0' as isize;
                                            } else if ch >= 'A' && ch <= 'F' {
                                                val += 10 - 'A' as isize;
                                            } else if ch >= 'a' && ch <= 'f' {
                                                val += 10 - 'a' as isize;
                                            }
                                            self.scan(None);
                                            if let None = self.ch {
                                                break;
                                            }
                                            ch = self.ch.unwrap();
                                        }
                                    } else {
                                        lex_error(self.scanner, NUM_HEX_TYPE as usize);
                                        token = Some(TokenType::Token(Token::new(ERR)));
                                    }
                                },
                                None => {
                                    lex_error(self.scanner, NUM_HEX_TYPE as usize);
                                    token = Some(TokenType::Token(Token::new(ERR)));
                                }
                            }
                        } else if ch == 'b' {       // 二进制
                            self.scan(None);
                            match self.ch {
                                Some(c) => {
                                    ch = c;
                                    if ch >= '0' && ch <= '1' {
                                        while ch >= '0' && ch <= '1' {
                                            val = val * 2 + ch as isize - '0' as isize;
                                            self.scan(None);
                                            if let None = self.ch {
                                                break;
                                            }
                                            ch = self.ch.unwrap();
                                        }
                                    } else {
                                        lex_error(self.scanner, NUM_BIN_TYPE as usize);
                                        token = Some(TokenType::Token(Token::new(ERR)));
                                    }
                                },
                                None => {
                                    lex_error(self.scanner, NUM_BIN_TYPE as usize);
                                    token = Some(TokenType::Token(Token::new(ERR)));
                                }
                            }
                        } else if ch >= '0' && ch <= '7' {
                            while ch >= '0' && ch <= '7' {
                                val = val * 8 + ch as isize - '0' as isize;
                                self.scan(None);
                                if let None = self.ch {
                                    break;
                                }
                                ch = self.ch.unwrap();
                            }
                        }
                    }
                }

                // 最终数字
                if let None = token {
                    token = Some(TokenType::Num(Num::new(val)));
                }
            } else if ch == '\'' {  // 字符
                let mut c: char = '0';

                self.scan(None);
                match self.ch {
                    Some(ch) => {
                        if ch == '\'' {         // 没有数据
                            lex_error(self.scanner, CHAR_NO_DATA as usize);
                            token = Some(TokenType::Token(Token::new(ERR)));
                        } else if ch == '\n' {  // 换行
                            lex_error(self.scanner, CHAR_NO_R_QUTION as usize);
                            token = Some(TokenType::Token(Token::new(ERR)));
                        } else if ch == '\\' {  // 转义
                            self.scan(None);

                            match self.ch {
                                Some(ch) => {
                                    if ch == 'n' {
                                        c = '\n';
                                    } else if ch == '\\' {
                                        c = '\\';
                                    } else if ch == 't' {
                                        c = '\t';
                                    } else if ch == '0' {
                                        c = '\0';
                                    } else if ch == '\'' {
                                        c = '\'';
                                    } else if ch == '\n' {          // 换行
                                        lex_error(self.scanner, CHAR_NO_R_QUTION as usize);
                                        token = Some(TokenType::Token(Token::new(ERR)));
                                    }
                                },
                                None => {   // 文件结束
                                    lex_error(self.scanner, CHAR_NO_R_QUTION as usize);
                                    token = Some(TokenType::Token(Token::new(ERR)));
                                }
                            }
                        } else {
                            c = ch;
                        }
                    }
                    None => {   // 文件结束
                        lex_error(self.scanner, CHAR_NO_R_QUTION as usize);
                        token = Some(TokenType::Token(Token::new(ERR)));
                    }
                }

                if let None = token {
                    if self.scan(Some('\'')) {
                        token = Some(TokenType::Char(Char::new(c)));
                    } else {
                        lex_error(self.scanner, CHAR_NO_R_QUTION as usize);
                        token = Some(TokenType::Token(Token::new(ERR)));
                    }
                }
            } else {
                match ch {
                    '#' => {    // 忽略行（忽略宏定义）
                        loop {
                            match self.ch {
                                Some(c) => {
                                    if c == '\n' {      // 换行
                                        break;
                                    }
                                },
                                None => {               // 文件结束
                                    break;
                                }
                            }
                            self.scan(None);
                        }
                        token = Some(TokenType::Token(Token::new(ERR)));
                    },
                    '+' => {
                        if self.scan(Some('+')) {
                            token = Some(TokenType::Token(Token::new(ADD)));
                        } else {
                            token = Some(TokenType::Token(Token::new(INC)));
                        }
                    },
                    '-' => {
                        if self.scan(Some('-')) {
                            token = Some(TokenType::Token(Token::new(SUB)));
                        } else {
                            token = Some(TokenType::Token(Token::new(DEC)));
                        }
                    },
                    '*' => {
                        token = Some(TokenType::Token(Token::new(MUL)));
                        self.scan(None);
                    },
                    '/' => {
                        self.scan(None);
                        ch = self.ch.unwrap();
                        if ch == '/' {              // 单行注释
                            while ch != '\n' {
                                self.scan(None);
                                if let None = self.ch {
                                    break
                                }
                                ch = self.ch.unwrap();
                            }
                            token = Some(TokenType::Token(Token::new(ERR)));
                        } else if ch == '*' {       // 多行注释
                            loop {
                                self.scan(None);
                                if let None = self.ch {
                                    break;
                                }
                                if self.ch.unwrap() == '*' {
                                    if self.scan(Some('/')) {
                                        break;
                                    }
                                }
                            }
                            if let None = self.ch {
                                lex_error(self.scanner, COMMENT_NO_END as usize);
                            }
                        } else {
                            token = Some(TokenType::Token(Token::new(DIV)));
                        }
                    },
                    '%' => {
                        token = Some(TokenType::Token(Token::new(MOD)));
                        self.scan(None);
                    },
                    '>' => {
                        if self.scan(Some('=')) {
                            token = Some(TokenType::Token(Token::new(GE)));
                        } else {
                            token = Some(TokenType::Token(Token::new(GT)));
                        }
                    },
                    '<' => {
                        if self.scan(Some('=')) {
                            token = Some(TokenType::Token(Token::new(LE)));
                        } else {
                            token = Some(TokenType::Token(Token::new(LT)));
                        }
                    },
                    '=' => {
                        if self.scan(Some('=')) {
                            token = Some(TokenType::Token(Token::new(EQU)));
                        } else {
                            token = Some(TokenType::Token(Token::new(ASSIGN)));
                        }
                    },
                    '&' => {
                        if self.scan(Some('&')) {
                            token = Some(TokenType::Token(Token::new(AND)));
                        } else {
                            token = Some(TokenType::Token(Token::new(LEA)));
                        }
                    },
                    '|' => {
                        if self.scan(Some('=')) {
                            token = Some(TokenType::Token(Token::new(NEQU)));
                        } else {
                            token = Some(TokenType::Token(Token::new(NOT)));
                        }
                    },
                    ',' => {
                        token = Some(TokenType::Token(Token::new(COMMA)));
                        self.scan(None);
                    },
                    ':' => {
                        token = Some(TokenType::Token(Token::new(COLON)));
                        self.scan(None);
                    },
                    ';' => {
                        token = Some(TokenType::Token(Token::new(SEMICON)));
                        self.scan(None);
                    },
                    '(' => {
                        token = Some(TokenType::Token(Token::new(LPAREN)));
                        self.scan(None);
                    },
                    ')' => {
                        token = Some(TokenType::Token(Token::new(RPAREN)));
                        self.scan(None);
                    },
                    '[' => {
                        token = Some(TokenType::Token(Token::new(LBRACK)));
                        self.scan(None);
                    },
                    ']' => {
                        token = Some(TokenType::Token(Token::new(RBRACK)));
                        self.scan(None);
                    },
                    '{' => {
                        token = Some(TokenType::Token(Token::new(LBRACE)));
                        self.scan(None);
                    },
                    '}' => {
                        token = Some(TokenType::Token(Token::new(RBRACE)));
                        self.scan(None);
                    },
                    _ => {
                        token = Some(TokenType::Token(Token::new(ERR)));
                        lex_error(self.scanner, TOKEN_NO_EXIST as usize);
                    }
                }
            }

            self.token = token.clone();

            // 有效，则返回token
            if let Some(token_type) = token.clone() {
               match token_type.clone() {
                   TokenType::Token(t) => {
                        if t.get_tag() != ERR {
                            return token_type;
                        }
                   },
                   _ => {
                       return token_type;
                   }
               }
            }
            // 否则继续扫描，直到结束
        }

        if let None = token {
            token = Some(TokenType::Token(Token::new(END)));
        }
        self.token = token.clone();
        token.unwrap()
    }
}
