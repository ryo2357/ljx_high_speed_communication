use nom::IResult;
use nom::number::complete::{be_u32};
use nom::bytes::streaming::take;

fn main(){
    convert();
}

#[derive(PartialEq, Debug)]
struct MyData {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

fn parse_mydata(input: &[u8]) -> IResult<&[u8], MyData> {
    let (input, a) = be_u32(input)?;
    let (input, b) = be_u32(input)?;
    let (input, c) = be_u32(input)?;
    let (input, d) = be_u32(input)?;

    Ok((input, MyData {
        a, b, c, d
    }))
}


// [Rust の struct, enum, &str, TraitObject などをバイト列として眺めてみる - Qiita](https://qiita.com/taskie/items/46f222017f672ed16346)
// [Rustで再帰的データ構造を定義する - Qiita](https://qiita.com/0yoyoyo/items/96705669f47ff8f9b70d)
use std::mem;

fn get_raw_bytes_with_size<T: ?Sized>(p: &T, size: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(size);
    let view = p as *const _ as *const u8;
    for i in 0..size {
        buf.push(unsafe {*view.offset(i as isize)});
    }
    buf
}


fn split(){
    let mut vec_u8:Vec<u8> = Vec::new();
    for i in 0..101 {
        vec_u8.push(i)
    }

    let array:Vec<&[u8;5]> = Vec::new();

    loop {

    }

}


fn convert(){
    let mut vec_u8:Vec<u8> = Vec::new();


    for i in 0..16 {
        vec_u8.push(i)
    }
    
    println!("vec_u8:{:?}",vec_u8);


    let (return_u8, mydata) = match parse_mydata(&vec_u8){
        Ok(t) => t,
        Err(e) => panic!("{}",e),
    };
    // 要素が足りない⇒Parsing Error: Error { input: [12, 13], code: Eof }
    // 要素が多い⇒余った
    
    println!("vec_u8:{:?}",vec_u8);
    println!("return_u8:{:?}",return_u8);
    println!("mydata:{:?}",mydata);

    // drop(vec_u8);
    // vec_u8のデータが保存されているヒープに対するアドレスを持っている

    println!("vec_u8[4]～[7]:{:x},{:x},{:x},{:x}",vec_u8[4],vec_u8[5],vec_u8[6],vec_u8[7],);
    println!("mydata.b:{:x}",mydata.b);

}