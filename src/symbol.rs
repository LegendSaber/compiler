use crate::common::Tag;
use std::vec;

enum InitData {
    IntVal(i32),
    CharVal(char),
}
pub struct Var {
    literal: bool,           // 是否是常量
    scope_path: Vec<i32>,    // 作用域路径
    externed: bool,          // 是否extern声明
    var_type: Tag,           // 变量类型
    name: String,            // 变量名称
    is_ptr: bool,            // 是否是指针
    is_array: bool,          // 是否是数组
    array_size: i32,         // 数组长度
    is_left: bool,           // 是否是左值
    init_data: Box<Var>,     // 初值数据
    inited: bool,            // 是否初始化
    init_val: InitData,      // int, char初始值
    str_val: String,         // 字符串常量初值
    ptr_val: String,         // 字符指针初值
    ptr: Box<Var>,           // 变量指针类型
    size: i32,               // 变量的大小
    offset: i32,             // 变量的栈帧偏移
}

impl Var {
    pub(crate) fn new_pointer(sp: &Vec<i32>, ext: bool, t: Tag, ptr: bool, name: String, init: Box<Var>) -> Self {

    }

    fn clear() -> Var {
        Self {
            literal: false,
            scope_path: vec![-1],       // 默认全局作用域
            externed: false,
            var_type: Tag::ERR,
            name: "".to_string(),
            is_ptr: false,
            is_array: false,
            array_size: 0,
            is_left: true,              // 默认变量可以作为左值
            init_data: Box::new(Var {}),
            inited: false,
            init_val: (),
            str_val: "".to_string(),
            ptr_val: "".to_string(),
            ptr: Box::new(Var {}),
            size: 0,
            offset: 0,
        }
    }
}

pub struct Fun {
    externed: bool,                  // 声明或定义
    return_type: Tag,                // 变量类型
    name: String,                    // 变量名称

    // 临时变量地址分配
    max_depth: i32,                  // 栈的最大深度，初始0,标识函数栈分配的最大空间
    cur_esp:   i32,                  // 当前栈指针位置，初始化为0，即ebp存储点
    relocated: bool,                 // 栈帧重定位标记

    // 作用域管理
    scope_esp: Vec<i32>,             // 作用域栈指针位置
}

impl Fun {
    pub(crate) fn enter_scope(&mut self) {
        self.scope_esp.push(0);
    }

    pub(crate) fn leave_scope(&mut self) {
        self.max_depth = if self.cur_esp > self.max_depth { self.cur_esp } else { self.max_depth };
        if let Some(esp) = self.scope_esp.pop() {
            self.cur_esp -= esp;
        }
    }
}
