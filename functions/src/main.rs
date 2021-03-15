fn five_times(x: i32) -> i32 {
    x * 5
}

fn main() {
    let num = 1;
    let x = five_times(num);
    println!("The value of {} * 5 is: {}", num, x);
}

#[test]
fn test_add() {
    assert_eq!(five_times(1), 5);
}
