use nom::IResult;
use nom::number::complete::{be_u32};
use nom::bytes::streaming::take;

use bytes::Buf;
use rand::Rng;

fn byte_test() {
    let data = vec![b'a', 0, 33, 42, 0];
    let mut p = &data[..];
    // assert_eq!(p.get_u8(), b'a'); // 0バイト目を8bit整数として読み出し
    // assert_eq!(p.get_u16(), 33); // 1〜2バイト目をビッグエンディアン16bit整数として読み出し
    // assert_eq!(p.get_u16_le(), 42); // 3〜4バイト目をリトルエンディアン16bit整数として読み出し

    // println!("{}",p.get_u8());
    // println!("{}",p.get_u16());
    // println!("{}",p.get_u16_le());
    // println!("{:?}",p);

    // 配列の参照はコピーが生成
    // let mut q = p;
    // println!("{}",p.get_u8());
    // println!("{}",p.get_u16());
    // println!("{}",p.get_u16_le());
    // println!("{:?}",p);
    
    // println!("{}",q.get_u8());
    // println!("{:?}",q);



    println!("{:?}",data);
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

fn make_dummy_data(){
    let mut vec:Vec<u8> = Vec::new();

    for _ in 0..5 {
        let mut header:Vec<u8> = make_header();
        let mut data:Vec<u8> = make_data();
        let mut footer:Vec<u8> = make_footer();

        vec.append(&mut header);
        vec.append(&mut data);
        vec.append(&mut footer);
        
    }

    println!("{:?}",vec.len() );
}

// fn make_header() -> Vec<u8> {
fn make_header() -> Vec<u8>{
    // Z相がTRUE
    let d1= 0b00000000_00000000_00000000_10000000u32.to_le_bytes();
    // 300回目のプロファイルデータ
    let d2 = 300u32.to_be_bytes();
    // 350回目のエンコーダカウント
    let l3 = 350i32.to_be_bytes();
    // 計測開始から6800ms
    let d4 = 68000u32.to_be_bytes();
    // データなし
    let d5= 0b00000000_00000000_00000000_00000000u32.to_le_bytes();
    // データなし
    let d6= 0b00000000_00000000_00000000_00000000u32.to_le_bytes();

    let vec:Vec<u8> =[d1,d2,l3,d4,d5,d6].concat();
    // println!("{:?}",d2);
    // println!("{:?}",vec);
    return vec
}
fn make_footer() -> Vec<u8>{
    // データなし
    let d1= 0b00000000_00000000_00000000_00000000u32.to_le_bytes();

    let vec:Vec<u8> =[d1].concat();
    return vec
}

fn make_data()-> Vec<u8>{
    let mut vec:Vec<u8> = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..3200 {
        let i: i32 = rng.gen();
        let i = i.to_be().to_le_bytes();
        vec.push(i[0]);
        vec.push(i[1]);
        vec.push(i[2]);
        vec.push(i[3]);
    }
    println!("{:?}",vec.len() );

    return vec
    
}

fn main(){
    make_dummy_data();
}


// fn u32array_to_u8array(u32:&[u32]) -> &[u8]{


// }   