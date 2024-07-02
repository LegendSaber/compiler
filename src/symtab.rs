use std::collections::HashMap;
use crate::common::SemError::{VAR_RE_DEF, VAR_UN_DEC};
use crate::symbol::{Fun, Var, sem_error};

pub struct SymTab {
    // 声明记录顺序
    var_list: Vec<String>,  // 记录变量的添加顺序
    fun_list: Vec<String>,  // 记录函数的添加顺序

    // 内部数据结构
    var_tab: HashMap<String, Vec<Box<Var>>>,
    str_tab: HashMap<String, Box<Var>>,

    // 辅助分析数据记录
    cur_fun: Option<Box<Fun>>,          // 当前分析的函数
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

    pub(crate) fn get_scope_path(&self) -> Vec<i32>{
        self.scope_path.clone()
    }

    // 添加一个变量到符号表
    pub(crate) fn add_var(&mut self, var: Box<Var>) {
        let name = var.get_name();
        if self.var_tab.contains_key(name.borrow()) {
            // 判断同名变量是否不在一个作用域
            let var_list = self.var_tab.get_mut(name.borrow()).unwrap();
            let mut is_exit = false;

            for v in var_list {
                if var.get_scope_path() == v.get_scope_path() {
                    is_exit = true;

                    break;
                }
            }

            if !is_exit || name[0] == '<' {
                var_list.push(var);
            } else {
                sem_error(VAR_RE_DEF as usize, name.borrow());
            }
        } else {
            self.var_tab.insert(name.clone(), vec![]);
            self.var_tab[name.clone()].push(var);
            self.var_list.push(name.clone());
        }
    }

    pub(crate) fn add_str(&mut self, v: Box<Var>) {
        self.str_tab.insert(v.get_name(), v);
    }

    pub(crate) fn get_var(&self, name: String) -> Option<Box<Var>> {
        let mut select: Option<Box<Var>> = None;

        if self.var_tab.contains_key(name.borrow()) {
            let var_list = self.var_tab.get(name.borrow()).unwrap();
            let path_len = self.scope_path.len();
            let mut max_len = 0;

            for &v in var_list {
                let len = v.get_scope_path().len();
                if len <= path_len && v.get_scope_path()[len - 1] == self.scope_path[len - 1] {
                    if len > max_len {
                        max_len = len;
                        select = Some(v.clone());
                    }
                }
            }
        }

        if let None = select {
            sem_error(VAR_UN_DEC as usize, name.borrow());    // 变量未声明
        }

        select
    }
}
