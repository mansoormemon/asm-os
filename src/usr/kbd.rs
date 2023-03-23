// MIT License
//
// Copyright (c) 2023 Mansoor Ahmed Memon.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use core::ops::Deref;
use core::str::FromStr;

use crate::api;
use crate::api::keyboard::Layout;
use crate::println;

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
                        api::keyboard::set_layout(lyt);
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
                println!("{}", api::keyboard::get_layout().as_str());
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
                api::keyboard::reset_layout();
                println!("The keyboard layout has been reset to `{}`", api::keyboard::get_layout().as_str());
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
