use std::collections::HashMap;
use crate::vm::{Object, Value};

#[derive(Debug, Clone)]
pub enum Eval {
    Int(i32),
    Bool(bool),
    Float(f32),
    String(String),
    Array(Vec<Eval>),
    Object(Box<Eval>),
    GetMember(Box<Eval>, String),
    VarRef(String),
    FnCall(String, Vec<Eval>),

    Add(Box<Eval>, Box<Eval>),
    Sub(Box<Eval>, Box<Eval>),
    Mul(Box<Eval>, Box<Eval>),
    Div(Box<Eval>, Box<Eval>),
    Mod(Box<Eval>, Box<Eval>),
    Pow(Box<Eval>, Box<Eval>),
    Eq(Box<Eval>, Box<Eval>),
    Ne(Box<Eval>, Box<Eval>),
    Gt(Box<Eval>, Box<Eval>),
    Ge(Box<Eval>, Box<Eval>),
    Lt(Box<Eval>, Box<Eval>),
    Le(Box<Eval>, Box<Eval>),
    And(Box<Eval>, Box<Eval>),
    Or(Box<Eval>, Box<Eval>),
    Not(Box<Eval>),
}

impl Eval {
    pub fn as_int(&self) -> i32 {
        match self {
            Eval::Int(val) => *val,
            _ => panic!("Expected int")
        }
    }
    pub fn as_bool(&self) -> bool {
        match self {
            Eval::Bool(val) => *val,
            _ => panic!("Expected bool")
        }
    }
    pub fn as_float(&self) -> f32 {
        match self {
            Eval::Float(val) => *val,
            _ => panic!("Expected float")
        }
    }
    pub fn as_string(&self) -> String {
        match self {
            Eval::String(val) => val.clone(),
            _ => panic!("Expected string")
        }
    }
    pub fn as_array(&self) -> Vec<Eval> {
        match self {
            Eval::Array(val) => val.clone(),
            _ => panic!("Expected array")
        }
    }
    pub fn deref_var_ref(&mut self, map: &mut HashMap<String, Value>) {
        let mut new_val = None;
        match self {
            Eval::VarRef(name) => {
                new_val = Some(map.get(&*name).unwrap().clone().as_eval());
            },
            _ => {}
        }

        if new_val.is_some(){
            *self = new_val.unwrap();
        }
    }
    pub fn deref_object_member(&mut self, objects: &mut HashMap<usize, Object>, variables: &mut HashMap<String, Value>) {
        match self {
            Eval::GetMember(id_loc, name) => {
                let id = match &**id_loc {
                    Eval::Int(id) => *id as usize,
                    Eval::String(var_name) => {
                        match variables.get_mut(&var_name.to_string()).unwrap() {
                            Value::Object(id) => *id as usize,
                            val => panic!("Expected Object for object id: {:?}", val)
                        }
                    }
                    _ => panic!("Expected int for object id")
                };

                let obj = objects.get_mut(&id).unwrap();
                *self = obj.fields.get_mut(name).unwrap().as_eval();
            }
            _ => {}
        }
    }
    pub fn is_an_operator(&self) -> bool {
        match self {
            Eval::Add(_, _) => true,
            Eval::Sub(_, _) => true,
            Eval::Mul(_, _) => true,
            Eval::Div(_, _) => true,
            Eval::Mod(_, _) => true,
            Eval::Pow(_, _) => true,
            Eval::Eq(_, _) => true,
            Eval::Ne(_, _) => true,
            Eval::Gt(_, _) => true,
            Eval::Ge(_, _) => true,
            Eval::Lt(_, _) => true,
            Eval::Le(_, _) => true,
            Eval::And(_, _) => true,
            Eval::Or(_, _) => true,
            Eval::Not(_) => true,
            _ => false
        }
    }
}


#[derive(Debug, Clone)]
pub enum Node {
    Assign(String, Eval),
    Unassign(String),
    SetMember(Eval, String, Eval),
    CreateObject(Eval, Vec<(String, Eval)>),
    DeleteObject(Eval),
    Conditional(Vec<(Eval, Vec<Node>)>, Vec<Node>),

    Loop(Vec<Node>),
    WhileLoop(Eval, Vec<Node>),
    For(String, Eval, Vec<Node>),
    Break,
    Continue,

    FnDef(String, Vec<String>, Vec<Node>),
    Return(Eval),

    FnCall(String, Vec<Eval>),
}