use serde::Serialize;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Debug)]
pub struct LjxDataConverterConfig {
    // .envから取得しないデータはOptionで
    pub ljx_data_path: Option<String>,
    pub output_dir: Option<String>,
    pub output_name: String,
    pub convert_quantity: usize,
}
fn main() -> anyhow::Result<()> {
    let test_struct = LjxDataConverterConfig {
        ljx_data_path: Some(String::from("Option String")),
        output_dir: None,
        output_name: String::from("String"),
        convert_quantity: 523,
    };
    println!("{:?}", test_struct);
    let toml_struct = toml::to_string(&test_struct).unwrap();
    println!("{:?}", toml_struct);

    let mut file = File::create("./test.txt")?;
    // tomlをSerializeした場合、最後に改行が入るっぽい
    write!(file, "{}", toml_struct)?;

    writeln!(file, "have_brightness = ")?;
    writeln!(file)?;
    writeln!(file, "have_brightness = ")?;
    Ok(())
}
