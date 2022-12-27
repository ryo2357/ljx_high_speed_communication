
mod lib;
use lib::dummy::make_dummy;

fn main(){
    let num_profile = 5;
    let num_data = 5;
    let mut vec= make_dummy(num_profile,num_data);

    
    // println!("{:?}",vec.len() );
    // println!("{:?}",vec);


    for profile in vec.chunks(4*(num_data+7)) {
        println!("{:?}",profile.len() );
        println!("{:?}",profile);
        parse_write_profile(profile,num_data);
    }

    // 所有権確認用
    println!("{:?}",vec.len() );
}

fn parse_write_profile(profile:&[u8],num_data:usize){
    let header = &profile[0..16];
    
    let data = &profile[24..24+4*num_data];
    
    println!("{:?}",header );
    println!("{:?}",data );

}


