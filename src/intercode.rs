
/*
	四元式类，定义了中间代码的指令的形式
*/
use crate::common::Operator;
use crate::gen_ir::GenIR;
use crate::symbol::{Fun, Var};

#[derive(Clone)]
pub(crate) struct InterInst {
    label: String,                    // 标签
    op: Operator,                     // 操作符
    result: Option<Box<Var>>,         // 运算结果
    arg1: Option<Box<Var>>,           // 参数1
    arg2: Option<Box<Var>>,           // 参数2
    fun: Option<Box<Fun>>,            // 函数
    target: Option<Box<InterInst>>,   // 跳转标号
}

impl InterInst {
    // 初始化
    fn init() -> Self{
        InterInst {
            label: String::from(""),
            op: Operator::OpNop,
            result: None,
            arg1: None,
            arg2: None,
            fun: None,
            target: None,
        }
    }

    // 一般运算指令
    pub(crate) fn new_common(op: Operator, rs: Box<Var>, arg1: Box<Var>, arg2: Option<Box<Var>>) -> Self {
        let mut inst = InterInst::init();

        inst.set_op(op);
        inst.set_result(Some(rs));
        inst.set_arg1(Some(arg1));
        inst.set_arg2(arg2);

        inst
    }

    // 函数调用指令
    pub(crate) fn new_call(op: Operator, fun: Box<Fun>, rs: Option<Box<Var>>) -> Self {
        let mut inst = InterInst::init();

        inst.set_op(op);
        inst.set_fun(fun);
        inst.set_result(rs);
        inst.set_arg2(None);

        inst
    }

    // 参数进栈指令
    pub(crate) fn new_param(op: Operator, arg1: Box<Var>) -> Self {
        let mut inst = InterInst::init();

        inst.set_op(op);
        inst.set_arg1(Some(arg1));
        inst.set_arg2(None);
        inst.set_result(None);

        inst
    }

    // 产生唯一标号
    pub(crate) fn new_label() -> Self {
        let mut inst = InterInst::init();

        inst.set_label(GenIR::gen_lb());

        inst
    }

    // 条件跳转指令
    pub(crate) fn new_jump(op: Operator, tar: Option<Box<InterInst>>, arg1: Option<Box<Var>>, arg2: Option<Box<Var>>) -> Self {
        let mut inst = InterInst::init();
        inst.set_op(op);
        inst.set_target(Some(tar));
        inst.set_arg1(arg1);
        inst.set_arg2(arg2);
        inst
    }
}

impl InterInst {
    pub(crate) fn set_op(&mut self, op: Operator) {
        self.op = op;
    }

    pub(crate) fn set_result(&mut self, rs: Option<Box<Var>>) {
        self.result = rs;
    }

    pub(crate) fn set_arg1(&mut self, arg1: Option<Box<Var>>) {
        self.arg1 = arg1;
    }

    pub(crate) fn set_arg2(&mut self, arg2: Option<Box<Var>>) {
        self.arg2 = arg2;
    }

    pub(crate) fn set_label(&mut self, label: String) {
        self.label = label;
    }
}

#[derive(Clone)]
pub(crate) struct InterCode {
    code: Vec<Box<InterInst>>,
}

impl InterCode {
    // 增加中间代码
    pub(crate) fn add_inst(&mut self, inst: Box<InterInst>) {
        self.code.push(inst);
    }
}

