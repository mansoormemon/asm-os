use alloc::vec::Vec;

use crate::{print, println};
use crate::api::io;
use crate::usr::{chrono, clear, kbd, list, reboot, shutdown, vga};

pub async fn main() {
    loop {
        print!("\x1B[32m@\x1b[0m ");
        let response = io::stdin().read_line();
        let response = response.trim();
        if response == "" { continue; }
        let tokens: Vec<&str> = response.split(' ').collect();
        exec(tokens.as_slice());
    }
}

fn exec(tokens: &[&str]) {
    let commands: [(&str, fn(&[&str])); 7] = [
        ("clear", clear::main),
        ("shutdown", shutdown::main),
        ("reboot", reboot::main),
        ("chrono", chrono::main),
        ("list", list::main),
        ("vga", vga::main),
        ("kbd", kbd::main),
    ];
    let mut cmd_found = false;
    match tokens {
        [exe, args @ ..] => {
            for (cmd, exe_func) in commands {
                if *exe == cmd {
                    exe_func(args);
                    cmd_found = true;
                    break;
                }
            }
        }
        [] => {}
    };
    if !cmd_found {
        println!("\x1B[31mError:\x1B[0m command `{}` not found", tokens[0]);
    }
}
