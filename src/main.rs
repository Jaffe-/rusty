mod server;
use std::env;
use std::thread;

fn main() {
    println!("Hello world");
    if env::args().count() <= 2 {
        println!("usage...");
        return;
    }
    let args: Vec<String> = env::args().collect();
    server::run_server(&args[1], &args[2]);
}
