use crate::common::Tag;

union InitValue {
    int_val: i32,
    char_value: char,
}

pub struct Var<'a> {
    // 特殊标记
    literal: bool,           // 是否是常量
    scope_path: Vec<i32>,    // 作用域路径

    // 基本声明形式
    externed: bool,          // 是否extern声明
    var_type: Tag,           // 变量类型
    name: String,            // 变量名称
    is_ptr: bool,            // 是否是指针
    is_array: bool,          // 是否是数组
    array_size: i32,         // 数组长度

    // 初始值部分
    is_left: bool,              // 是否可以作为左值
    init_data: Option<&'a Var<'a>>,     // 缓存初值数据，延迟处置处理
    inited: bool,               // 是否初始化
    init_value: InitValue,       // int, char初始值
    str_val: String,            // 字符串常量初值
    ptr_val: String,            // 字符指针初值
    ptr: Option<&'a Var<'a>>,           // 变量指针类型

    // 附加信息
    size: i32,               // 变量的大小
    offset: i32,             // 变量的栈帧偏移
}

impl Var {
    pub(crate) fn new_pointer(sp: &Vec<i32>, ext: bool, t: Tag, ptr: bool, name: String, init: Box<Var>) -> Self {

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
}

pub struct Fun<'a> {
    externed: bool,                  // 声明或定义
    return_type: Tag,                // 返回类型
    name: String,                    // 函数名称
    para_var: Vec<&'a Var<'a>>,                // 参数列表

    // 临时变量地址分配
    max_depth: i32,                  // 栈的最大深度，初始0,标识函数栈分配的最大空间
    cur_esp:   i32,                  // 当前栈指针位置，初始化为0，即ebp存储点
    relocated: bool,                 // 栈帧重定位标记

    // 作用域管理
    scope_esp: Vec<i32>,             // 作用域栈指针位置
}

impl Fun {

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
