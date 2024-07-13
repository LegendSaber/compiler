use crate::common::SynError::{self, ColonLost, ColonWrong, CommaLost, IdLost, IdWrong, LbraceLost, LbraceWrong, LparenLost, LparenWrong, NumLost, NumWrong, RbraceLost, RbraceWrong, RbrackLost, RparenLost, RparenWrong, SemiconLost, SemiconWrong, TypeLost, TypeWrong};
use crate::common::Tag::{self, CH, DEC, ID, INC, KwWhile, LBRACE, LEA, LPAREN, MUL, NOT, NUM, RPAREN, STR, SUB, KwFor, KwDo, KwIf, KwSwitch, KwBreak, SEMICON, KwInt, KwVoid, KwChar, RBRACE, KwContinue, KwReturn, END, ASSIGN, KwElse, KwCase, KwDefault, COLON, LBRACK, RBRACK, COMMA, OR, AND, GT, GE, LT, ADD, NEQU, EQU, LE, DIV, KwExtern, MOD};
use crate::gen_ir::GenIR;
use crate::lexer::Lexer;
use crate::scanner::Scanner;
use crate::symbol::{Fun, Var};
use crate::symtab::SymTab;
use crate::token::{Num, TokenType};

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
    ir: Option<Box<GenIR>>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(lexer: &'a mut Lexer<'a>, sym_tab: &'a mut SymTab, ir: Option<Box<GenIR>>) -> Self {
        Parser {
            lexer,
            look: TokenType::Num(Num::new(1900)),
            sym_tab,
            ir,
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

    // 语法分析主程序
    pub(crate) fn analyze(&mut self) {
        self.move_token();      // 预先读入
        self.program();
    }

    fn program(&mut self) {
        if equal_tag(&self.look, END) {
            return;
        } else {
            self.segment();
            self.program();
        }
    }

    /*
        <segment>			->	rsv_extern <type><def>|<type><def>
    */
    fn segment(&mut self) {
        let ext = self.match_tag(KwExtern);
        let t = self.var_type();
        self.def(ext, t);
    }


    /*
	    <def>					->	mul id <init><deflist>|ident <idtail>
    */
    fn def(&mut self, ext: bool, t: Tag) {
        let mut name = String::new();
        if self.match_tag(MUL) {    // 指针
            if equal_tag(&self.look, ID) {
                if let TokenType::Id(id) = &self.look {
                    name = id.get_name();
                    self.move_token();
                }
            } else {
                self.recovery(equal_tag(&self.look, SEMICON) || equal_tag(&self.look, COMMA) || equal_tag(&self.look, ASSIGN), IdLost, IdWrong);
            }

            let v = self.init(ext, t, true, name);
            self.sym_tab.add_var(v);
            self.deflist(ext, t);
        } else {
            if equal_tag(&self.look, ID) {  // 变量、数组、函数
                if let TokenType::Id(id) = &self.look {
                    name = id.get_name();
                    self.move_token();
                }
            } else {
                self.recovery(equal_tag(&self.look, SEMICON) || equal_tag(&self.look, COMMA) || equal_tag(&self.look, ASSIGN) || equal_tag(&self.look, LPAREN) || equal_tag(&self.look, LBRACK), IdLost, IdWrong);
            }
            self.id_tail(ext, t, false, name);
        }
    }

    /*
        <type>				->	rsv_int|rsv_char|rsv_bool|rsv_void
    */
    fn var_type(&mut self) -> Tag {
        let mut tmp: Tag = KwInt;  // 默认类型

        if type_first(&self.look) {
            tmp = self.look.get_tag();
            self.move_token();
        } else {
            self.recovery(equal_tag(&self.look, ID) || equal_tag(&self.look, MUL), TypeLost, TypeWrong);
        }

        tmp
    }

    fn block(&mut self) {
        if !self.match_tag(LBRACE) {
            self.recovery(type_first(&self.look) || statement_first(&self.look) || equal_tag(&self.look, RBRACE), LbraceLost, LbraceWrong);
        }

        self.subprogram();

        if !self.match_tag(RBRACE) {
            self.recovery(type_first(&self.look) || statement_first(&self.look) || equal_tag(&self.look, KwExtern) || equal_tag(&self.look, KwElse) || equal_tag(&self.look, KwCase) || equal_tag(&self.look, KwDefault), RbraceLost, RbraceWrong);
        }
    }

    fn subprogram(&mut self) {
        if type_first(&self.look) { // 局部变量
            self.local_def();
            self.subprogram();
        } else if statement_first(&self.look) { // 语句
            self.statement();
            self.subprogram();
        }
    }

    /*
	    <localdef>		->	<type><defdata><deflist>
    */
    fn local_def(&mut self) {
        let t = self.var_type();
        let v = self.defdata(false, t);
        self.sym_tab.add_var(v);
        self.deflist(false, t);
    }
}

// 表达式
impl <'a> Parser<'a> {
    /*
    	<altexpr>			->	<expr>|^
    */
    fn alt_expr(&mut self) -> Option<Box<Var>> {
        return if expr_first(&self.look) {
            self.expr()
        } else {
            Var::get_void()     // 返回特殊void变量
        }
    }

    /*
	    <expr> 				-> 	<assexpr>
    */
    fn expr(&mut self) -> Option<Box<Var>> {
        self.ass_expr()
    }

    /*
	    <assexpr>			->	<orexpr><asstail>
    */
    fn ass_expr(&mut self) -> Option<Box<Var>> {
        let lval = self.or_expr();
        self.ass_tail(lval)
    }

    /*
	    <asstail>			->	assign<assexpr>|^
    */
    fn ass_tail(&mut self, lval: Option<Box<Var>>) -> Option<Box<Var>> {
        return if self.match_tag(ASSIGN) {
            self.ass_expr();
            self.ass_tail(lval)
        } else {
            lval
        }
    }

    /*
	    <orexpr> 			-> 	<andexpr><ortail>
    */
    fn or_expr(&mut self) -> Option<Box<Var>> {
        let lval = self.and_expr();
        self.or_tail(lval)
    }

    /*
	    <ortail> 			-> 	or <andexpr> <ortail>|^
    */
    fn or_tail(&mut self, lval: Option<Box<Var>>) -> Option<Box<Var>> {
        return if self.match_tag(OR) {
            let rval = self.and_expr();

            self.or_tail(rval)
        } else {
            lval
        }
    }

    /*
	    <andexpr> 		-> 	<cmpexpr><andtail>
    */
    fn and_expr(&mut self) -> Option<Box<Var>> {
        let lval = self.cmp_expr();
        self.and_tail(lval)
    }

    /*
	    <andtail> 		-> 	and <cmpexpr> <andtail>|^
    */
    fn and_tail(&mut self, lval: Option<Box<Var>>) -> Option<Box<Var>> {
        return if self.match_tag(AND) {
            let rval = self.cmp_expr();
            let mut ir = self.ir.clone().unwrap();
            let result = ir.gen_two_op(lval, AND, rval);

            self.and_tail(result)
        } else {
            lval
        }
    }

    /*
	    <cmpexpr>			->	<aloexpr><cmptail>
    */
    fn cmp_expr(&mut self) -> Option<Box<Var>> {
        let lval = self.alo_expr();
        self.cmp_tail(lval)
    }

    /*
	    <cmptail>			->	<cmps><aloexpr><cmptail>|^
    */
    fn cmp_tail(&mut self, lval: Option<Box<Var>>) -> Option<Box<Var>> {
        return if equal_tag(&self.look, GT) || equal_tag(&self.look, GE) || equal_tag(&self.look, LT) || equal_tag(&self.look, LE) || equal_tag(&self.look, EQU) || equal_tag(&self.look, NEQU) {
            let opt = self.cmps();
            let rval = self.alo_expr();
            let mut ir = self.ir.clone().unwrap();
            let result = ir.gen_two_op(lval, opt, rval);
            self.cmp_tail(result)
        } else {
            lval
        }
    }

    /*
	    <cmps>				->	gt|ge|ls|le|equ|nequ
    */
    fn cmps(&mut self) -> Tag {
        let opt = self.look.get_tag();
        self.move_token();
        opt
    }

    /*
	    <aloexpr>			->	<item><alotail>
    */
    fn alo_expr(&mut self) -> Option<Box<Var>> {
        let lval = self.item();
        self.alo_tail(lval)
    }

    /*
	    <alotail>			->	<adds><item><alotail>|^
    */
    fn alo_tail(&mut self, lval: Option<Box<Var>>) -> Option<Box<Var>> {
        return if equal_tag(&self.look, ADD) || equal_tag(&self.look, SUB) {
            let opt = self.adds();
            let rval = self.item();
            let mut ir = self.ir.clone().unwrap();

            let result = ir.gen_two_op(lval, opt, rval);
            self.alo_tail(result)
        } else {
            lval
        }
    }

    /*
	    <adds>				->	add|sub
    */
    fn adds(&mut self) -> Tag {
        let opt = self.look.get_tag();
        self.move_token();
        opt
    }

    /*
	    <item>				->	<factor><itemtail>
    */
    fn item(&mut self) -> Option<Box<Var>> {
        let lval = self.factor();
        self.item_tail(lval)
    }

    /*
	    <itemtail>		->	<muls><factor><itemtail>|^
    */
    fn item_tail(&mut self, lval: Option<Box<Var>>) -> Option<Box<Var>> {
        return if equal_tag(&self.look, MUL) || equal_tag(&self.look, DIV) || equal_tag(&self.look, MOD) {
            let opt = self.muls();
            let rval = self.factor();
            let mut ir = self.ir.clone().unwrap();
            let result = ir.gen_two_op(lval, opt, rval);

            self.item_tail(result)
        } else {
            lval
        }
    }

    /*
	    <muls>				->	mul|div|mod
    */
    fn muls(&mut self) -> Tag {
        let opt = self.look.get_tag();
        self.move_token();
        opt
    }

    /*
	    <factor> 			-> 	<lop><factor>|<val>
    */
    fn factor(&mut self) -> Option<Box<Var>> {
        return if equal_tag(&self.look, NOT) || equal_tag(&self.look, SUB) || equal_tag(&self.look, LEA) || equal_tag(&self.look, MUL) || equal_tag(&self.look, INC) || equal_tag(&self.look, DEC) {
            let opt = self.lop();
            let v = self.factor();
            let mut ir = self.ir.clone().unwrap();

            ir.gen_one_op_left(opt, v)
        } else {
            self.val()
        }
    }

    /*
	    <lop> 				-> 	not|sub|lea|mul|incr|decr
    */
    fn lop(&mut self) -> Tag {
        let opt = self.look.get_tag();
        self.move_token();
        opt
    }

    /*
	    <val>					->	<elem><rop>
    */
    fn val(&mut self) -> Option<Box<Var>> {
        let v = self.elem();
        return if equal_tag(&self.look, INC) || equal_tag(&self.look, DEC) {
            let opt = self.rop();
            let mut ir = self.ir.clone().unwrap();

            ir.gen_one_op_right(v, opt)
        } else {
            v
        }
    }

    /*
	    <rop>					->	incr|decr|^
    */
    fn rop(&mut self) -> Tag {
        let opt = self.look.get_tag();
        self.move_token();
        opt
    }

    /*
	    <elem>				->	ident<idexpr>|lparen<expr>rparen|<literal>
    */
    fn elem(&mut self) -> Option<Box<Var>> {
        let mut v = None;
        if equal_tag(&self.look, ID) {
            if let TokenType::Id(id) = &self.look {
                let name = id.get_name();
                self.move_token();
                v = self.id_expr(name);
            }
        } else if self.match_tag(LPAREN) {
            v = self.expr();
            if !self.match_tag(RPAREN) {
                self.recovery(lval_opr(&self.look), RparenLost, RparenWrong);
            }
        } else {
            // 常量
            v = self.literal();
        }

        v
    }

    /*
        <literal>			->	number|string|chara
    */
    fn literal(&mut self) -> Option<Box<Var>> {
        let mut ret_v = None;

        if equal_tag(&self.look, NUM) || equal_tag(&self.look, STR) || equal_tag(&self.look, CH) {
            let v = Box::new(Var::new_const(&self.look));
            ret_v = Some(v.clone());
            if equal_tag(&self.look, STR) {
                self.sym_tab.add_str(v);        // 字符串常量记录
            } else {
                self.sym_tab.add_var(v);        // 其他常量也记录到符号表
            }
            self.move_token();
        }

        ret_v
    }

    /*
	    <idexpr>			->	lbrack <expr> rbrack|lparen<realarg>rparen|^
    */
    fn id_expr(&mut self, name: String) -> Option<Box<Var>> {
        let v;
        if self.match_tag(LBRACK) {
            let index = self.expr();
            if !self.match_tag(RBRACK) {
                self.recovery(lval_opr(&self.look), LbraceLost, LbraceWrong);
            }
            let array = self.sym_tab.get_var(name);
            let mut ir = self.ir.clone().unwrap();
            v = ir.gen_array(array, index);
        } else if self.match_tag(LPAREN) {
            let mut args = Vec::new();
            self.real_arg(&mut args);
            if !self.match_tag(RPAREN) {
                self.recovery(rval_opr(&self.look), RparenLost, RparenWrong);
            }
            let function = self.sym_tab.get_fun(name, args.clone());
            let mut ir = self.ir.clone().unwrap();
            v = ir.gen_call(function, args);
        } else {
            v = self.sym_tab.get_var(name);
        }

        v
    }

    /*
	    <real_arg>			->	<arg><arglist>|^
    */
    fn real_arg(&mut self, args: &mut Vec<Box<Var>>) {
        if expr_first(&self.look) {
            if let Some(arg) = self.arg() {
                args.push(arg);
                self.args_list(args);
            }
        }
    }

    /*
	    <arglist>			->	comma<arg><arglist>|^
    */
    fn args_list(&mut self, args: &mut Vec<Box<Var>>) {
        if !self.match_tag(COMMA) {
            if let Some(arg) = self.arg() {
                args.push(arg);
                self.args_list(args);
            }
        }
    }

    /*
	    <arg> 				-> 	<expr>
    */
    fn arg(&mut self) -> Option<Box<Var>> {
        self.expr()
    }
}

// 语句
impl<'a> Parser<'a> {
    /*
	    <statement>		->	<altexpr>semicon
										|<whilestat>|<forstat>|<dowhilestat>
										|<ifstat>|<switchstat>
										|rsv_break semicon
										|rsv_continue semicon
										|rsv_return<altexpr>semicon
    */
    fn statement(&mut self) {
        match self.look.get_tag() {
            KwWhile => self.while_stat(),
            KwFor => self.for_stat(),
            KwDo => self.do_while_stat(),
            KwIf => self.if_stat(),
            KwSwitch => self.switch_stat(),
            KwBreak => {
                if let Some(mut ir) = self.ir.clone() {
                    ir.gen_break();
                }
                self.move_token();
                if !self.match_tag(SEMICON) {
                    self.recovery(type_first(&self.look) || statement_first(&self.look) || equal_tag(&self.look, RBRACE), SemiconLost, SemiconWrong);
                }
            },
            KwContinue => {
                if let Some(mut ir) = self.ir.clone() {
                    ir.gen_continue();
                }
                self.move_token();
                if !self.match_tag(SEMICON) {
                    self.recovery(type_first(&self.look)|| statement_first(&self.look) || equal_tag(&self.look, RBRACE), SemiconLost, SemiconWrong);
                }
            },
            KwReturn => {
                self.move_token();
                if let Some(mut ir) = self.ir.clone() {
                    ir.gen_return(self.alt_expr()); // 产生return语句
                }
                if !self.match_tag(SEMICON) {
                    self.recovery(type_first(&self.look)|| statement_first(&self.look) || equal_tag(&self.look, RBRACE), SemiconLost, SemiconWrong);
                }
            },
            _ => {
                self.alt_expr();
                if !self.match_tag(SEMICON) {
                    self.recovery(type_first(&self.look)|| statement_first(&self.look) || equal_tag(&self.look, RBRACE), SemiconLost, SemiconWrong);
                }
            }
        }
    }

    /*
	    <whilestat>		->	rsv_while lparen<altexpr>rparen<block>
	    <block>				->	<block>|<statement>
    */
    fn while_stat(&mut self) {
        self.sym_tab.enter();

        let mut ir = self.ir.clone().unwrap();
        let (_while, _exit) = ir.gen_while_head();

        self.match_tag(KwWhile);
        if !self.match_tag(LPAREN) {
            self.recovery(expr_first(&self.look) || equal_tag(&self.look, RPAREN), TypeLost, TypeWrong);
        }

        let cond = self.alt_expr();
        ir.gen_while_cond(cond, Some(_exit.clone()));

        if !self.match_tag(RPAREN) {
            self.recovery(equal_tag(&self.look, LBRACE), RparenLost, RparenWrong);
        }

        // block
        if equal_tag(&self.look, LBRACE) {
            self.block();
        } else {
            self.statement();
        }

        ir.gen_while_tail(_while.clone(), _exit.clone());
        self.sym_tab.leave();
    }

    /*
	    <forstat> 		-> 	rsv_for lparen <forinit> semicon <altexpr> semicon <altexpr> rparen <block>
	    <block>				->	<block>|<statement>
    */
    fn for_stat(&mut self) {
        self.sym_tab.enter();

        let mut ir = self.ir.clone().unwrap();

        if self.match_tag(KwFor) {
            if !self.match_tag(LPAREN) {
                self.recovery(type_first(&self.look) || expr_first(&self.look), LparenLost, LparenWrong);
            }

            self.for_init();
            let (_for, _exit) = ir.gen_for_head();
            if !self.match_tag(SEMICON) {
                self.recovery(expr_first(&self.look), SemiconLost, SemiconWrong);
            }

            let cond = self.alt_expr();
            let (_block, _step) = ir.gen_for_cond_begin(cond, _exit.clone());

            if !self.match_tag(RPAREN) {
                self.recovery(equal_tag(&self.look, LBRACE), RparenLost, RparenWrong);
            }
            ir.gen_for_cond_end(_for, _block);
            if equal_tag(&self.look, LBRACE) {
                self.block();
            } else {
                self.statement();
            }
            ir.gen_for_tail(_step, _exit);
        }


        self.sym_tab.leave();
    }

    /*
	    <forinit> 		->  <localdef> | <altexpr>
    */
    fn for_init(&mut self) {
        if type_first(&self.look) {
            self.local_def();
        } else {
            self.alt_expr();
            if !self.match_tag(SEMICON) {
                self.recovery(expr_first(&self.look), SemiconLost, SemiconWrong);
            }
        }
    }

    /*
	    <dowhilestat> -> 	rsv_do <block> rsv_while lparen<altexpr>rparen semicon
	    <block>				->	<block>|<statement>
    */
    fn do_while_stat(&mut self) {
        // 进入do作用域
        self.sym_tab.enter();
        let mut ir = self.ir.clone().unwrap();
        let (_do, _exit) = ir.gen_do_while_head();
        self.match_tag(KwDo);
        if equal_tag(&self.look, LBRACE) {
            self.block();
        } else {
            self.statement();
        }

        if !self.match_tag(KwWhile) {
            self.recovery(expr_first(&self.look) || equal_tag(&self.look, RPAREN), LparenLost, LparenWrong);
        }
        if !self.match_tag(LPAREN) {
            self.recovery(expr_first(&self.look) || equal_tag(&self.look, RPAREN), LparenLost, LparenWrong);
        }

        // 离开do作用域
        self.sym_tab.leave();

        let cond = self.alt_expr().unwrap();

        if !self.match_tag(RPAREN) {
            self.recovery(equal_tag(&self.look, SEMICON), RparenLost, RparenWrong);
        }
        if !self.match_tag(SEMICON) {
            self.recovery(type_first(&self.look) || statement_first(&self.look) || equal_tag(&self.look, RBRACE), SemiconLost, SemiconWrong);
        }

        ir.gen_do_while_tail(Some(cond), _do, _exit);
    }

    /*
	    <ifstat>			->	rsv_if lparen<expr>rparen<block><elsestat>
    */
    fn if_stat(&mut self) {
        self.sym_tab.enter();

        let mut _else = None;
        if self.match_tag(KwIf) {
            if !self.match_tag(LPAREN) {
                self.recovery(expr_first(&self.look), LparenLost, LparenWrong);
            }

            let cond = self.expr();
            if let Some(mut ir) = self.ir.clone() {
                _else = Some(ir.gen_if_head(cond));
            }

            if !self.match_tag(RPAREN) {
                self.recovery(equal_tag(&self.look, LBRACE), RparenLost, RparenWrong);
            }
        }

        if equal_tag(&self.look, LBRACE) {
            self.block();
        } else {
            self.statement();
        }

        self.sym_tab.leave();

        if equal_tag(&self.look, KwElse) {
            let mut _exit = None;
            if let Some(mut ir) = self.ir.clone() {
                if let Some(_else) = _else {
                    _exit = Some(ir.gen_else_head(_else));      // 有else
                }
            }

            self.else_stat();

            if let Some(mut ir) = self.ir.clone() {
                if let Some(_exit) = _exit {
                    ir.gen_else_tail(_exit);      // 有else
                }
            }
        } else {
            if let Some(mut ir) = self.ir.clone() {
                if let Some(_else) = _else {
                    ir.gen_if_tail(_else);      // 无else
                }
            }
        }
    }

    /*
	    <elsestat>		-> 	rsv_else<block>|^
    */
    fn else_stat(&mut self) {
        if self.match_tag(KwElse) {
            self.sym_tab.enter();

            if equal_tag(&self.look, LBRACE) {
                self.block();
            } else {
                self.statement();
            }

            self.sym_tab.leave();
        }
    }

    /*
	    <switchstat>	-> 	rsv_switch lparen <expr> rparen lbrac <casestat> rbrac
    */
    fn switch_stat(&mut self) {
        self.sym_tab.enter();

        let mut _exit = None;

        if let Some(mut ir) = self.ir.clone() {
            _exit = Some(ir.gen_switch_head());
        }

        if self.match_tag(KwSwitch) {
            if !self.match_tag(LPAREN) {
                self.recovery(expr_first(&self.look), RparenLost, RparenWrong);
            }

            let mut cond = self.expr().unwrap();
            if let Some(mut ir) = self.ir.clone() {
                cond = ir.gen_assign(cond);
            }

            if !self.match_tag(RPAREN) {
                self.recovery(equal_tag(&self.look, LBRACE), RparenLost, RparenWrong);
            }

            if !self.match_tag(LBRACE) {
                self.recovery(equal_tag(&self.look, KwCase) || equal_tag(&self.look, KwDefault), LbraceLost, LbraceWrong);
            }

            self.case_stat(Some(cond));

            if !self.match_tag(RBRACE) {
                self.recovery(type_first(&self.look) || statement_first(&self.look), RbraceLost, RbraceWrong);
            }
        }

        if let Some(mut ir) = self.ir.clone() {
            if let Some(_exit) = _exit {
                ir.gen_switch_tail(_exit);
            }
        }

        self.sym_tab.leave();
    }

    /*
	    <casestat> 		-> 	rsv_case <caselabel> colon <subprogram><casestat>
										| rsv_default colon <subprogram>
    */
    fn case_stat(&mut self, cond: Option<Box<Var>>) {
        if self.match_tag(KwCase) {
            let mut _case_exit = None;
            let lb = self.case_label();
            if let Some(mut ir) = self.ir.clone() {
                _case_exit = Some(ir.gen_case_head(cond.clone(), lb));
            }
            if !self.match_tag(COLON) {
                self.recovery(type_first(&self.look) || statement_first(&self.look), ColonLost, ColonWrong);
            }
            self.sym_tab.enter();
            self.subprogram();
            self.sym_tab.leave();
            if let Some(mut ir) = self.ir.clone() {
                if let Some(_case_exit) = _case_exit {
                    ir.gen_case_tail(_case_exit);
                }
            }
            self.case_stat(cond);
        } else if self.match_tag(KwDefault) {
            if !self.match_tag(COLON) {
                self.recovery(type_first(&self.look) || statement_first(&self.look), ColonLost, ColonWrong);
            }
            self.sym_tab.enter();
            self.subprogram();
            self.sym_tab.leave();
        }
    }

    /*
	    <caselabel>		->	<literal>
    */
    fn case_label(&mut self) -> Option<Box<Var>> {
        self.literal()
    }
}

// 声明与定义
impl<'a> Parser<'a> {
    /*
	    <init>				->	assign <expr>|^
    */
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
                if let TokenType::Num(num) = &self.look {
                    len = num.get_val();
                }

                self.move_token();
            } else {
                self.recovery(equal_tag(&self.look, RBRACK), NumLost, NumWrong);
            }


            if !self.match_tag(RBRACK) {
                self.recovery(equal_tag(&self.look, COMMA) || equal_tag(&self.look, SEMICON), RbrackLost, RbraceWrong);
            }

            // 新的数组
            Box::new(Var::new_array(self.sym_tab.get_scope_path(), ext, t, name, len))
        } else {
            self.init(ext, t, ptr, name)
        }
    }

    /*
	    <deflist>			->	comma <defdata> <deflist>| semicon
    */
    fn deflist(&mut self, ext: bool, t: Tag) {
        if self.match_tag(COMMA) {  // 下一个声明
            let v = self.defdata(ext, t);
            self.sym_tab.add_var(v);
            self.deflist(ext, t);
        } else if !self.match_tag(SEMICON) {
            // 出错了
            // 不是最后一个声明
            if equal_tag(&self.look, ID) || equal_tag(&self.look, MUL) {
                self.recovery(true, CommaLost, ColonWrong);
                let v = self.defdata(ext, t);
                self.sym_tab.add_var(v);
                self.deflist(ext, t);
            } else {
                self.recovery(type_first(&self.look) || statement_first(&self.look) || equal_tag(&self.look, KwExtern) || equal_tag(&self.look, RBRACK),
                             SemiconLost, SemiconWrong);
            }
        }
    }

    /*
	    <defdata>			->	ident <varrdef>|mul ident <init>
    */
    fn defdata(&mut self, ext: bool, t: Tag) -> Box<Var> {
        let mut name = String::new();

        if equal_tag(&self.look, ID) {
            if let TokenType::Id(id) = &self.look {
                name = id.get_name();
                self.move_token();
            }
            self.varrdef(ext, t, false, name)
        } else if self.match_tag(MUL) {
            if equal_tag(&self.look, ID) {
                if let TokenType::Id(id) = &self.look {
                    name = id.get_name();
                    self.move_token();
                }
            } else {
                self.recovery(equal_tag(&self.look, SEMICON) || equal_tag(&self.look, COMMA) || equal_tag(&self.look, ASSIGN), IdLost, IdWrong);
            }
            self.init(ext, t, true, name)
        } else {
            self.recovery(equal_tag(&self.look, SEMICON) || equal_tag(&self.look, COMMA) || equal_tag(&self.look, ASSIGN) || equal_tag(&self.look, LBRACK), IdLost, IdWrong);

            self.varrdef(ext, t, false, name)
        }
    }

    /*
	    <idtail>			->	<varrdef><deflist>|lparen <para> rparen <funtail>
    */
    fn id_tail(&mut self, ext: bool, t: Tag, ptr: bool, name: String) {

        if self.match_tag(LPAREN) {     // 函数
            // 进入作用域
            self.sym_tab.enter();
            let mut para_list: Vec<Box<Var>> = Vec::new();
            self.para(&mut para_list);
            if !self.match_tag(RPAREN) {
                self.recovery(equal_tag(&self.look, LBRACK) || equal_tag(&self.look, SEMICON), RparenLost, RparenWrong);
            }
            let fun = Box::new(Fun::new(ext, t, name, para_list));
            self.fun_tail(fun);
            // 离开作用域
            self.sym_tab.leave();
        } else {
            let v = self.varrdef(ext, t, ptr, name);
            self.sym_tab.add_var(v);
            self.deflist(ext, t);
        }
    }
}

// 函数
impl<'a> Parser<'a> {
    /*
        <funtail>			->	<block>|semicon
    */
    fn fun_tail(&mut self, f: Box<Fun>) {
        if self.match_tag(SEMICON) {        // 函数声明
            self.sym_tab.dec_fun(f);
        } else {    // 函数定义
            self.sym_tab.def_fun(f);        // 函数定义
            self.block();
            self.sym_tab.end_def_fun();     // 结束函数定义
        }
    }

    /*
	    <para>				->	<type><paradata><paralist>|^
    */
    fn para(&mut self, para_list: &mut Vec<Box<Var>>) {
        if !equal_tag(&self.look, RPAREN) {
            let t  = self.var_type();
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
            let t = self.var_type();
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
                if let TokenType::Id(id) = &self.look {
                    name = id.get_name();
                    self.move_token();
                }
            } else {
                self.recovery(equal_tag(&self.look, COMMA) || equal_tag(&self.look, RPAREN), IdLost, IdWrong);
            }

            Box::new(Var::new_pointer(self.sym_tab.get_scope_path(), false, t, true, name, None))
        } else if equal_tag(&self.look, ID) {
            if let TokenType::Id(id) = &self.look {
                name = id.get_name();
                self.move_token();
            }
            self.para_data_tail(t, name)
        } else {
            self.recovery(equal_tag(&self.look, COMMA) || equal_tag(&self.look, RPAREN) || equal_tag(&self.look, LBRACK), IdLost, IdWrong);
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
                if let TokenType::Num(num) = &self.look {
                    len = num.get_val();
                }
                self.move_token();
            }   // 可以没有指定长度
            if !self.match_tag(RBRACK) {
                self.recovery(equal_tag(&self.look, COMMA) || equal_tag(&self.look, RPAREN), RbrackLost, RbraceWrong);
            }
            return Box::new(Var::new_array(self.sym_tab.get_scope_path(), false, t, name, len));
        }

        Box::new(Var::new_pointer(self.sym_tab.get_scope_path(), false, t, false, name, None))
    }
}

// 语句
fn statement_first(look: &TokenType) -> bool {
    expr_first(look) || equal_tag(look, SEMICON) || equal_tag(look, KwWhile) || equal_tag(look, KwFor) ||
    equal_tag(look, KwDo) || equal_tag(look, KwIf) || equal_tag(look, KwSwitch) || equal_tag(look, KwReturn) ||
    equal_tag(look, KwBreak) || equal_tag(look, KwContinue)
}

// 类型
fn type_first(look: &TokenType) -> bool {
    equal_tag(look, KwInt) || equal_tag(look, KwChar) || equal_tag(look, KwVoid)
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
