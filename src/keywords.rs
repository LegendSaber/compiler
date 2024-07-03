use std::collections::HashMap;
use crate::common::Tag::{self, *};

pub struct Keywords {
    keywords: HashMap<String, Tag>
}

// 初始化关键字列表
impl Keywords {
    pub(crate) fn new() -> Self {
        let mut keywords: HashMap<String, Tag> = HashMap::default();

        keywords.insert("int".to_string(), KwInt);
        keywords.insert("char".to_string(), KwChar);
        keywords.insert("void".to_string(), KwVoid);
        keywords.insert("extern".to_string(), KwExtern);
        keywords.insert("if".to_string(), KwIf);
        keywords.insert("else".to_string(), KwElse);
        keywords.insert("switch".to_string(), KwSwitch);
        keywords.insert("case".to_string(), KwCase);
        keywords.insert("default".to_string(), KwDefault);
        keywords.insert("while".to_string(), KwWhile);
        keywords.insert("do".to_string(), KwDo);
        keywords.insert("for".to_string(), KwFor);
        keywords.insert("break".to_string(), KwBreak);
        keywords.insert("continue".to_string(), KwContinue);
        keywords.insert("return".to_string(), KwReturn);

        Keywords {
            keywords
        }
    }

    // 测试是否是关键字
    pub(crate) fn get_tag(&self, name: String) -> Tag {
        *(self.keywords.get(&name).unwrap_or_else(|| &ID))
    }
}
