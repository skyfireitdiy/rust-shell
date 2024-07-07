use libc::exit;
use shell_server::{reg_shell_cmd, Server, Shell};

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

fn get_self_pid() -> u64 {
    return std::process::id() as u64;
}

fn main() {
    let mut shell = Shell::new();

    reg_shell_cmd!(shell,
        {"hello", print_hello},
        {"add_two", add_two},
        {"print_str", print_str},
        {"add_seven", add_seven},
        {"run_exit", exit}
    );

    let pid = get_self_pid();

    println!("pid: {}", pid);

    match Server::new(
        shell,
        format!("/tmp/rust_shell_cmd_{}", pid),
        format!("/tmp/rust_shell_output_{}", pid),
    )
    .run()
    {
        Ok(_) => (),
        Err(err) => println!("run err: {}", err),
    }
}
