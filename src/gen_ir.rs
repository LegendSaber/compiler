use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::common::Operator::{OpAdd, OpAnd, OpArg, OpAs, OpCall, OpDiv, OpEntry, OpEqu, OpExit, OpGe, OpGet, OpGt, OpJf, OpJmp, OpLe, OpLea, OpLt, OpMod, OpMul, OpNe, OpNeg, OpNot, OpOr, OpProc, OpRet, OpRetv, OpSet, OpSub};
use crate::common::SemError::{ArrTypeErr, AssignTypeErr, ExprIsBase, ExprIsVoid, ExprNotBase, ExprNotLeftVal, ReturnErr};
use crate::common::Tag;
use crate::common::Tag::{OR, AND, EQU, NEQU, ADD, SUB, GT, GE, LT, LE, MUL, DIV, MOD, LEA, INC, DEC, NOT};
use crate::common::Tag::{ASSIGN, KwInt, KwVoid};
use crate::intercode::InterInst;
use crate::symbol::{Fun, sem_error, Var};
use crate::symtab::SymTab;

lazy_static! {
    static ref COUNTER: Mutex<u32> = Mutex::new(0);
}

/*
	中间代码生成器
*/

#[derive(Clone)]
pub(crate) struct GenIR {
    sym_tab: Box<SymTab>,
}

impl GenIR {
    // 全局函数

    // 获取唯一名字的标签
    pub(crate) fn gen_lb() -> String {
        let mut data = COUNTER.lock().unwrap();
        *data += 1;
        let s = format!("{}.L", *data);
        s
    }

    // 检查类型是否可以转换
    pub(crate) fn type_check(&self, lval: Option<Box<Var>>, rval: Option<Box<Var>>) -> bool {
        if lval.is_none() || rval.is_none() {
            return false;
        }
        let lval = lval.unwrap();
        let rval = rval.unwrap();

        if lval.is_base() && rval.is_base() {
            return true;
        } else if !lval.is_base() && !rval.is_base() && lval.get_type() == rval.get_type() {
            return true;
        }

        false
    }
}

impl GenIR {

    // 函数调用
    pub(crate) fn gen_para(&mut self, arg: Box<Var>) {  // 参数传递语句
        let mut arg = arg.clone();
        if arg.is_ref() {
            arg = self.gen_assign(arg);
        }

        let inst = Box::new(InterInst::new_param(OpArg, arg));
        self.sym_tab.add_inst(inst);
    }
}

impl GenIR {
    // 产生特殊语句

    // 产生函数入口语句
    pub(crate) fn gen_fun_head(&mut self, mut function: Box<Fun>) {
        function.enter_scope();      // 进入函数作用域
        let inst = Box::new(InterInst::new_call(OpEntry, function.clone(), None));
        self.sym_tab.add_inst(inst);                    // 添加函数入口指令
        let inst = Box::new(InterInst::new_label());
        function.set_return_point(Some(inst));          // 创建函数返回点
    }

    // 产生函数出口语句
    pub(crate) fn gen_fun_tail(&mut self, mut function: Box<Fun>) {
        let inst = Box::new(InterInst::new_call(OpExit, function.clone(), None));
        self.sym_tab.add_inst(inst);        // 添加函数返回点，return的目的标号
        let inst = Box::new(InterInst::new_call(OpExit, function.clone(), None));
        self.sym_tab.add_inst(inst);        // 添加函数出口指令
        function.leave_scope();             // 退出函数作用域
    }

    // 产生return语句
    pub(crate) fn gen_return(&mut self, ret: Option<Box<Var>>) {
        if let Some(ret) = ret {
            let fun = self.sym_tab.get_cur_fun();
            if let Some(fun) = fun {
                if  (ret.is_base() && fun.get_type() == KwVoid) || (ret.is_void() && fun.get_type() != KwVoid) {
                    sem_error(ReturnErr as usize, "");
                    return;
                }

                let return_point = fun.get_return_point();  // 获取返回点
                if ret.is_void() {
                    let inst = Box::new(InterInst::new_jump(OpRet, return_point, None, None));
                    self.sym_tab.add_inst(inst);
                } else {
                    if ret.is_ref() {
                        // 处理ret是*p的情况
                        let r = self.gen_assign(ret.clone());
                        let inst = Box::new(InterInst::new_jump(OpRetv, return_point, Some(r), None));
                        self.sym_tab.add_inst(inst);
                    }
                }
            }
        }
    }

    // 函数调用语句
    pub(crate) fn gen_call(&mut self, fun: Option<Box<Fun>>, args: Vec<Box<Var>>) -> Option<Box<Var>> {
        if fun.is_none() {
            return None;
        }

        for i in (0..args.len()).rev() {
            // 逆向传递实际参数
            self.gen_para(args[i].clone());
        }

        let fun = fun.unwrap();
        if fun.get_type() == KwVoid {
            // 中间代码fun()
            let inst = Box::new(InterInst::new_call(OpProc, fun.clone(), None));
            self.sym_tab.add_inst(inst);

            Var::get_void()
        } else {
            let ret = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), fun.get_type(), false));
            let inst = Box::new(InterInst::new_call(OpCall, fun.clone(), Some(ret.clone())));
            self.sym_tab.add_inst(inst);
            self.sym_tab.add_var(ret.clone());
            Some(ret)
        }
    }

}

impl GenIR {
    /* 产生符号和语句 */

    // 双目运算语句
    pub(crate) fn gen_two_op(&mut self, lval: Option<Box<Var>>, opt: Tag, rval: Option<Box<Var>>) -> Option<Box<Var>> {
        if lval.is_none() || rval.is_none() {
            return None;
        }
        let mut lval = lval.unwrap();
        let mut rval = rval.unwrap();

        if lval.is_void() || rval.is_void() {
            sem_error(ExprIsVoid as usize, "");     // void函数返回值不能出现在表达式中
            return None;
        }

        if !lval.is_base() || !rval.is_base() {
            sem_error(ExprNotBase as usize, "");    // 不是基本类型
            return Some(lval);
        }

        // 赋值单独处理
        if opt == ASSIGN {
            return Some(self.gen_assign_stmt(lval.clone(), rval.clone()));
        }

        // 先处理(*p)变量
        if lval.is_ref() {
            lval = self.gen_assign(lval.clone());
        }

        if rval.is_ref() {
            rval = self.gen_assign(rval.clone());
        }

        let ret = match opt {
            // 或
            OR  => self.gen_or(lval.clone(), rval.clone()),
            // 与
            AND => self.gen_and(lval.clone(), rval.clone()),
            // 等于
            EQU => self.gen_equ(lval.clone(), rval.clone()),
            // 不等于
            NEQU => self.gen_nequ(lval.clone(), rval.clone()),
            // 加
            ADD => self.gen_add(lval.clone(), rval.clone()),
            // 减
            SUB => self.gen_sub(lval.clone(), rval.clone()),
            // 大于
            GT => self.gen_gt(lval.clone(), rval.clone()),
            // 大于等于
            GE => self.gen_gt(lval.clone(), rval.clone()),
            // 小于
            LT => self.gen_lt(lval.clone(), rval.clone()),
            // 小于等于
            LE => self.gen_le(lval.clone(), rval.clone()),
            // 乘
            MUL => self.gen_mul(lval.clone(), rval.clone()),
            // 除
            DIV => self.gen_div(lval.clone(), rval.clone()),
            // 取模
            MOD => self.gen_mod(lval.clone(), rval.clone()),
            // 默认返回左值
            _ => lval,
        };

        Some(ret)
    }

    // 左单目运算语句
    pub(crate) fn get_one_op_left(&mut self, opt: Tag, val: Option<Box<Var>>) -> Option<Box<Var>> {
        if val.is_none() {
            return None;
        }

        let val = val.clone().unwrap();
        if val.is_void() {
            sem_error(ExprIsVoid as usize, "");
            return None;
        }

        if val.is_ref() {
            return Some(self.gen_assign(val.clone()));
        }

        let ret = match opt {
            LEA => self.gen_lea(val.clone()),
            MUL => Some(self.gen_ptr(val.clone())),
            INC => self.gen_incl(val.clone()),
            DEC => self.gen_decl(val.clone()),
            NOT => self.gen_not(val.clone()),
            SUB => self.gen_minus(val.clone()),
            _ => Some(val.clone()),
        };

        ret
    }

    // 右单目运算语句
    pub(crate) fn get_one_op_right(&mut self, opt: Tag, val: Option<Box<Var>>) -> Option<Box<Var>> {
        if val.is_none() {
            return None;
        }

        let val = val.unwrap();

        if val.is_void() {
            sem_error(ExprIsVoid as usize, "");
            return None;
        }

        if !val.get_left() {
            sem_error(ExprNotLeftVal as usize, "");
            return Some(val.clone());
        }

        let mut ret = Some(val.clone());
        if opt == INC {
            ret = self.gen_incr(val.clone());
        } else if opt == DEC {
            ret = self.gen_decr(val.clone());
        }

        ret
    }


    // 数组索引语句
    pub(crate) fn gen_array(&mut self, array: Option<Box<Var>>, index: Option<Box<Var>>) -> Option<Box<Var>> {
        if array.is_none() || index.is_none() {
            return None;
        }

        let array = array.unwrap();
        let index = index.unwrap();

        if array.is_base() || !index.is_base() {
            sem_error(ArrTypeErr as usize, "");
            return Some(index);
        }

        let v = self.gen_add(array, index);
        let ret = self.gen_ptr(v);

        Some(ret)
    }
}

impl GenIR {
    /* 双目运算 */

    // 拷贝赋值语句，处理*p的情况
    pub(crate) fn gen_assign(&mut self, val: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_copy_temp(self.sym_tab.get_scope_path(), val.clone()));   // 拷贝变量信息

        let t = tmp.clone();
        let inst;
        if val.is_ref() {
            // 中间代码tmp = *(val->ptr)
            inst = Box::new(InterInst::new_common(OpGet, t, val.get_pointer(), None));
        } else {
            inst = Box::new(InterInst::new_common(OpAs, t, Some(val), None));   // 中间代码tmp = val
        }

        self.sym_tab.add_inst(inst);

        tmp
    }

    // 赋值语句
    pub(crate) fn gen_assign_stmt(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        // 被赋值对象必须是左值
        if !lval.get_left() {
            sem_error(ExprNotLeftVal as usize, "");
            return rval;
        }

        if !self.type_check(Some(lval.clone()), Some(rval.clone())) {
            sem_error(AssignTypeErr as usize, "");
            return rval;
        }

        let r;
        // 考虑右值*p
        if rval.is_ref() {
            if !lval.is_ref() {
                // 中间代码lval=*(rval->ptr)
                let inst = Box::new(InterInst::new_common(OpGet, lval.clone(), rval.get_pointer(), None));
                self.sym_tab.add_inst(inst);
                return lval;
            } else {
                // 中间代码*(lval->ptr)=*(rval->ptr),先处理右值
                r  = Some(self.gen_assign(rval.clone()));
            }
        }

        let inst ;
        // 赋值运算
        if lval.is_ref() {
            // 中间代码*(lval->ptr)=rval
            inst = Box::new(InterInst::new_common(OpSet, rval.clone(), lval.get_pointer(), None));
        } else {
            // 中间代码*(lval->ptr)=rval
            inst = Box::new(InterInst::new_common(OpAs, lval.clone(), Some(rval.clone()), None));
        }
        self.sym_tab.add_inst(inst);

        lval
    }

    // 或运算语句
    pub(crate) fn gen_or(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpOr, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 与运算符
    pub(crate) fn gen_and(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpAnd, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 大于运算符
    pub(crate) fn gen_gt(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpGt, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 大于等于运算符
    pub(crate) fn gen_ge(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpGe, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 小于运算符
    pub(crate) fn gen_lt(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpLt, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 小于等于运算符
    pub(crate) fn gen_le(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpLe, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 等于运算符
    pub(crate) fn gen_equ(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpEqu, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 不等于运算符
    pub(crate) fn gen_nequ(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpNe, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 加法运算符
    pub(crate) fn gen_add(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp;
        let mut lval = lval.clone();
        let mut rval = rval.clone();
        // 指针和数组只能和基本类型相加
        if (lval.get_array() || lval.get_ptr()) && rval.is_base() {
            tmp = Box::new(Var::new_copy_temp(self.sym_tab.get_scope_path(), lval.clone()));
            let step = Var::get_step(lval.clone());
            if let Some(s) = step {
                rval = self.gen_mul(rval, s);
            }
        } else if rval.is_base() && (rval.get_array() || rval.get_ptr()) {
            tmp = Box::new(Var::new_copy_temp(self.sym_tab.get_scope_path(), rval.clone()));
            let stop = Var::get_step(rval.clone());
            if let Some(s) = stop {
                lval = self.gen_mul(lval, s);
            }
        } else if lval.is_base() && rval.is_base() {
            // 基本类型
            tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));
        } else {
            sem_error(ExprNotBase as usize, "");
            tmp = lval.clone();
        }

        // 加法命令
        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpAdd, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 减法运算符
    pub(crate) fn gen_sub(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp;
        let lval = lval.clone();
        let mut rval = rval.clone();

        if !rval.is_base() {
            sem_error(ExprNotBase as usize, "");
            tmp = lval.clone();
        } else if lval.get_array() || lval.get_ptr() {
            // 指针和数组
            tmp = Box::new(Var::new_copy_temp(self.sym_tab.get_scope_path(), lval.clone()));
            let step = Var::get_step(lval.clone());
            if let Some(s) = step {
                rval = self.gen_mul(rval.clone(), s);
            }
        } else {
            // 基本类型
            tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));
        }

        // 减法命令
        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpSub, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 乘法运算符
    pub(crate) fn gen_mul(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), lval.get_type(), false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpMul, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 除法运算符
    pub(crate) fn gen_div(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), lval.get_type(), false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpDiv, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }

    // 模运算
    pub(crate) fn gen_mod(&mut self, lval: Box<Var>, rval: Box<Var>) -> Box<Var> {
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), lval.get_type(), false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpMod, tmp.clone(), Some(lval.clone()), Some(rval.clone())));
        self.sym_tab.add_inst(inst);

        tmp
    }
}

impl GenIR {
    /* 单目运算 */

    // 指针取值语句
    pub(crate) fn gen_ptr(&mut self, val: Box<Var>) -> Box<Var> {
        return if val.is_base() {
            sem_error(ExprIsBase as usize, "");
            val
        } else {
            let mut tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), val.get_type(), false));

            tmp.set_left(true);
            tmp.set_pointer(val.clone());

            self.sym_tab.add_var(tmp.clone());

            tmp
        }
    }

    // 取址语句
    pub(crate) fn gen_lea(&mut self, val: Box<Var>) -> Option<Box<Var>> {
        return if !val.get_left() {
            sem_error(ExprNotLeftVal as usize, ""); // 不能取地址
            Some(val)
        } else {
            return if val.is_ref() {
                val.get_pointer()
            } else {
                let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), val.get_type(), true));

                self.sym_tab.add_var(tmp.clone());

                let inst = Box::new(InterInst::new_common(OpLea, tmp.clone(), Some(val.clone()), None));
                self.sym_tab.add_inst(inst);

                Some(tmp)
            }
        }
    }

    // 取反
    pub(crate) fn gen_not(&mut self, val: Box<Var>) -> Option<Box<Var>> {
        // 生成整数
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpNot, tmp.clone(), Some(val.clone()), None));
        self.sym_tab.add_inst(inst);

        Some(tmp)
    }

    // 取负
    pub(crate) fn gen_minus(&mut self, val: Box<Var>) -> Option<Box<Var>> {
        if !val.is_base() {
            sem_error(ExprNotBase as usize, "");
            return Some(val.clone());
        }

        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), KwInt, false));

        self.sym_tab.add_var(tmp.clone());
        let inst = Box::new(InterInst::new_common(OpNeg, tmp.clone(), Some(val.clone()), None));
        self.sym_tab.add_inst(inst);

        Some(tmp)
    }

    // 左自加
    pub(crate) fn gen_incl(&mut self, val: Box<Var>) -> Option<Box<Var>> {
        if !val.get_left() {
            sem_error(ExprNotLeftVal as usize, "");
            return Some(val.clone());
        }

        if val.is_ref() {                                                       // ++*p
            let t1 = self.gen_assign(val.clone());                   // t1 = *p
            let step = Var::get_step(val.clone());
            if let Some(s) = step {
                let t2 = self.gen_add(t1, s);                        // t2 = t1 + 1
                return Some(self.gen_assign_stmt(val.clone(), t2.clone()));     // *p = t2
            }
        }

        let inst = Box::new(InterInst::new_common(OpAdd, val.clone(), Some(val.clone()), Var::get_step(val.clone())));
        self.sym_tab.add_inst(inst);

        Some(val.clone())
    }

    // 左自减
    pub(crate) fn gen_decl(&mut self, val: Box<Var>) -> Option<Box<Var>> {
        if !val.get_left() {
            sem_error(ExprNotLeftVal as usize, "");
            return Some(val.clone());
        }

        if val.is_ref() {                                                          // ++*p
            let t1 = self.gen_assign(val.clone());                      // t1 = *p
            let step = Var::get_step(val.clone());
            if let Some(s) = step {
                let t2 = self.gen_sub(t1, s);                           // t2 = t1 + 1
                return Some(self.gen_assign_stmt(val.clone(), t2.clone()));        // *p = t2
            }
        }

        Some(val.clone())
    }

    // 右自加
    pub(crate) fn gen_incr(&mut self, val: Box<Var>) -> Option<Box<Var>> {
        let tmp = self.gen_assign(val.clone());

        let inst = Box::new(InterInst::new_common(OpAdd, val.clone(), Some(val.clone()), Var::get_step(val.clone())));  // 中间代码val++
        self.sym_tab.add_inst(inst);

        Some(tmp)
    }

    // 右自减
    pub(crate) fn gen_decr(&mut self, val: Box<Var>) -> Option<Box<Var>> {
        let tmp = self.gen_assign(val.clone());

        let inst = Box::new(InterInst::new_common(OpSub, val.clone(), Some(val.clone()), Var::get_step(val.clone())));  // 中间代码val--
        self.sym_tab.add_inst(inst);

        Some(tmp)
    }

}

impl GenIR {
    /* 产生复合语句 */

    // 产生if头部
    pub(crate) fn gen_if_head(&mut self, cond: Option<Box<Var>>) -> Box<InterInst> {
        let _else = Box::new(InterInst::new_label());
        if let Some(cond) = cond {
            let mut cond = cond.clone();
            if cond.is_ref() {
                cond = self.gen_assign(cond);
            }

            let inst = Box::new(InterInst::new_jump(OpJf, Some(_else.clone()), Some(cond), None));
            self.sym_tab.add_inst(inst);
        }
        _else
    }

    // 产生if尾部
    pub(crate) fn gen_if_tail(&mut self, _else: Box<InterInst>) {
        self.sym_tab.add_inst(_else);
    }

    // 产生else头部
    pub(crate) fn gen_else_head(&mut self, _else: Box<InterInst>) -> Box<InterInst> {
        let _exit = Box::new(InterInst::new_label());
        let inst = Box::new(InterInst::new_jump(OpJmp, Some(_exit.clone()), None, None));
        self.sym_tab.add_inst(inst);
        self.sym_tab.add_inst(_else);
        _exit
    }

    // 产生else尾部
    pub(crate) fn gen_else_tail(&mut self, _exit: Box<InterInst>) {
        self.sym_tab.add_inst(_exit);
    }
}
