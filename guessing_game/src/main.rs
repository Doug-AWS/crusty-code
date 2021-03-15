use rand::Rng;
use std::cmp::Ordering;
use std::io;

fn main() {
    let start: u32 = 1;
    let end: u32 = 100;

    let secret_number = rand::thread_rng().gen_range(start, end);

    println!("Guess a number between {} and {}", start, end + 1);

    // println!("The secret number is: {}", secret_number);
    let mut num_tries: u32 = 0;

    loop {
        num_tries = num_tries + 1;

        println!("Enter your guess.");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("{} is not a number", guess.trim());
                continue;
            }
        };

        println!("You guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                println!("That took you {} tries", num_tries);
                break;
            }
        }
    }
}
