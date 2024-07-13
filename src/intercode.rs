
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
    pub(crate) fn new_common(op: Operator, rs: Box<Var>, arg1: Option<Box<Var>>, arg2: Option<Box<Var>>) -> Self {
        let mut inst = InterInst::init();

        inst.set_op(op);
        inst.set_result(Some(rs));
        inst.set_arg1(arg1);
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
        inst.set_target(tar);
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

    pub(crate) fn set_fun(&mut self, fun: Box<Fun>) {
        self.fun = Some(fun.clone());
    }

    pub(crate) fn get_fun(&self) -> Option<Box<Fun>> {
        self.fun.clone()
    }

    pub(crate) fn set_target(&mut self, target: Option<Box<InterInst>>) {
        self.target = target;
    }

    pub(crate) fn get_label(&self) -> String {
        self.label.clone()
    }
}

impl InterInst {
    pub(crate) fn load_var(&self, reg32: String, reg8: String, var: Option<Box<Var>>) {
        if var.is_none() {
            return;
        }

        let var = var.unwrap();
        let reg = if var.is_char() {
            reg8.clone()
        } else {
            reg32.clone()
        };

        if var.is_char() {
            println!("mov {}, 0", reg32.clone());
        }

        let name = var.get_name();
        if var.not_const() {
            let offset = var.get_offset();

            if offset == 0 {
                if !var.get_array() {
                    println!("mov {}, [{}]", reg, name);
                } else {
                    println!("mov {}, {}", reg, name);
                }
            } else {
                if !var.get_array() {
                    println!("mov {}, [ebp + {}]", reg, offset);
                } else {
                    println!("mov {}, [ebp + {}]", reg, offset);
                }
            }
        } else {        // 常量
            if var.is_base() {
                println!("mov {}, {}", reg, var.get_val())
            } else {
                println!("mov {}, {}", reg, name);
            }
        }
    }

    pub(crate) fn lea_var(&self, reg: String, var: Option<Box<Var>>) {
        if var.is_none() {
            return;
        }

        let var = var.unwrap();
        let name = var.get_name();
        let offset = var.get_offset();

        if offset == 0 {
            println!("mov {}, {}", reg, name);
        } else {
            println!("lea {}, [ebp + {}]", reg, offset);
        }
    }

    pub(crate) fn store_var(&self, reg32: String, reg8: String, var: Option<Box<Var>>) {
        if var.is_none() {
            return;
        }

        let var = var.unwrap();
        let reg = if var.is_char() {
            reg8
        } else {
            reg32
        };
        let name = var.get_name();
        let offset = var.get_offset();

        if offset == 0 {
            println!("mov [{}], {}", name, reg);
        } else {
            println!("mov [ebp + %{}], {}", offset, name);
        }
    }

    pub(crate) fn init_var(&self, var: Option<Box<Var>>) {
        if var.is_none() {
            return;
        }

        let var = var.unwrap();
        if !var.is_un_init() {
            println!("mov eax, {}", var.get_val());
        } else {
            println!("mov eax, {}", var.get_ptr_val());
        }
    }

    pub(crate) fn to_x86(&self) {
        if self.label != "" {
            println!("{}:", self.label.as_str());
        }

        match self.op {
            Operator::OpNop => {
                println!("nop");
            },
            Operator::OpDec => {
                self.init_var(self.arg1.clone());
            },
            Operator::OpEntry => {
                println!("push ebp");
                println!("mov ebp, esp");
                let fun = self.get_fun().unwrap();
                println!("sub eps, {}", fun.get_max_depth());
            },
            Operator::OpExit => {
                println!("mov esp, ebp");
                println!("pop ebp");
                println!("ret");
            },
            Operator::OpAs => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.store_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpAdd => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("add eax, ebx");
                self.store_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpSub => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("sub eax, ebx");
                self.store_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpMul => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("imul ebx");
                self.store_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpDiv => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("idiv ebx");
                self.store_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpMod => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("idiv ebx");
                self.store_var("eax".to_string(), "dl".to_string(), self.result.clone());
            },
            Operator::OpNeg => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                println!("neg eax");
                self.store_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpGt => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("mov ecx, 0");
                println!("cmp eax, ebx");
                println!("setg cl");
                self.store_var("ecx".to_string(), "cl".to_string(), self.result.clone());
            },
            Operator::OpGe => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("mov ecx, 0");
                println!("cmp eax, ebx");
                println!("setge cl");
                self.store_var("ecx".to_string(), "cl".to_string(), self.result.clone());
            },
            Operator::OpLt => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("mov ecx, 0");
                println!("cmp eax, ebx");
                println!("setl cl");
                self.store_var("ecx".to_string(), "cl".to_string(), self.result.clone());
            },
            Operator::OpLe => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("mov ecx, 0");
                println!("cmp eax, ebx");
                println!("setle cl");
                self.store_var("ecx".to_string(), "cl".to_string(), self.result.clone());
            },
            Operator::OpEqu => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("mov ecx, 0");
                println!("cmp eax, ebx");
                println!("sete cl");
                self.store_var("ecx".to_string(), "cl".to_string(), self.result.clone());
            },
            Operator::OpNe => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("mov ecx, 0");
                println!("cmp eax, ebx");
                println!("setne cl");
                self.store_var("ecx".to_string(), "cl".to_string(), self.result.clone());
            },
            Operator::OpAnd => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                println!("cmp eax, 0");
                println!("setne cl");
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("cmp ebx, 0");
                println!("setne bl");
                println!("add eax, ebx");
                self.store_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpOr => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                println!("cmp eax, 0");
                println!("setne al");
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("cmp ebx, 0");
                println!("setne bl");
                println!("or eax, ebx");
                self.store_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpNot => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                println!("mov ebx, 0");
                println!("cmp eax, 0");
                println!("sete bl");
                self.store_var("ebx".to_string(), "bl".to_string(), self.result.clone());
            },
            Operator::OpLea => {
                self.lea_var("eax".to_string(), self.arg1.clone());
                self.store_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpSet => {
                self.load_var("eax".to_string(), "al".to_string(), self.result.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.result.clone());
                println!("mov [ebx], eax");
            },
            Operator::OpGet => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                println!("mov eax, [eax]");
                self.store_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpJmp => {
                let target = self.target.clone().unwrap();
                let label = target.get_label();
                println!("jmp {}", label);
            },
            Operator::OpJt => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                println!("cmp eax, 0");
                let target = self.target.clone().unwrap();
                let label = target.get_label();
                println!("jne {}", label);
            },
            Operator::OpJf => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                println!("cmp eax, 0");
                let target = self.target.clone().unwrap();
                let label = target.get_label();
                println!("je {}", label);
            },
            Operator::OpJne => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                self.load_var("ebx".to_string(), "bl".to_string(), self.arg2.clone());
                println!("cmp eax, ebx");
                let target = self.target.clone().unwrap();
                let label = target.get_label();
                println!("jne {}", label);
            },
            Operator::OpArg => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                println!("push eax");
            },
            Operator::OpProc => {
                let fun = self.fun.clone().unwrap();
                println!("call {}", fun.get_name());
                println!("add esp, {}", fun.get_para_var().len() * 4);
            },
            Operator::OpCall => {
                let fun = self.fun.clone().unwrap();
                println!("call {}", fun.get_name());
                println!("add esp, {}", fun.get_para_var().len() * 4);
                self.load_var("eax".to_string(), "al".to_string(), self.result.clone());
            },
            Operator::OpRet => {
                let target = self.target.clone().unwrap();
                let label = target.get_label();
                println!("jmp {}", label);
            },
            Operator::OpRetv => {
                self.load_var("eax".to_string(), "al".to_string(), self.arg1.clone());
                let target = self.target.clone().unwrap();
                let label = target.get_label();
                println!("jmp {}", label);
            },
        }

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

