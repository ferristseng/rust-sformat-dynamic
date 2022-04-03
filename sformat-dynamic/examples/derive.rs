use sformat_dynamic::{derive::Context, CompiledFormat};
use std::io;

#[derive(Context)]
struct TestContext {
    unsigned_int: usize,
    signed_int: isize,
    float: f64,
    boolean: bool,
}

fn main() {
    let context = TestContext {
        unsigned_int: 128,
        signed_int: -128,
        float: -1.3918371,
        boolean: false,
    };
    let format = "uint = {unsigned_int}, int = {signed_int}, float = {float}, bool = {boolean}\n";
    let format: CompiledFormat<'_> = format.try_into().unwrap();
    let mut output = io::stdout();

    format
        .format(&mut output, &context)
        .expect("expected to write formatted string");
}
