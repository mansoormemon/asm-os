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

use crate::api::vga;
use crate::println;

const OPS: [(&str, fn(&[&str])); 1] = [
    ("set", set),
];

fn set(args: &[&str]) {
    let mut iter = args.iter();
    if let Some(property) = iter.next() {
        match property.deref() {
            "palette" => {
                if let Some(value) = iter.next() {
                    match value.deref() {
                        "material" => {
                            vga::set_palette(vga::palette::MATERIAL);
                            println!("The VGA color palette has been set to `{}`.", value);
                        }
                        "material-darker" => {
                            vga::set_palette(vga::palette::MATERIAL_DARKER);
                            println!("The VGA color palette has been set to `{}`.", value);
                        }
                        "material-ligher" => {
                            vga::set_palette(vga::palette::MATERIAL_LIGHTER);
                            println!("The VGA color palette has been set to `{}`.", value);
                        }
                        "gruvbox" => {
                            vga::set_palette(vga::palette::GRUVBOX);
                            println!("The VGA color palette has been set to `{}`.", value);
                        }
                        _ => {
                            println!("\x1B[31mError:\x1B[0m VGA color palette `{}` does not exist.", value);
                        }
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
