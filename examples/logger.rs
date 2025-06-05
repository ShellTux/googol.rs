use googol::{debugv, errorv, infov};
use log::{debug, error, info};
use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
struct Foo {
    bar: Option<usize>,
}

impl fmt::Display for Foo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.bar {
            Some(bar) => write!(f, "Foo({bar})"),
            None => write!(f, "Foo"),
        }
    }
}

fn main() {
    pretty_env_logger::init();

    for foo in &[Foo { bar: Some(5) }, Foo { bar: None }] {
        dbg!(foo);
        debugv!(foo);
        debugv!(foo, debug); // Using Debug explicitly
        debugv!(foo, display); // Using Display explicitly
        infov!(foo);
        infov!(foo, debug); // Using Debug explicitly
        infov!(foo, display); // Using Display explicitly
        errorv!(foo);
        errorv!(foo, debug); // Using Debug explicitly
        errorv!(foo, display); // Using Display explicitly
        println!("\n-------------------------------------------\n");
    }
}
