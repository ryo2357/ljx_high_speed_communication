fn input() -> String{
    let mut buf = String::new(); //A
    std::io::stdin().read_line(&mut buf).unwrap(); //B
    buf.trim().to_string() //C
}

use std::io::Write;
fn _main(){    

    print!("enter some thing: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");
    print!("{}", input);
}

fn wait_until_enter(){
    print!("press enter: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    // print!("{}", input);
}


fn main (){
    println!("aaaa");
    wait_until_enter();
    println!("aaaa");
}