use std::env;

fn main() {
    let mut here = env::current_dir().unwrap();
    here.push("vendor");
    // 親ディレクトリは探しにいけなかった
    println!("cargo:rustc-link-search=native={}",format!("{}", here.display()));
    cc::Build::new().file("c_src/bridge.c").compile("bridge");
}
