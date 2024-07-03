use crate::plat::STACK_BASE;
use crate::common::{SemError, Tag};
use crate::common::SemError::VoidVar;
use crate::common::Tag::{KwChar, KwInt, KwVoid};
use crate::token::TokenType;

pub(crate)  fn sem_error(code: usize, name: &str) {
    //语义错误信息串
    const SEM_ERROR_TABLE: [&str; 21] = ["变量重定义",										//附加名称信息
                                         "函数重定义",
                                         "变量未声明",
                                         "函数未声明",
                                         "函数声明与定义不匹配",
                                         "函数行参实参不匹配",
                                         "变量声明时不允许初始化",
                                         "函数定义不能声明extern",
                                         "数组长度应该是正整数",
                                         "变量初始化类型错误",
                                         "全局变量初始化值不是常量",
                                         "变量不能声明为void类型",					//没有名称信息
                                         "无效的左值表达式",
                                         "赋值表达式类型不兼容",
                                         "表达式运算对象不能是基本类型",
                                         "表达式运算对象不是基本类型",
                                         "数组索引运算类型错误",
                                         "void的函数返回值不能参与表达式运算",
                                         "break语句不能出现在循环或switch语句之外",
                                         "continue不能出现在循环之外",
                                         "return语句和函数返回值类型不匹配"];
    println!("语义错误: {} {}.", name, SEM_ERROR_TABLE[code]);
}

#[derive(Clone)]
pub struct Var {
    // 特殊标记
    literal: bool,           // 是否是常量
    scope_path: Vec<i32>,    // 作用域路径

    // 基本声明形式
    externed: bool,          // 是否extern声明
    var_type: Tag,           // 变量类型
    name: String,            // 变量名称
    is_ptr: bool,            // 是否是指针
    is_array: bool,          // 是否是数组
    array_size: isize,         // 数组长度

    // 初始值部分
    is_left: bool,              // 是否可以作为左值
    init_data: Option<Box<Var>>,     // 缓存初值数据，延迟处置处理
    inited: bool,               // 是否初始化
    int_val: isize,
    char_value: char,
    str_val: String,            // 字符串常量初值
    ptr_val: String,            // 字符指针初值
    ptr: Option<Box<Var>>,           // 变量指针类型

    // 附加信息
    size: isize,               // 变量的大小
    offset: isize,             // 变量的栈帧偏移
}

impl Var {
    fn new() -> Self {
        Var {
            literal: false,
            scope_path: vec![],
            externed: false,
            var_type: Tag::ERR,
            name: "".to_string(),
            is_ptr: false,
            is_array: false,
            array_size: 0,
            is_left: false,
            init_data: None,
            inited: false,
            int_val: 0,
            char_value: '0',
            str_val: "".to_string(),
            ptr_val: "".to_string(),
            ptr: None,
            size: 0,
            offset: 0,
        }
    }

    // 关键信息初始化
    fn clear(&mut self) {
        self.scope_path.push(-1);   // 默认全局作用域
        self.externed = false;
        self.is_ptr = false;
        self.is_array = false;
        self.is_left = true;
        self.inited = false;
        self.literal = false;
        self.size = 0;
        self.offset = 0;
        self.ptr = None;
        self.init_data = None;
    }

    // 变量，指针
    pub(crate) fn new_pointer(sp: Vec<i32>, ext: bool, t: Tag, ptr: bool, name: String, init: Option<Box<Var>>) -> Self {
        let mut var = Var::new();

        var.clear();
        var.set_scope_path(sp);
        var.set_extern(ext);
        var.set_type(t);
        var.set_ptr(ptr);
        var.set_name(name);
        var.set_init_data(init);

        var
    }

    // 变量，数组
    pub(crate) fn new_array(sp: Vec<i32>, ext: bool, t: Tag, name: String, len: isize) -> Self {
        let mut var = Var::new();

        var.clear();
        var.set_scope_path(sp);
        var.set_extern(ext);
        var.set_type(t);
        var.set_name(name);
        var.set_array(len);

        var
    }


	// 常量,不涉及作用域的变化，字符串存储在字符串表，其他常量作为初始值(使用完删除)
    pub(crate) fn new_const(lt: &TokenType) -> Self {
        let mut var = Var::new();

        var.clear();
        var.set_literal(true);
        var.set_left(false);

        match lt {
            TokenType::Str(str) => {
                var.set_type(KwChar);
                // var.set_name(GenIR::genLb());
                var.set_str_val(str.get_str());
                var.set_array(var.str_val.len() as isize);
            },
            TokenType::Num(num) => {
                var.set_type(KwInt);
                var.set_name("<int>".to_string());
                var.set_int_val(num.get_val());
            },
            TokenType::Char(c) => {
                var.set_type(KwChar);
                var.set_name("<char>".to_string());
                var.set_int_val(0);
                var.set_char_value(c.get_ch());
            },
            _ => {

            }
        }

        var
    }

    // void变量
    pub(crate) fn new_void() -> Self {
        let mut var = Var::new();

        var.clear();
        var.set_name("<void>".to_string());
        var.set_left(false);
        var.set_int_val(0);
        var.set_literal(false);
        var.set_type(KwVoid);
        var.set_ptr(true);

        var
    }

    // 整形变量
    pub(crate) fn new_int(val: isize) -> Self {
        let mut var = Var::new();

        var.clear();
        var.set_name("<int>".to_string());
        var.set_left(false);
        var.set_literal(true);
        var.set_type(KwInt);
        var.set_int_val(val);

        var
    }
}

impl Var {

    pub(crate) fn set_char_value(&mut self, char_value: char) {
        self.char_value = char_value;
    }

    pub(crate) fn set_int_val(&mut self, int_val: isize) {
        self.int_val = int_val;
    }

    pub(crate) fn set_str_val(&mut self, str_val: String) {
        self.str_val = str_val;
    }

    pub(crate) fn set_literal(&mut self, literal: bool) {
        self.literal = literal;
    }

    pub(crate) fn get_literal(&self) -> bool {
        self.literal
    }

    pub(crate) fn set_left(&mut self, left: bool) {
        self.is_left = left;
    }

    pub(crate) fn get_left(&self) -> bool {
        self.is_left
    }

    pub(crate) fn set_scope_path(&mut self, scope_path: Vec<i32>) {
        self.scope_path = scope_path;
    }

    pub(crate) fn get_scope_path(&self) -> Vec<i32> {
        self.scope_path.clone()
    }

    pub(crate) fn set_init_data(&mut self, init_data: Option<Box<Var>>) {
        self.init_data = init_data
    }

    pub(crate) fn get_init_data(&self) -> Option<Box<Var>>{
        self.init_data.clone()
    }

    pub(crate) fn set_extern(&mut self, ext: bool) {
        self.externed = ext;
        self.size = 0;
    }

    pub(crate) fn get_extern(&self) -> bool {
        self.externed
    }

    pub(crate) fn set_type(&mut self, t: Tag) {
        self.var_type = t;
        if self.var_type == KwVoid {
            sem_error(VoidVar as usize, "");     // 不允许void变量
            self.var_type = KwInt;    // 默认为int
        }

        if !self.externed && self.var_type == KwInt {
            self.size = 4;
        } else if !self.externed && self.var_type == KwChar {
            self.size = 1;
        }
    }

    pub(crate) fn get_type(&self) -> Tag {
        self.var_type
    }

    pub(crate) fn set_ptr(&mut self, ptr: bool) {
        if ptr {
            self.is_ptr = true;
            if !self.externed {
                self.size = 4;
            }
        }
    }

    pub(crate) fn get_ptr(&self) -> bool {
        self.is_ptr
    }

    pub(crate) fn set_name(&mut self, n: String) {
        self.name = n;
    }

    pub(crate) fn get_name(&self) -> String {
        self.name.clone()
    }

    pub(crate) fn set_array(&mut self, len: isize) {
        if len <= 0 {
            sem_error(SemError::ArrayLenInvalid as usize, &self.name);
        } else {
            self.is_array = true;
            self.is_left = false;   // 数组不能作为左值
            self.array_size = len;
            if !self.externed {
                self.size *= len;
            }
        }
    }

    pub(crate) fn set_offset(&mut self, off: isize) {
        self.offset = off;
    }

    pub(crate) fn get_offset(&self) -> isize {
        self.offset
    }
}

#[derive(Clone)]
pub struct Fun {
    externed: bool,                  // 声明或定义
    return_type: Tag,                // 返回类型
    name: String,                    // 函数名称
    para_var: Vec<Box<Var>>,                // 参数列表

    // 临时变量地址分配
    max_depth: i32,                  // 栈的最大深度，初始0,标识函数栈分配的最大空间
    cur_esp:   i32,                  // 当前栈指针位置，初始化为0，即ebp存储点
    relocated: bool,                 // 栈帧重定位标记

    // 作用域管理
    scope_esp: Vec<i32>,             // 作用域栈指针位置
}

impl Fun {
    // 声明定义匹配
    pub(crate) fn match_fun(&self, f: Box<Fun>) -> bool {
        if self.name != f.get_name() {
            return false;
        }

        if self.para_var.len() != f.para_var.len() {
            return false;
        }

        for i in 0..self.para_var.len() {
            if self.para_var[i].get_type() != f.para_var[i].get_type() {
                return false;
            }
        }

        if self.return_type != f.return_type {
            return false;
        }

        true
    }

    // 行参实惨匹配
    pub(crate) fn match_args(&self, args: Vec<Box<Var>>) -> bool {
        if self.para_var.len() != args.len() {
            return false;
        }

        let len = self.para_var.len();
        for i in 0..len {
            if self.para_var[i].get_type() != args[i].get_type() {
                return false;
            }
        }
        true
    }

    pub(crate) fn define(&mut self, def: Box<Fun>) {
        self.externed = false;
        self.para_var = def.para_var;
    }

    // 进入一个新的作用域
    pub(crate) fn enter_scope(&mut self) {
        self.scope_esp.push(0);
    }

    // 离开当前作用域
    pub(crate) fn leave_scope(&mut self) {
        self.max_depth = if self.cur_esp > self.max_depth { self.cur_esp } else { self.max_depth }; // 计算最大深度
        if let Some(esp) = self.scope_esp.pop() {
            self.cur_esp -= esp;
        }
    }
}

impl Fun {
    pub(crate) fn new(ext: bool, t: Tag, n: String, para_list: Vec<Box<Var>>) -> Self {
        let mut para_list = para_list;
        let mut arg_off = 4;
        for i in 0..para_list.len() {
            para_list[i].set_offset(arg_off);
            arg_off += 4;
        }

        Fun {
            externed: ext,
            return_type: t,
            name: n,
            para_var: para_list,
            max_depth: STACK_BASE,
            cur_esp: STACK_BASE,
            relocated: false,
            scope_esp: vec![0],
        }
    }

    pub(crate) fn get_name(&self) -> String {
        self.name.clone()
    }

    pub(crate) fn get_return_type(&self) -> Tag {
        self.return_type
    }

    pub(crate) fn set_extern(&mut self, ext: bool) {
        self.externed = ext;
    }

    pub(crate) fn get_extern(&self) -> bool {
        self.externed
    }
}
