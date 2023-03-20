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
