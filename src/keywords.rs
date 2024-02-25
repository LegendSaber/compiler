use std::collections::HashMap;
use std::ops::Deref;
use crate::common::Tag::{self, *};

pub struct Keywords {
    keywords: HashMap<String, Tag>
}

// 初始化关键字列表
impl Keywords {
    pub(crate) fn new() -> Self {
        let mut keywords: HashMap<String, Tag> = HashMap::default();

        keywords.insert("int".to_string(), KW_INT);
        keywords.insert("char".to_string(), KW_CHAR);
        keywords.insert("void".to_string(), KW_VOID);
        keywords.insert("extern".to_string(), KW_EXTERN);
        keywords.insert("if".to_string(), KW_IF);
        keywords.insert("else".to_string(), KW_ELSE);
        keywords.insert("switch".to_string(), KW_SWITCH);
        keywords.insert("case".to_string(), KW_CASE);
        keywords.insert("default".to_string(), KW_DEFAULT);
        keywords.insert("while".to_string(), KW_WHILE);
        keywords.insert("do".to_string(), KW_DO);
        keywords.insert("for".to_string(), KW_FOR);
        keywords.insert("break".to_string(), KW_BREAK);
        keywords.insert("continue".to_string(), KW_CONTINUE);
        keywords.insert("return".to_string(), KW_RETURN);

        Keywords {
            keywords
        }
    }

    // 测试是否是关键字
    pub(crate) fn get_tag(&self, name: String) -> Tag {
        *(self.keywords.get(&name).unwrap_or_else(|| &ID))
    }
}
