use rand::Rng;

pub fn make_dummy(num_profile:usize,num_data:usize) -> Vec<u8>{
    let mut vec:Vec<u8> = Vec::new();

    for _ in 0..num_profile {
        let mut header:Vec<u8> = make_header();
        let mut data:Vec<u8> = make_data(num_data);
        let mut footer:Vec<u8> = make_footer();

        vec.append(&mut header);
        vec.append(&mut data);
        vec.append(&mut footer);
    }

    return vec
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

fn make_data(num_data:usize)-> Vec<u8>{
    let mut vec:Vec<u8> = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..num_data {
        let i: i32 = rng.gen();
        let i = i.to_be().to_le_bytes();
        vec.push(i[0]);
        vec.push(i[1]);
        vec.push(i[2]);
        vec.push(i[3]);
    }

    return vec
    
}

