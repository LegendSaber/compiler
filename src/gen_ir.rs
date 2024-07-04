use std::sync::mpsc::SyncSender;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::common::Operator::{OpAs, OpEntry, OpExit, OpGet, OpRet, OpRetv};
use crate::common::SemError::ReturnErr;
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
        let tmp = Box::new(Var::new_temp(self.sym_tab.get_scope_path(), val.clone()));   // 拷贝变量信息

        let t = tmp.clone();
        let mut inst;
        if val.is_ref() {
            // 中间代码tmp = *(val->ptr)
            inst = Box::new(InterInst::new_common(OpGet, t, val.get_pointer().unwrap(), None));
        } else {
            inst = Box::new(InterInst::new_common(OpAs, t, val, None));   // 中间代码tmp = val
        }

        self.sym_tab.add_inst(inst);

        Some(tmp)
    }
}
