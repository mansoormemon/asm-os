use crate::api::chrono;
use crate::println;

pub fn main(_args: &[&str]) {
    println!("{}", chrono::Clock::now());
}
