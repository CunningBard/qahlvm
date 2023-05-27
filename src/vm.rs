use std::collections::{HashMap};
use std::fmt::{Debug, Formatter};
use std::iter::IntoIterator;
use std::string::ToString;
use std::io::Write;
use crate::ast::{Eval, Node};


const VARIADIC_ARG_NAME: &str = "varargs";

#[derive(Debug ,Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Float(f32),
    String(String),
    Array(Vec<Value>),
    Object(usize),
}


pub trait Callable: Debug {
    fn call(&self, vm: &mut VirtualMachine, args: Vec<Eval>) -> Option<Value>;
    fn args_len(&self) -> usize;
    fn minimum_args_len(&self) -> usize;
    fn is_variadic(&self) -> bool;
}

#[derive(Debug)]
pub struct DefinedFunction {
    name: String,
    args: Vec<String>,
    body: Vec<Node>,
    has_variadic: bool,
}

impl DefinedFunction {
    pub fn new(name: String, args: Vec<String>, body: Vec<Node>, has_variadic: bool) -> Self {
        Self {
            name,
            args,
            body,
            has_variadic
        }
    }
}

impl Callable for DefinedFunction {
    fn call(&self, vm: &mut VirtualMachine, args: Vec<Eval>) -> Option<Value> {
        println!("Calling function: {}", self.name);
        if vm.local.is_some() {
            vm.locals.push(vm.local.take().unwrap());
        }

        vm.local = Some(HashMap::new());
        for (index, arg_name) in self.args.iter().enumerate() {
            let res = vm.eval(args[index].clone());
            vm.local.as_mut().unwrap().insert(arg_name.to_string(), res);
        }

        if self.has_variadic {
            let mut variadic = vec![];
            for arg in args.into_iter().skip(self.args.len()) {
                let res = vm.eval(arg);
                variadic.push(res);
            }
            vm.local.as_mut().unwrap().insert(VARIADIC_ARG_NAME.to_string(), Value::Array(variadic));
        }


        let mut ret = None;
        for node in self.body.iter() {
            match *node {
                Node::Return(ref value) => {
                    ret = Some(vm.eval(value.clone()));
                    break;
                }
                _ => {
                    vm.single_run(node.clone());
                }
            }
        }

        vm.local = vm.locals.pop();

        ret
    }

    fn args_len(&self) -> usize {
        self.args.len()
    }

    fn minimum_args_len(&self) -> usize {
        if self.has_variadic {
            self.args.len() - 1
        } else {
            self.args.len()
        }
    }

    fn is_variadic(&self) -> bool {
        self.has_variadic
    }
}


#[derive(Clone)]
pub struct BuiltInFunction {
    pub name: String,
    pub args_len: usize,
    pub is_variadic: bool,
    pub func: fn(&mut VirtualMachine, Vec<Eval>) -> Option<Value>,
}

impl BuiltInFunction {
    pub fn new(name: String, args_len: usize, is_variadic: bool, func: fn(&mut VirtualMachine, Vec<Eval>) -> Option<Value>) -> Self {
        Self {
            name,
            args_len,
            is_variadic,
            func
        }
    }
}

impl Callable for BuiltInFunction {
    fn call(&self, vm: &mut VirtualMachine, args: Vec<Eval>) -> Option<Value> {
        (self.func)(vm, args)
    }

    fn args_len(&self) -> usize {
        self.args_len
    }

    fn minimum_args_len(&self) -> usize { self.args_len }

    fn is_variadic(&self) -> bool {
        self.is_variadic
    }
}

impl Debug for BuiltInFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BuiltInFunction {{ name: {}, args_len: {} }}", self.name, self.args_len)
    }
}

pub fn println_array(val: &Vec<Value>){
    print!("[");
    for (i, val) in val.iter().enumerate() {
        if i != 0 {
            print!(", ");
        }
        match val {
            Value::Int(val) => { print!("{}", val) }
            Value::Bool(val) => { print!("{}", val) }
            Value::Float(val) => { print!("{}", val) }
            Value::String(val) => { print!("\"{}\"", val) }
            Value::Object(val) => { print!("Object <{:#08x}>", val) }
            Value::Array(val) => {
                println_array(&val)
            }
        }
    }
    print!("]");
}

pub fn builtin_print(vm: &mut VirtualMachine, args: Vec<Eval>) -> Option<Value> {
    for (index, arg) in args.into_iter().enumerate() {
        if index != 0 {
            print!(" ");
        }

        let arg = vm.eval(arg);
        match arg {
            Value::Int(val) => { print!("{}", val) }
            Value::Bool(val) => { print!("{}", val) }
            Value::Float(val) => { print!("{}", val) }
            Value::String(val) => { print!("{}", val) }
            Value::Object(val) => { print!("Object <{:#08x}>", val) }
            Value::Array(val) => {
                println_array(&val)
            }
        }
    }
    None
}

pub fn builtin_println(vm: &mut VirtualMachine, args: Vec<Eval>) -> Option<Value> {
    builtin_print(vm, args);
    println!();
    None
}

pub fn builtin_input(_: &mut VirtualMachine, args: Vec<Eval>) -> Option<Value> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    Some(Value::String(input[..input.len() - 1].to_string()))
}

pub fn builtin_input_print(vm: &mut VirtualMachine, args: Vec<Eval>) -> Option<Value> {
    builtin_print(vm, args.clone());
    std::io::stdout().flush().unwrap();
    builtin_input(vm, args)
}


pub fn builtin_functions() -> Vec<BuiltInFunction>{
    vec![
        BuiltInFunction::new("print".to_string(), 0, true, builtin_print),
        BuiltInFunction::new("println".to_string(), 0, true, builtin_println),
        BuiltInFunction::new("input".to_string(), 0, false, builtin_input),
        BuiltInFunction::new("input_print".to_string(), 0, true, builtin_input_print),
    ]
}

impl Value {
    pub fn as_eval(&mut self) -> Eval {
        match self {
            Value::Int(val) => { Eval::Int(*val) }
            Value::Bool(val) => { Eval::Bool(*val) }
            Value::Float(val) => { Eval::Float(*val) }
            Value::String(val) => { Eval::String(val.clone()) }
            Value::Object(val) => { Eval::Object(Box::new(Eval::Int(*val as i32))) }
            Value::Array(val) => { Eval::Array(val.iter_mut().map(|x| x.as_eval()).collect()) }
        }
    }

    pub fn as_int(&self) -> i32 {
        match self {
            Value::Int(val) => *val,
            _ => panic!("Expected int")
        }
    }
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(val) => *val,
            _ => panic!("Expected bool")
        }
    }
    pub fn as_float(&self) -> f32 {
        match self {
            Value::Float(val) => *val,
            _ => panic!("Expected float")
        }
    }
    pub fn as_string(&self) -> String {
        match self {
            Value::String(val) => val.clone(),
            _ => panic!("Expected string")
        }
    }
}

#[derive(Debug)]
pub struct Object {
    pub fields: HashMap<String, Value>,
}

impl Object {
    fn new(fields: HashMap<String, Value>) -> Self {
        Object {
            fields
        }
    }
}

pub enum GcApproach {
    None,
    ReferenceCounting,
    Custom { func: fn(&mut VirtualMachine, Vec<String>) }
}

impl Debug for GcApproach {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GcApproach::None => { write!(f, "None") }
            GcApproach::ReferenceCounting => { write!(f, "ReferenceCounting") }
            GcApproach::Custom { .. } => { write!(f, "Custom") }
        }
    }
}


#[derive(Debug)]
pub struct VirtualMachine {
    pub objects: HashMap<usize, Object>,
    pub objects_in_use: Vec<(usize, u32)>,
    pub functions: HashMap<String, Box<dyn Callable>>,
    pub global_variables: HashMap<String, Value>,
    pub locals: Vec<HashMap<String, Value>>,
    pub local: Option<HashMap<String, Value>>,
    pub gc_approach: GcApproach,
}

impl VirtualMachine {
    pub fn new(gc_approach: GcApproach) -> Self {
        let mut functions = HashMap::new();

        for func in builtin_functions() {
            functions.insert(func.name.clone(), Box::new(func) as Box<dyn Callable>);
        }

        VirtualMachine {
            objects: HashMap::new(),
            objects_in_use: vec![],
            functions,
            global_variables: Default::default(),
            locals: vec![],
            local: Default::default(),
            gc_approach,
        }
    }

    pub fn add_defined_functions(&mut self, functions: Vec<DefinedFunction>) {
        for func in functions {
            self.functions.insert(func.name.clone(), Box::new(func) as Box<dyn Callable>);
        }
    }

    pub fn add_rust_functions(&mut self, functions: Vec<BuiltInFunction>) {
        for func in functions {
            self.functions.insert(func.name.clone(), Box::new(func) as Box<dyn Callable>);
        }
    }

    pub fn eval(&mut self, val: Eval) -> Value {
        match val {
            Eval::Int(i) => { Value::Int(i) }
            Eval::Bool(b) => { Value::Bool(b) }
            Eval::Float(f) => { Value::Float(f) }
            Eval::String(s) => { Value::String(s) }
            Eval::Array(arr) => { Value::Array(arr.into_iter().map(|x| self.eval(x)).collect()) }
            Eval::Object(obj) => {
                let obj_id;
                match *obj {
                    Eval::Int(id) => { obj_id = id as usize; }
                    _ => { unimplemented!() }
                }
                Value::Object(obj_id)
            }
            Eval::VarRef(name) => {
                // old
                // self.global_variables.get(&name).unwrap().clone()

                // new
                if self.local.is_some(){
                    return if let Some(val) = self.local.as_ref().unwrap().get(&name) {
                        val.clone()
                    } else {
                        self.global_variables.get(&name).unwrap().clone()
                    }
                } else {
                    self.global_variables.get(&name).unwrap().clone()
                }
            }
            Eval::FnCall(func_name, args) => {
                if !self.functions.contains_key(&*func_name){
                    panic!("Function {} does not exist", func_name);
                }

                let function = self.functions.remove(&*func_name).unwrap();

                if function.args_len() != args.len() && !function.is_variadic(){
                    panic!("Function {} takes {} arguments, {} given", func_name, function.args_len(), args.len());
                }

                let res = match function.call(self, args){
                    None => { panic!("Function {} returned None", func_name) }
                    Some(val) => { val }
                };

                self.functions.insert(func_name, function);
                res
            }
            Eval::Add(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Int(l + r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Float(l + r) }
                    (Eval::String(l), Eval::String(r)) => { Value::String(l + &r) }
                    res => { unimplemented!("{:?}", res) }
                }
            }
            Eval::Sub(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Int(l - r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Float(l - r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Mul(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Int(l * r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Float(l * r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Div(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Int(l / r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Float(l / r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Mod(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Int(l % r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Float(l % r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Pow(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Int(l.pow(r as u32)) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Float(l.powf(r)) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Eq(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Bool(l == r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Bool(l == r) }
                    (Eval::String(l), Eval::String(r)) => { Value::Bool(l == r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Ne(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Bool(l != r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Bool(l != r) }
                    (Eval::String(l), Eval::String(r)) => { Value::Bool(l != r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Gt(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Bool(l > r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Bool(l > r) }
                    (Eval::String(l), Eval::String(r)) => { Value::Bool(l > r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Lt(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Bool(l < r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Bool(l < r) }
                    (Eval::String(l), Eval::String(r)) => { Value::Bool(l < r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Ge(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Bool(l >= r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Bool(l >= r) }
                    (Eval::String(l), Eval::String(r)) => { Value::Bool(l >= r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Le(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Int(l), Eval::Int(r)) => { Value::Bool(l <= r) }
                    (Eval::Float(l), Eval::Float(r)) => { Value::Bool(l <= r) }
                    (Eval::String(l), Eval::String(r)) => { Value::Bool(l <= r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::And(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Bool(l), Eval::Bool(r)) => { Value::Bool(l && r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Or(mut lhs, mut rhs) => {
                lhs.deref_var_ref(&mut self.global_variables);
                rhs.deref_var_ref(&mut self.global_variables);
                lhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                rhs.deref_object_member(&mut self.objects, &mut self.global_variables);
                if lhs.is_an_operator(){ lhs = Box::new(self.eval(*lhs).as_eval()); }
                if rhs.is_an_operator(){ rhs = Box::new(self.eval(*rhs).as_eval()); }

                match (*lhs, *rhs) {
                    (Eval::Bool(l), Eval::Bool(r)) => { Value::Bool(l || r) }
                    _ => { unimplemented!() }
                }
            }
            Eval::Not(mut val) => {
                val.deref_var_ref(&mut self.global_variables);
                val.deref_object_member(&mut self.objects, &mut self.global_variables);
                if val.is_an_operator(){ val = Box::new(self.eval(*val).as_eval()); }

                match *val {
                    Eval::Bool(b) => { Value::Bool(!b) }
                    _ => { unimplemented!() }
                }
            }
            Eval::GetMember(obj_id, member) => {
                let obj_loc = self.eval(*obj_id);
                let obj_id;
                match obj_loc {
                    Value::Int(id) => { obj_id = id as usize; }
                    Value::String(var_name) => {
                        match *self.global_variables.get(&var_name).unwrap() {
                            Value::Object(id) => { obj_id = id as usize; }
                            _ => { unreachable!()}
                        }
                    }
                    _ => { unreachable!() }
                }
                let obj = self.objects.get(&(obj_id as usize)).unwrap();
                return obj.fields.get(&member).unwrap().clone()
            }
        }
    }

    fn reference_count(&mut self, variable_name: String){
        match self.global_variables.get_mut(&variable_name).unwrap(){
            &mut Value::Object(id) => {
                match self.objects_in_use.binary_search_by_key(&id, |&(a, _)| a) {
                    Ok(i) => {
                        let tracker = self.objects_in_use.get_mut(i).unwrap();
                        tracker.1 -= 1;
                        if tracker.1 == 0 {
                            self.objects.remove(&id);
                            self.objects_in_use.remove(i);
                        }

                        self.global_variables.remove(&*variable_name);
                    }
                    _ => { unreachable!() }
                }
            }
            _ => {
                self.global_variables.remove(&*variable_name);
            }
        }
    }

    fn reference_count_vec(&mut self, variable_names: Vec<String>){
        for var_name in variable_names {
            self.reference_count(var_name);
        }

        let to_remove: Vec<usize> = self.objects_in_use.iter().filter(|(_, tracker)| *tracker == 0).map(|(id, _)| *id).collect();
        for id in to_remove {
            self.objects.remove(&id);
        }
    }

    fn run_gc(&mut self, var_names: Vec<String>){
        match self.gc_approach {
            GcApproach::None => {}
            GcApproach::ReferenceCounting => {
                self.reference_count_vec(var_names)
            }
            GcApproach::Custom { func } => {
                func(self, var_names);
            }
        }
    }

    fn dec_use_count(&mut self, val: &Value){
        match val {
            Value::Object(id) => {
                match self.objects_in_use.binary_search_by_key(&id, |(a, _)| a) {
                    Ok(i) => {
                        let tracker = self.objects_in_use.get_mut(i).unwrap();
                        tracker.1 -= 1;
                    }
                    _ => { unreachable!() }
                }
            }
            _ => {}
        }
    }

    fn inc_use_count(&mut self, val: &Value){
        match val {
            Value::Object(id) => {
                match self.objects_in_use.binary_search_by_key(&id, |(a,_)| a) {
                    Ok(i) => {
                        let tracker = self.objects_in_use.get_mut(i).unwrap();
                        tracker.1 += 1;
                    }
                    Err(i) => {
                        self.objects_in_use.insert(i, (*id, 1));
                    }
                }
            }
            _ => {}
        }
    }

    fn loop_run(&mut self, nodes: Vec<Node>){
        let mut assigned: Vec<String> = vec![];
        loop {
            for node in nodes.clone() {
                match node {
                    Node::Break => {
                        self.run_gc(assigned);
                        return;
                    }
                    Node::Continue => { break; }
                    _ => {
                        if let Some(var_name) = self.single_run(node) {
                            assigned.push(var_name);
                        }
                    }
                }
            }
        }
    }

    fn while_loop(&mut self, condition: Eval, body: Vec<Node>){
        let mut assigned: Vec<String> = vec![];
        while self.eval(condition.clone()) == Value::Bool(true) {
            for node in body.clone() {
                match node {
                    Node::Break => {
                        self.run_gc(assigned);
                        return;
                    }
                    Node::Continue => { break; }
                    _ => {
                        if let Some(var_name) = self.single_run(node) {
                            assigned.push(var_name);
                        }
                    }
                }
            }
        }

        self.run_gc(assigned);
    }

    fn single_run(&mut self, node: Node) -> Option<String> {
        // also handle local variables
        match node {
            Node::Assign(var_name, var_val) => {
                if self.local.is_some(){
                    if self.global_variables.contains_key(&*var_name){
                        panic!("Variable {} already exists globally", var_name);
                    }

                    let val = self.eval(var_val);
                    self.local.as_mut().unwrap().insert(var_name, val);

                } else {
                    let val = self.eval(var_val);
                    self.global_variables.insert(var_name.clone(), val);
                }
            }
            Node::Unassign(var_name) => {
                if self.local.is_some(){
                    match self.local.as_mut().unwrap().remove(&*var_name) {
                        Some(val) => {
                            self.dec_use_count(&val);
                        }
                        None => {
                            match self.global_variables.remove(&*var_name) {
                                Some(val) => { self.dec_use_count(&val); }
                                None => { panic!("Variable {} does not exist", var_name); }
                            }
                        }
                    }
                } else {
                    match self.global_variables.remove(&*var_name) {
                        Some(val) => { self.dec_use_count(&val); }
                        None => { panic!("Variable {} does not exist", var_name); }
                    }
                }
            }
            Node::CreateObject(ptr, fields) => {
                let obj_loc = self.eval(ptr);
                let ptr;
                match obj_loc {
                    Value::Int(id) => { ptr = id as usize; }
                    _ => { unreachable!() }
                }

                if self.objects.contains_key(&ptr) {
                    panic!("Object already exists, Deallocate first");
                }

                let mut value = HashMap::new();
                for field in fields {
                    let res = self.eval(field.1);
                    self.inc_use_count(&res);
                    value.insert(field.0, res);
                }
                let object = Object::new(value);
                self.objects.insert(ptr, object);
            }
            Node::DeleteObject(ptr) => {
                let obj_loc = self.eval(ptr);
                let ptr;
                match obj_loc {
                    Value::Int(id) => { ptr = id as usize; }
                    _ => { unreachable!() }
                }

                match self.objects.remove(&ptr){
                    None => {}
                    Some(old) => {
                        for (_, val) in old.fields {
                            self.dec_use_count(&val);
                        }
                    }
                }
            }
            Node::Conditional(conditions, else_block) => {
                let mut ran = false;
                for condition in conditions {
                    if self.eval(condition.0) == Value::Bool(true) {
                        self.multi_run(condition.1);
                        ran = true;
                        break;
                    }
                }

                if !ran && !else_block.is_empty() {
                    self.multi_run(else_block);
                }
            }
            Node::Loop(nodes) => {
                self.loop_run(nodes);
            }
            Node::WhileLoop(condition, body) => {
                self.while_loop(condition, body);
            }
            Node::For(_, _, _) => { unimplemented!() }
            Node::Break => { unreachable!("Break outside of loop") }
            Node::Continue => { unreachable!("Continue outside of loop") }
            Node::FnDef(_, _, _) => { unimplemented!()}
            Node::Return(_) => { unreachable!("Return outside of function") }
            Node::FnCall(name, args) => {
                if !self.functions.contains_key(&*name){
                    panic!("Function {} does not exist", name);
                }

                let function = self.functions.remove(&*name).unwrap();

                if function.args_len() != args.len() && !function.is_variadic() {
                    panic!("Function {} takes {} arguments, {} given", name, function.args_len(), args.len());
                }

                function.call(self, args);

                self.functions.insert(name, function);
            }
            Node::SetMember(obj_id, member, val) => {
                let obj_loc = self.eval(obj_id);
                let obj_id;
                match obj_loc {
                    Value::Int(id) => { obj_id = id as usize; }
                    Value::String(var_name) => {
                        match *self.global_variables.get(&var_name).unwrap() {
                            Value::Object(id) => { obj_id = id as usize; }
                            _ => { unreachable!()}
                        }
                    }
                    _ => { unreachable!() }
                }
                let res = self.eval(val);
                self.inc_use_count(&res);

                let obj = self.objects.get_mut(&(obj_id as usize)).unwrap();
                obj.fields.insert(member, res);
            }
        }
        None
    }

    fn multi_run(&mut self, nodes: Vec<Node>){
        let mut assigned = vec![];
        for node in nodes {
            if let Some(var) = self.single_run(node) {
                assigned.push(var);
            }
        }

        self.run_gc(assigned);
    }

    pub fn run(&mut self, nodes: Vec<Node>) {
        let mut assigned = vec![];
        for node in nodes {
            if let Some(var) = self.single_run(node) {
                assigned.push(var);
            }
        }

        // println!("{:#?}", self);

        self.run_gc(assigned);

        if !self.objects_in_use.is_empty() {
            eprintln!("WARNING UNALLOCATED OBJECTS!")
        }
        for (obj_id , obj) in &self.objects {
            eprintln!("Object {}: {:?}", obj_id, obj);
        }
    }
}