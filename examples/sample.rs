use std::env;

use rand::Rng;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: sample <points_num> [upper_bound=100]");
        return;
    }
    let points_num = args[1].parse::<usize>().expect("First argument should be integer");
    let upper_bound = if args.len() >= 3 {
        args[2].parse::<usize>().expect("Second argument should be integer")
    } else { 100 };
    if points_num < 2 { panic!("Wrong points_num argument") }
    if upper_bound < 4 || upper_bound > 1000 { panic!("Wrong upper_bound argument") }

    let mut rng = rand::thread_rng();
    for _ in 0..points_num {
        println!("{}", rng.gen_range(0, upper_bound + 1));
    }
}
