use rand::seq::IteratorRandom;

fn main() {
    let str = include_str!("wisesayings.txt");
    let lines = str.lines();

    let line = lines
        .choose(&mut rand::thread_rng())
        .expect("File had no lines");
    
    println!("{}", line);
}
