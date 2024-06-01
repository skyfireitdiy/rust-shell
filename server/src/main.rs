use std::io::{self, BufRead};

use server::{reg_shell_cmd, Shell};

fn print_hello() {
    println!("Hello, world!");
}

fn add_two(a: i64, b: i64) -> i64 {
    println!("{} + {} = {}", a, b, a + b);
    return a + b;
}

fn print_str(s: &String) {
    println!("{}", s);
}

fn add_seven(a: i64, b: i64, c: i64, d: i64, e: i64, f: i64, g: i64) -> i64 {
    println!(
        "{} + {} + {} + {} + {} + {} + {} = {}",
        a,
        b,
        c,
        d,
        e,
        f,
        g,
        a + b + c + d + e + f + g
    );
    return a + b + c + d + e + f + g;
}

fn main() {
    let mut stdin = io::stdin().lock();
    let mut buf = String::new();

    let mut shell = Shell::new();

    reg_shell_cmd!(shell,
        {"hello", print_hello},
        {"add_two", add_two},
        {"print_str", print_str},
        {"add_seven", add_seven}
    );

    while let Ok(_) = stdin.read_line(&mut buf) {
        match shell.run_command(&buf) {
            Ok(r) => {
                println!("return: {}", r);
            }
            Err(e) => println!("{}", e),
        }
        buf.clear();
    }
}
