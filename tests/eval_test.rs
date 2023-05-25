use std::collections::HashMap;
use std::iter::zip;
use qahlvm::ast::*;
use qahlvm::vm::*;

mod common;

#[test]
fn int() {
    let val = Eval::Int(1);
    assert_eq!(val.as_int(), 1);
}

#[test]
fn bool() {
    let val = Eval::Bool(true);
    assert_eq!(val.as_bool(), true);
}

#[test]
fn float() {
    let val = Eval::Float(1.0);
    assert_eq!(val.as_float(), 1.0);
}

#[test]
fn string() {
    let val = Eval::String("Hello".to_string());
    assert_eq!(val.as_string(), "Hello");
}

#[test]
fn array() {
    let arr = vec![Eval::Int(1), Eval::Int(2)];
    let val = Eval::Array(arr.clone());
    for (left, right) in zip(val.as_array(), arr) {
        assert_eq!(left.as_int(), right.as_int());
    }
}

#[test]
fn is_an_operator() {
    let val = Eval::Add(Box::new(Eval::Int(1)), Box::new(Eval::Int(2)));
    assert_eq!(val.is_an_operator(), true);
}

#[test]
fn deref_var_ref() {
    let mut map = HashMap::new();
    map.insert("test".to_string(), Value::Int(1));
    let mut val = Eval::VarRef("test".to_string());
    val.deref_var_ref(&mut map);
    assert_eq!(val.as_int(), 1);
}

#[test]
fn deref_object_member() {
    let mut objects = HashMap::new();
    let mut fields = HashMap::new();

    fields.insert("test".to_string(), Value::Int(1));
    objects.insert(1, Object { fields });

    let mut variables = HashMap::new();
    variables.insert("test".to_string(), Value::Object(1));


    let mut val = Eval::GetMember(Box::new(Eval::Int(1)), "test".to_string());
    val.deref_object_member(&mut objects, &mut variables);

    assert_eq!(val.as_int(), 1);
}