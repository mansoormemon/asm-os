use crate::api::system;

pub fn main(_args: &[&str]) {
    system::shutdown();
}
