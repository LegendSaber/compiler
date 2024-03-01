
use crate::common::Tag;
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};

pub struct Parser<'a> {
    lexer: &'a mut Lexer<'a>,
    look: TokenType,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(lexer: &'a mut Lexer<'a>, token_type: TokenType) -> Self {
        Parser {
            lexer,
            look: token_type,
        }
    }

    // 移进
    fn move_token(&mut self) {
        self.look = self.lexer.tokenize();
    }

    // 匹配，查看并移动
    fn match_tag(&mut self, need: Tag) -> bool {
        if self.look.get_tag() == need {
            self.move_token();
            return true;
        }

        return false;
    }
}

impl<'a> Parser<'a> {

}
