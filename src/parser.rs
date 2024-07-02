use std::any::Any;
use crate::common::SynError::{self, COLON_LOST, COLON_WRONG, ID_LOST, ID_WRONG, LBRACE_LOST, LBRACE_WRONG, LITERAL_LOST, LITERAL_WRONG, LPAREN_LOST, LPAREN_WRONG, NUM_LOST, NUM_WRONG, RBRACE_LOST, RBRACE_WRONG, RBRACK_LOST, RPAREN_LOST, RPAREN_WRONG, SEMICON_LOST, SEMICON_WRONG, TYPE_LOST, TYPE_WRONG};
use crate::common::Tag::{self, CH, DEC, ID, INC, KW_WHILE, LBRACE, LEA, LPAREN, MUL, NOT, NUM, RPAREN, STR, SUB, KW_FOR, KW_DO, KW_IF, KW_SWITCH, KW_BREAK, SEMICON, KW_INT, KW_VOID, KW_CHAR, RBRACE, KW_CONTINUE, KW_RETURN, END, ASSIGN, KW_ELSE, KW_CASE, KW_DEFAULT, COLON, LBRACK, RBRACK, COMMA, OR, AND, GT, GE, LT, ADD, NEQU, EQU, LE, DIV};
use crate::lexer::Lexer;
use crate::scanner::Scanner;
use crate::symbol::{Fun, Var};
use crate::symtab::SymTab;
use crate::token::{Token, TokenType};

fn syn_error(scanner: &mut Scanner, code: usize, t: &TokenType)
{
    //语法错误信息串
    const SYN_ERROR_TABLE: [&str; 15] = ["类型", "标识符", "数组长度",
                                         "常量", "逗号", "分号", "=",
                                         "冒号", "while", "(", ")",
                                         "[", "]", "{", "}"];

    if code % 2 == 0 {
        println!("{}<第{}行>语法错误 : 在 {} 之前丢失 {} .", scanner.file_name(), scanner.line_num(),
                                                           t.to_string(), SYN_ERROR_TABLE[code / 2]);
    } else {
        println!("{}<第{}行>语法错误 : 在 {} 之前没有正确匹配 {} .", scanner.file_name(), scanner.line_num(),
                                                                   t.to_string(), SYN_ERROR_TABLE[code / 2]);
    }
}

pub struct Parser<'a> {
    lexer: &'a mut Lexer<'a>,
    look: TokenType,
    sym_tab: &'a mut SymTab,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(lexer: &'a mut Lexer<'a>, token_type: TokenType, sym_tab: &'a mut SymTab) -> Self {
        Parser {
            lexer,
            look: token_type,
            sym_tab,
        }
    }

    // <literal>			->	number|string|chara
    pub(crate) fn literal(&mut self) -> Option<Box<Var>> {
        if equal_tag(&self.look, NUM) || equal_tag(&self.look, STR) || equal_tag(&self.look, CH) {
            let v = Box::new(Var::new_const(&self.look));
            if equal_tag(&self.look, STR) {
                self.sym_tab.add_str(v.clone());        // 字符串常量记录
            } else {
                self.sym_tab.add_var(v.clone());        // 其他常量记录
            }

            Some(v)
        } else {
            self.recovery(rval_opr(&self.look), LITERAL_LOST, LITERAL_WRONG);
            None
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

    fn recovery(&mut self, cond: bool, lost: SynError, wrong: SynError) {
        if cond {
            syn_error(self.lexer.get_scanner(), lost as usize, &self.look);
        } else {
            syn_error(self.lexer.get_scanner(), wrong as usize, &self.look);
            self.move_token();
        }
    }

    fn analyze(&mut self) {
        self.move_token();
        self.program();
    }

    fn segment(&self) {

    }

    fn program(&mut self) {
        if equal_tag(&self.look, END) {
            return;
        } else {
            self.segment();
            self.program();
        }
    }

    fn block(&mut self) {

    }

    fn altexpr(&mut self) {

    }

    fn expr(&mut self) -> Option<Box<Var>> {
        self.assexpr()
    }

    fn assexpr(&mut self) -> Option<Box<Var>> {
        let lval = self.orexpr();
        self.asstail(lval)
    }

    fn asstail(&mut self, lval: Option<Box<Var>>) -> Option<Box<Var>> {
        if self.match_tag(ASSIGN) {

            // self.asstail()
        }
        lval
    }

    fn orexpr(&self) -> Option<Box<Var>> {
        None
    }
}

impl<'a> Parser<'a> {

    fn statement(&mut self) {
        match self.look.get_tag() {
            KW_WHILE => self.while_stat(),
            KW_FOR => self.for_stat(),
            KW_DO => self.do_while_stat(),
            KW_IF => self.if_stat(),
            KW_SWITCH => self.switch_stat(),
            KW_BREAK => {
                self.move_token();
                if !self.match_tag(SEMICON) {
                    self.recovery(type_first(&self.look) || statement_first(&self.look) || equal_tag(&self.look, RBRACE), SEMICON_LOST, SEMICON_WRONG);
                }
            },
            KW_CONTINUE => {
                self.move_token();
                if !self.match_tag(SEMICON) {
                    self.recovery(type_first(&self.look)|| statement_first(&self.look) || equal_tag(&self.look, RBRACE), SEMICON_LOST, SEMICON_WRONG);
                }
            },
            KW_RETURN => {
                self.move_token();
                if !self.match_tag(SEMICON) {
                    self.recovery(type_first(&self.look)|| statement_first(&self.look) || equal_tag(&self.look, RBRACE), SEMICON_LOST, SEMICON_WRONG);
                }
            },
            _ => {
                if !self.match_tag(SEMICON) {
                    self.recovery(type_first(&self.look)|| statement_first(&self.look) || equal_tag(&self.look, RBRACE), SEMICON_LOST, SEMICON_WRONG);
                }
            }
        }
    }

    fn while_stat(&mut self) {
        self.sym_tab.enter();

        self.match_tag(KW_WHILE);
        if !self.match_tag(LPAREN) {
            self.recovery(expr_first(&self.look) || equal_tag(&self.look, RPAREN), TYPE_LOST, TYPE_WRONG);
        }

        self.altexpr();

        if !self.match_tag(RPAREN) {
            self.recovery(equal_tag(&self.look, LBRACE), RPAREN_LOST, RPAREN_WRONG);
        }

        // block
        if equal_tag(&self.look, LBRACE) {
            self.block();
        }

        self.sym_tab.leave();
    }

    fn for_stat(&mut self) {
        self.sym_tab.enter();

        if self.match_tag(KW_FOR) {
            if !self.match_tag(LPAREN) {
                self.recovery(type_first(&self.look) || expr_first(&self.look), LPAREN_LOST, LPAREN_WRONG);
            }

            self.for_init();

            if !self.match_tag(SEMICON) {
                self.recovery(expr_first(&self.look), SEMICON_LOST, SEMICON_WRONG);
            }

            self.altexpr();

            if !self.match_tag(RPAREN) {
                self.recovery(equal_tag(&self.look, LBRACE), RPAREN_LOST, RPAREN_WRONG);
            }

            if equal_tag(&self.look, LBRACE) {
                self.block();
            } else {
                self.statement();
            }
        }


        self.sym_tab.leave();
    }

    fn for_init(&self) {

    }

    fn do_while_stat(&mut self) {
        // 进入do作用域
        self.sym_tab.enter();

        self.match_tag(KW_DO);
        self.block();
        if !self.match_tag(KW_WHILE) {
            self.recovery(expr_first(&self.look) || equal_tag(&self.look, RPAREN), LPAREN_LOST, LPAREN_WRONG);
        }
        if !self.match_tag(LPAREN) {
            self.recovery(expr_first(&self.look) || equal_tag(&self.look, RPAREN), LPAREN_LOST, LPAREN_WRONG);
        }

        // 离开do作用域
        self.sym_tab.leave();

        self.altexpr();

        if !self.match_tag(RPAREN) {
            self.recovery(equal_tag(&self.look, SEMICON), RPAREN_LOST, RPAREN_WRONG);
        }
        if !self.match_tag(SEMICON) {
            self.recovery(type_first(&self.look) || statement_first(&self.look) || equal_tag(&self.look, RBRACE), SEMICON_LOST, SEMICON_WRONG);
        }
    }

    fn if_stat(&mut self) {
        self.sym_tab.enter();

        if self.match_tag(KW_IF) {
            if !self.match_tag(LPAREN) {
                self.recovery(expr_first(&self.look), LPAREN_LOST, LPAREN_WRONG);
            }

            self.altexpr();

            if !self.match_tag(RPAREN) {
                self.recovery(equal_tag(&self.look, LBRACE), RPAREN_LOST,RPAREN_WRONG);
            }
        }

        self.sym_tab.leave();
    }

    fn else_stat(&mut self) {
        if self.match_tag(KW_ELSE) {
            self.sym_tab.enter();

            if equal_tag(&self.look, LBRACE) {
                self.block();
            } else {
                self.statement();
            }

            self.sym_tab.leave();
        }
    }

    fn switch_stat(&mut self) {
        self.sym_tab.enter();

        if self.match_tag(KW_SWITCH) {
            if !self.match_tag(LPAREN) {
                self.recovery(expr_first(&self.look), RPAREN_LOST, RPAREN_WRONG);
            }

            if !self.match_tag(RPAREN) {
                self.recovery(equal_tag(&self.look, LBRACE), RPAREN_LOST, RPAREN_WRONG);
            }

            if !self.match_tag(LBRACE) {
                self.recovery(equal_tag(&self.look, KW_CASE) || equal_tag(&self.look, KW_DEFAULT), LBRACE_LOST, LBRACE_WRONG);
            }

            if !self.match_tag(RBRACE) {
                self.recovery(type_first(&self.look) || statement_first(&self.look), RBRACE_LOST, RBRACE_WRONG);
            }
        }

        self.sym_tab.leave();
    }

    fn case_stat(&mut self) {
        if self.match_tag(KW_CASE) {

        } else if self.match_tag(KW_DEFAULT) {
            if !self.match_tag(COLON) {
                self.recovery(type_first(&self.look) || statement_first(&self.look), COLON_LOST, COLON_WRONG);
            }
            self.sym_tab.enter();
            // subprogram
            self.sym_tab.leave();
        }
    }
}

impl<'a> Parser<'a> {

    fn init(&mut self, ext: bool, t: Tag, ptr: bool, name: String) -> Box<Var>{
        let mut init_val: Option<Box<Var>> = None;
        if self.match_tag(ASSIGN) {
            init_val = self.expr();
        }

        // 新的变量活指针
        Box::new(Var::new_pointer(self.sym_tab.get_scope_path(), ext, t, ptr, name, init_val))
    }

    fn varrdef(&mut self, ext: bool, t: Tag, ptr: bool, name: String) -> Box<Var>{
        if self.match_tag(LBRACK) {
            let mut len = 0;
            if self.match_tag(NUM) {
                if let TokenType::Num(num) = self.look.borrow() {
                    len = num.get_val();
                }

                self.move_token();
            } else {
                self.recovery(equal_tag(&self.look, RBRACK), NUM_LOST, NUM_WRONG);
            }


            if !self.match_tag(RBRACK) {
                self.recovery(equal_tag(&self.look, COMMA) || equal_tag(&self.look, SEMICON), RBRACK_LOST, RBRACE_WRONG);
            }

            // 新的数组
            Box::new(Var::new_array(self.sym_tab.get_scope_path(), ext, t, name, len))
        } else {
            self.init(ext, t, ptr, name)
        }
    }

    /*
	<defdata>			->	ident <varrdef>|mul ident <init>
    */
    fn defdata(&mut self, ext: bool, t: Tag) -> Box<Var> {
        let mut name = String::new();

        if equal_tag(&self.look, ID) {
            if let TokenType::Id(id) = self.look.borrow() {
                name = id.get_name();
                self.move_token();
            }
            self.varrdef(ext, t, false, name)
        } else if self.match_tag(MUL) {
            if equal_tag(&self.look, ID) {
                if let TokenType::Id(id) = self.look.borrow() {
                    name = id.get_name();
                    self.move_token();
                }
            } else {
                self.recovery(equal_tag(&self.look, SEMICON) || equal_tag(&self.look, COMMA) || equal_tag(&self.look, ASSIGN), ID_LOST, ID_WRONG);
            }
            self.init(ext, t, true, name)
        } else {
            self.recovery(equal_tag(&self.look, SEMICON) || equal_tag(&self.look, COMMA) || equal_tag(&self.look, ASSIGN) || equal_tag(&self.look, LBRACK), ID_LOST, ID_WRONG);

            self.varrdef(ext, t, false, name)
        }
    }

    fn deflist(&mut self, ext: bool, t: Tag) {

    }

    /*
	<idtail>			->	<varrdef><deflist>|lparen <para> rparen <funtail>
    */
    fn idtail(&mut self, ext: bool, t: Tag, name: String) {

        if self.match_tag(LPAREN) {     // 函数
            // 进入作用域
            self.sym_tab.enter();
            let mut para_list: Vec<Box<Var>> = Vec::new();
            self.para(&mut para_list);
            if !self.match_tag(RPAREN) {
                self.recovery(equal_tag(&self.look, LBRACK) || equal_tag(&self.look, SEMICON), RPAREN_LOST, RPAREN_WRONG);
            }
            let fun = Box::new(Fun::new(ext, t, name, para_list));
            self.fun_tail(fun);
            // 离开作用域
            self.sym_tab.leave();
        } else {
            self.sym_tab.add_var(self.varrdef(ext, t, false, name));
            self.deflist(ext, t);
        }
    }

    /*
	    <funtail>			->	<block>|semicon
    */
    fn fun_tail(&mut self, f: Box<Fun>) {
        if self.match_tag(SEMICON) {    // 函数声明
            // self.sym_tab;
        } else {    // 函数定义

            self.block();
        }
    }

    /*
	    <para>				->	<type><paradata><paralist>|^
    */
    fn para(&mut self, para_list: &mut Vec<Box<Var>>) {
        if !equal_tag(&self.look, RPAREN) {
            let t  = self.para_type();
            let v = self.para_data(t);

            self.sym_tab.add_var(v.clone());
            para_list.push(v.clone());
            self.para_list(para_list);
        }
    }

    /*
	    <paralist>		->	comma<type><paradata><paralist>|^
    */
    fn para_list(&mut self, para_list: &mut Vec<Box<Var>>) {
        if self.match_tag(COMMA) {  // 下一个参数
            let t = self.para_type();
            let v = self.para_data(t);
            self.sym_tab.add_var(v.clone());
            para_list.push(v.clone());
            self.para_list(para_list);
        }
    }

    /*
	    <paradata>		->	mul ident|ident <paradatatail>
    */
    fn para_data(&mut self, t: Tag) -> Box<Var> {
        let mut name = String::new();

        return if self.match_tag(MUL) {
            if equal_tag(&self.look, ID) {
                if let TokenType::Id(id) = self.look.borrow() {
                    name = id.get_name();
                    self.move_token();
                }
            } else {
                self.recovery(equal_tag(&self.look, COMMA) || equal_tag(&self.look, RPAREN), ID_LOST, ID_WRONG);
            }

            Box::new(Var::new_pointer(self.sym_tab.get_scope_path(), false, t, true, name, None))
        } else if equal_tag(&self.look, ID) {
            if let TokenType::Id(id) = self.look.borrow() {
                name = id.get_name();
                self.move_token();
            }
            self.para_data_tail(t, name)
        } else {
            self.recovery(equal_tag(&self.look, COMMA) || equal_tag(&self.look, RPAREN) || equal_tag(&self.look, LBRACK), ID_LOST, ID_WRONG);
            Box::new(Var::new_pointer(self.sym_tab.get_scope_path(), false, t, false, name, None))
        }
    }

    /*
	    <paradatatail>->	lbrack rbrack|lbrack num rbrack|^
    */
    fn para_data_tail(&mut self, t: Tag, name: String) -> Box<Var> {
        if self.match_tag(LBRACK) {
            let mut len = 1;
            if equal_tag(&self.look, NUM) {
                if let TokenType::Num(num) = self.look.borrow() {
                    len = num.get_val();
                }
                self.move_token();
            }   // 可以没有指定长度
            if !self.match_tag(RBRACK) {
                self.recovery(equal_tag(&self.look, COMMA) || equal_tag(&self.look, RPAREN), RBRACK_LOST, RBRACE_WRONG);
            }
            return Box::new(Var::new_array(self.sym_tab.get_scope_path(), false, t, name, len));
        }

        Box::new(Var::new_pointer(self.sym_tab.get_scope_path(), false, t, false, name, None))
    }

    /*
	    <type>				->	rsv_int|rsv_char|rsv_bool|rsv_void
    */
    fn para_type(&mut self) -> Tag {
        let mut tmp: Tag = KW_INT;  // 默认类型

        if type_first(&self.look) {
            tmp = self.look.get_tag();
            self.move_token();
        } else {
            self.recovery(equal_tag(&self.look, ID) || equal_tag(&self.look, MUL), TYPE_LOST, TYPE_WRONG);
        }

        tmp
    }
}

// 语句
fn statement_first(look: &TokenType) -> bool {
    expr_first(look) || equal_tag(look, SEMICON) || equal_tag(look, KW_WHILE) || equal_tag(look, KW_FOR) ||
    equal_tag(look, KW_DO) || equal_tag(look, KW_IF) || equal_tag(look, KW_SWITCH) || equal_tag(look, KW_RETURN) ||
    equal_tag(look, KW_BREAK) || equal_tag(look, KW_CONTINUE)
}

// 类型
fn type_first(look: &TokenType) -> bool {
    equal_tag(look, KW_INT) || equal_tag(look, KW_CHAR) || equal_tag(look, KW_VOID)
}

// 表达式
fn expr_first(look: &TokenType) -> bool {

    equal_tag(look, LPAREN) || equal_tag(look, NUM) || equal_tag(look, CH) || equal_tag(look, STR) || equal_tag(look, ID) || equal_tag(look, NOT)
        || equal_tag(look, SUB) || equal_tag(look, LEA) || equal_tag(look, MUL) || equal_tag(look, INC) || equal_tag(look, DEC)
}

// 左值运算
fn lval_opr(look: &TokenType) -> bool {
    equal_tag(look, ASSIGN) || equal_tag(look, OR) || equal_tag(look, AND) || equal_tag(look, GT) || equal_tag(look, GE) || equal_tag(look, LT)
        || equal_tag(look, LE) || equal_tag(look, EQU) || equal_tag(look, NEQU) || equal_tag(look, ADD) || equal_tag(look, SUB) || equal_tag(look, MUL) || equal_tag(look, DIV)
}

fn rval_opr(look: &TokenType) -> bool {
    equal_tag(look, OR) || equal_tag(look, AND) || equal_tag(look, GT) || equal_tag(look, GE) || equal_tag(look, LT)
        || equal_tag(look, LE) || equal_tag(look, EQU) || equal_tag(look, NEQU) || equal_tag(look, ADD) || equal_tag(look, SUB) || equal_tag(look, MUL) || equal_tag(look, DIV)
}

fn equal_tag(look: &TokenType, tag: Tag) -> bool {
    look.get_tag() == tag
}
