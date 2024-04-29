use crate::symbol::Fun;

pub struct SymTab<'a> {
    // 声明记录顺序
    var_list: Vec<String>,  // 记录变量的添加顺序
    fun_list: Vec<String>,  // 记录函数的添加顺序

    // 内部数据结构
    // var_tab: HashMap<String, Vec<>>,

    // 辅助分析数据记录
    cur_fun: Option<&'a Fun<'a>>,       // 当前分析的函数
    scope_id: i32,                      // 作用域的唯一编号
    scope_path: Vec<i32>,               // 动态记录作用域的路径，全局为0,0 1 2-第一个函数的第一个局部块
}

impl SymTab {

    // 初始化符号表
    pub(crate) fn new(&mut self) {
        self.scope_id = 0;
        self.cur_fun = None;
        self.scope_path.push(0);
    }

    pub(crate) fn enter(&mut self) {
        self.scope_id += 1;
        self.scope_path.push(self.scope_id);
        if let Some(fun) = self.cur_fun.as_mut() {
            fun.enter_scope()
        }
    }

    pub(crate) fn leave(&mut self) {
        self.scope_path.pop();
        if let Some(fun) = self.cur_fun.as_mut() {
            fun.leave_scope()
        }
    }
}
