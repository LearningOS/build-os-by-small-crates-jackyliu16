#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use output::log::*;

#[no_mangle]
fn main() -> i32 {
    debug!("hello, world");
    println!("Hello, world!");
    0
}
