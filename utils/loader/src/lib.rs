#![no_std]
#![no_main]

mod stack;
mod load;


pub use load::{get_num_app, load_app, init_app_cx};

