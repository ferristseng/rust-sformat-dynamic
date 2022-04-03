use sformat_dynamic::{CompiledFormat, DynPointer, TypedValue};
use std::{collections::HashMap, io};

#[allow(dead_code)]
#[derive(Debug)]
struct DebugStruct {
    stuff: &'static str,
}

fn main() {
    let debug = DebugStruct {
        stuff: "hello world",
    };
    let context = HashMap::from([
        ("unsigned_int", TypedValue::Uint(128)),
        ("signed_int", TypedValue::Int(-128)),
        ("boolean", TypedValue::Bool(false)),
        ("string", TypedValue::Str("testing")),
        ("float", TypedValue::Float64(-391.3198491)),
        ("debug", TypedValue::Dyn(DynPointer::Debug(&debug))),
    ]);
    let format = "uint = {unsigned_int}, int = {signed_int}, bool = {boolean}, string = {string}, float = {float}, debug = {debug}\n";
    let format: CompiledFormat<'_> = format.try_into().unwrap();
    let mut output = io::stdout();

    format
        .format(&mut output, &context)
        .expect("expected to write formatted string");
}
