use core::ops::Deref;
use core::str::FromStr;

use crate::println;
use crate::api;
use crate::api::kbd::Layout;

const OPS: [(&str, fn(&[&str])); 3] = [
    ("set", set),
    ("get", get),
    ("reset", reset)
];

fn set(args: &[&str]) {
    let mut iter = args.iter();
    if let Some(property) = iter.next() {
        match property.deref() {
            "layout" => {
                if let Some(value) = iter.next() {
                    if let Ok(lyt) = Layout::from_str(value) {
                        api::kbd::set_layout(lyt);
                        println!("The keyboard layout has been set to `{}`.", lyt.as_str());
                    } else {
                        println!("\x1B[31mError:\x1B[0m keyboard layout `{}` is not supported.", value);
                    }
                } else {
                    println!("\x1B[31mError:\x1B[0m value for property is missing.");
                }
            }
            _ => {
                println!("\x1B[31mError:\x1B[0m property `{}` is not recognized.", property);
            }
        }
    } else {
        println!("\x1B[31mError:\x1B[0m please specify a property.");
    }
}

fn get(args: &[&str]) {
    let mut iter = args.iter();
    if let Some(property) = iter.next() {
        match property.deref() {
            "layout" => {
                println!("{}", api::kbd::get_layout().as_str());
            }
            _ => {
                println!("\x1B[31mError:\x1B[0m property `{}` is not recognized.", property);
            }
        }
    } else {
        println!("\x1B[31mError:\x1B[0m please specify a property.");
    }
}

fn reset(args: &[&str]) {
    let mut iter = args.iter();
    if let Some(property) = iter.next() {
        match property.deref() {
            "layout" => {
                api::kbd::reset_layout();
                println!("The keyboard layout has been reset to `{}`", api::kbd::get_layout().as_str());
            }
            _ => {
                println!("\x1B[31mError:\x1B[0m property `{}` is not recognized.", property);
            }
        }
    } else {
        println!("\x1B[31mError:\x1B[0m please specify a property.");
    }
}

pub fn main(args: &[&str]) {
    match args {
        [operation, sub_args @ ..] => {
            for (op, func) in OPS {
                if operation.deref() == op {
                    func(sub_args);
                    break;
                }
            }
        }
        [] => {}
    }
}
