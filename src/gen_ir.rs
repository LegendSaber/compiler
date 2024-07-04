use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::common::Operator::{OpAs, OpEntry, OpExit, OpGet, OpLea, OpRet, OpRetv, OpSet};
use crate::common::SemError::{AssignTypeErr, ExprIsBase, ExprNotLeftVal, ReturnErr};
use crate::common::Tag::KwVoid;
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

    // 获取唯一名字的标签
    pub(crate) fn gen_lb() -> String {
        let mut data = COUNTER.lock().unwrap();
        *data += 1;
        let s = format!("{}.L", *data);
        s
    }

    // 检查类型是否可以转换
    pub(crate) fn type_check(&self, lval: Option<Box<Var>>, rval: Option<Box<Var>>) -> bool {
        if let None = lval {
            return false;
        }
        if let None = rval {
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
                        let inst = Box::new(InterInst::new_jump(OpRetv, return_point, r, None));
                        self.sym_tab.add_inst(inst);
                    }
                }
            }
        }
    }

    // 拷贝赋值语句，处理*p的情况
    pub(crate) fn gen_assign(&mut self, val: Box<Var>) -> Option<Box<Var>> {
        let tmp = Box::new(Var::new_copy_temp(self.sym_tab.get_scope_path(), val.clone()));   // 拷贝变量信息

        let t = tmp.clone();
        let mut inst;
        if val.is_ref() {
            // 中间代码tmp = *(val->ptr)
            inst = Box::new(InterInst::new_common(OpGet, t, val.get_pointer(), None));
        } else {
            inst = Box::new(InterInst::new_common(OpAs, t, Some(val), None));   // 中间代码tmp = val
        }

        self.sym_tab.add_inst(inst);

        Some(tmp)
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

        let mut r = None;
        // 考虑右值*p
        if rval.is_ref() {
            if !lval.is_ref() {
                // 中间代码lval=*(rval->ptr)
                let inst = Box::new(InterInst::new_common(OpGet, lval.clone(), rval.get_pointer(), None));
                self.sym_tab.add_inst(inst);
                return lval;
            } else {
                // 中间代码*(lval->ptr)=*(rval->ptr),先处理右值
                r  = self.gen_assign(rval.clone());
            }
        }

        let mut inst ;
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

    /*
	    指针取值语句
    */
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

    // 取值语句
    pub(crate) fn get_lea(&mut self, val: Box<Var>) -> Option<Box<Var>> {
        return if !val.get_left() {
            sem_error(ExprNotLeftVal as usize, ""); // 不能取地址
            Some(val)
        } else {
            return if val.is_ref() {
                val.get_pointer()
            } else {
                let mut tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), val.get_type(), true));

                self.sym_tab.add_var(tmp.clone());

                let inst = Box::new(InterInst::new_common(OpLea, tmp.clone(), Some(val.clone()), None));
                self.sym_tab.add_inst(inst);

                Some(tmp)
            }
        }
    }
}
