use crate::common::Tag::{self, CH, ID, NUM, STR};

const TOKEN_NAME: [&str; 48] = [
    "error",                                      // 错误，异常，结束标记等
    "文件结尾",                                    // 文件结束
    "标识符",                                      // 标识符
    "int", "char", "void",                        // 数据类型
    "extern",                                     // extern
    "数字", "字符", "字符串",                      // 字面量
    "!", "&",                                    // 单目运算 ! - & *
    "+", "-", "*", "/", "%",                     // 算术运算符
    "++","--",
    ">", ">=", "<", "<=", "==", "!=",            // 比较运算符
    "&&", "||",                                  // 逻辑运算
    "(", ")",                                    // ()
    "[", "]",                                    // []
    "{", "}",                                    // {}
    ",", ":", ";",                               // 逗号,冒号,分号
    "=",                                         // 赋值
    "if", "else",                                // if-else
    "switch", "case", "default",                 // swicth-case-deault
    "while", "do", "for",                        // 循环
    "break", "continue", "return"                // break, continue, return
];

trait TokenToString {
    fn to_string(&self) -> String;
}

#[derive(Clone, PartialEq)]
pub struct Token {
    tag: Tag,
}

impl Token {
    pub(crate) fn new(tag: Tag) -> Self{
        Token {
            tag,
        }
    }
}

impl TokenToString for Token {
    fn to_string(&self) -> String {
        TOKEN_NAME[self.tag as usize].to_string()
    }
}

#[derive(Clone, PartialEq)]
pub struct Id {
    token: Token,
    name:  String,
}

impl Id {
    pub(crate) fn new(n: String) -> Self {
        Id {
            token: Token::new(ID),
            name: n,
        }
    }
}

impl TokenToString for Id {
    fn to_string(&self) -> String {
        format!("[{}]:{}", self.token.to_string(), self.name.as_str())
    }
}

#[derive(Clone, PartialEq)]
pub struct Str {
    token: Token,
    str: String,
}

impl Str {
    pub(crate) fn new(s: String) -> Self {
        Str {
            token: Token::new(STR),
            str: s,
        }
    }
}

impl TokenToString for Str {
    fn to_string(&self) -> String {
        format!("[{}]:{}", self.token.to_string(), self.str.as_str())
    }
}

#[derive(Clone, PartialEq)]
pub struct Num {
    token: Token,
    val: i32,
}

impl Num {
    pub(crate) fn new(v: i32) -> Self {
        Num {
            token: Token::new(NUM),
            val: v,
        }
    }
}

impl TokenToString for Num {
    fn to_string(&self) -> String {
        format!("[{}]:{}", self.token.to_string(), self.val)
    }
}

#[derive(Clone, PartialEq)]
pub struct Char {
    token: Token,
    ch:    char,
}

impl Char {
    pub(crate) fn new(c: char) -> Self {
        Char {
            token: Token::new(CH),
            ch: c,
        }
    }
}

impl TokenToString for Char {
    fn to_string(&self) -> String {
        format!("[{}]:{}", self.token.to_string(), self.ch)
    }
}

#[derive(Clone, PartialEq)]
pub enum TokenType {
    Token(Token),
    Id(Id),
    Str(Str),
    Num(Num),
    Char(Char),
}
