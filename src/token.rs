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

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    tag: Tag,
}

impl Token {
    pub(crate) fn new(tag: Tag) -> Self{
        Token {
            tag,
        }
    }

    pub(crate) fn get_tag(&self) -> Tag {
        self.tag
    }
}

impl TokenToString for Token {
    fn to_string(&self) -> String {
        TOKEN_NAME[self.tag as usize].to_string()
    }
}

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct Num {
    token: Token,
    val: isize,
}

impl Num {
    pub(crate) fn new(v: isize) -> Self {
        Num {
            token: Token::new(NUM),
            val: v,
        }
    }

    pub(crate) fn get_val(&self) -> isize {
        self.val
    }
}

impl TokenToString for Num {
    fn to_string(&self) -> String {
        format!("[{}]:{}", self.token.to_string(), self.val)
    }
}

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub enum TokenType {
    Token(Token),
    Id(Id),
    Str(Str),
    Num(Num),
    Char(Char),
}

impl TokenType {
    pub(crate) fn get_tag(&self) -> Tag {
        let tag =  match self {
                            TokenType::Token(t) => t.get_tag(),
                            TokenType::Id(id) => id.token.get_tag(),
                            TokenType::Str(str) => str.token.get_tag(),
                            TokenType::Num(num) => num.token.get_tag(),
                            TokenType::Char(c) => c.token.get_tag()
                        };

        tag
    }

    pub(crate) fn to_string(&self) -> String {
        return match self {
            TokenType::Token(t) => to_string(t),
            TokenType::Id(id) => to_string(id),
            TokenType::Str(str) => to_string(str),
            TokenType::Num(num) => to_string(num),
            TokenType::Char(c) => to_string(c)
        }
    }
}

fn to_string(t: &dyn TokenToString) -> String {
    t.to_string()
}
