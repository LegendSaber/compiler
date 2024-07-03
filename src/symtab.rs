use std::collections::HashMap;
use crate::common::SemError::{ExternFunDef, FunDecErr, FunReDef, VarReDef, VarUnDec};
use crate::symbol::{Fun, Var, sem_error};

pub struct SymTab {
    // 声明记录顺序
    var_list: Vec<String>,  // 记录变量的添加顺序
    fun_list: Vec<String>,  // 记录函数的添加顺序

    // 内部数据结构
    var_tab: HashMap<String, Vec<Box<Var>>>,
    str_tab: HashMap<String, Box<Var>>,
    fun_tab: HashMap<String, Box<Fun>>,

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
        if self.var_tab.contains_key(&name) {
            // 判断同名变量是否不在一个作用域
            let var_list = self.var_tab.get_mut(&name).unwrap();
            let mut is_exit = false;

            for v in var_list.clone() {
                if var.get_scope_path() == v.get_scope_path() {
                    is_exit = true;

                    break;
                }
            }

            if !is_exit || name.get(..1).unwrap() == "<" {
                var_list.push(var);
            } else {
                sem_error(VarReDef as usize, &name);
            }
        } else {
            self.var_tab.insert(name.clone(), vec![]);
            let v = self.var_tab.get_mut(&name).unwrap();
            v.push(var);
            self.var_list.push(name.clone());
        }
    }

    pub(crate) fn add_str(&mut self, v: Box<Var>) {
        self.str_tab.insert(v.get_name(), v);
    }

    pub(crate) fn get_var(&self, name: String) -> Option<Box<Var>> {
        let mut select: Option<Box<Var>> = None;

        if self.var_tab.contains_key(&name) {
            let var_list = self.var_tab.get(&name).unwrap();
            let path_len = self.scope_path.len();
            let mut max_len = 0;

            for v in var_list.clone() {
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
            sem_error(VarUnDec as usize, &name);    // 变量未声明
        }

        select
    }

    // 声明一个函数
    pub(crate) fn dec_fun(&mut self, mut fun: Box<Fun>) {
        fun.set_extern(true);


        if self.fun_tab.contains_key(&fun.get_name()) {
            // 重复声明的函数，判断是否重复声明
            let last = self.fun_tab.get(&fun.get_name()).unwrap();
            if !last.match_fun(fun.clone()) {
                // 函数声明与定义不匹配
                sem_error(FunDecErr as usize, &fun.get_name());
            }

        } else {
            // 没声明，则添加函数
            self.fun_tab.insert(fun.get_name(), fun.clone());
            self.fun_list.push(fun.get_name());
        }
    }

    // 定义一个函数
    pub(crate) fn def_fun(&mut self, mut fun: Box<Fun>) {
        let mut cur_fun = fun.clone();
        if fun.get_extern() {   // extern不允许出现在定义
            sem_error(ExternFunDef as usize, &fun.get_name());
            fun.set_extern(false);
        }

        // 没有该名字的函数
        if !self.fun_tab.contains_key(&fun.get_name()) {
            // 添加函数
            self.fun_tab.insert(fun.get_name(), fun.clone());
            self.fun_list.push(fun.get_name());
        } else {
            // 已经声明
            let last = self.fun_tab.get_mut(&fun.get_name()).unwrap();
            if last.get_extern() {
                // 之前是声明
                if !last.match_fun(fun.clone()) {   // 匹配的声明
                    sem_error(FunDecErr as usize, &fun.get_name());
                }
                last.define(fun.clone());
            } else {
                // 重定义
                sem_error(FunReDef as usize, &fun.get_name());
            }
            // cur_fun = *last;
        }

        // self.cur_fun = Some(cur_fun);
    }

    // 结束定义一个函数
    pub(crate) fn end_def_fun(&mut self) {
        self.cur_fun = None;
    }
}
