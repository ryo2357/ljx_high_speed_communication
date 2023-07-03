use log::info;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

// TODO:命名を変更したので波及範囲を調査
#[derive(Deserialize, Debug)]
pub struct LjxDataConverterConfig {
    // .envから取得しないデータはOptionで
    pub ljx_data_path: Option<String>,
    pub output_dir: String,
    pub output_name: String,

    // ljx_data_to_pcd
    pub convert_quantity: usize,

    pub y_start_num: usize,
    pub y_pitch: f64,
    pub y_take_num: usize,

    pub y_overlap: usize,

    pub x_start_num: usize,
    pub x_pitch: f64,
    pub x_take_num: usize,

    pub z_lower_limit: i32,
    pub z_upper_limit: i32,
}

impl LjxDataConverterConfig {
    pub fn set_ljx_data_path(&mut self, path: String) {
        self.ljx_data_path = Some(path);
    }
    pub fn check_completeness(&self) -> bool {
        if self.ljx_data_path.is_none() {
            return false;
        }

        true
    }
}

pub fn convert_ljx_data_to_images(config: LjxDataConverterConfig) -> anyhow::Result<()> {
    let converter = ConverterLjxToPly::create(&config)?;
    let mesh_generater = MeshGenerator::create(&config)?;
    let image_generater = ImageGenerator::create(&config)?;
    loop {
        let plz_path = match converter.make_single_plz() {
            Ok(path) => path,
            Err(_err) => break,
        };

        let mesh_path = mesh_generater.make_mesh_from_ply(plz_path)?;
        let image_path = image_generater.generate_from_ply(mesh_path)?;
    }

    Ok(())
}

// 非公開構造体
enum ResultConverterLjxToPly {
    Path(String),
    FinishData,
}
struct ConverterLjxToPly {
    //　読み取り位置を保持する
    reader: LjxDataStreamReader,
    // プロファイルからPCDデータへの変換器
    converter: ProfileToPly,
    // リーダーに投げる変数。
    profile_start: usize,
    // reader生成時に使う
    profile_take_num: usize,
    made_num: usize,

    output_dir: String,
    output_name: String,
}
impl ConverterLjxToPly {
    fn create(config: &LjxDataConverterConfig) -> anyhow::Result<Self> {
        let converter = ProfileToPly::create(&config);
        Ok(Self {})
    }

    // 返り値は生成したplzファイルのパス
    fn make_single_plz(&self) -> anyhow::Result<ResultConverterLjxToPly> {
        let create_file_path = String::new()
            + &self.output_dir
            + &self.output_name
            + "_"
            + &self.made_num.to_string()
            + ".plz";
        let writer = PlyStreamWriter::create(&create_file_path)?;

        // 読み取り地点の調整

        // for _i in 0..self.profile_start {
        //     self.reader.skip_read()?;
        // }

        // この中をエラーハンドリングしたい
        for _i in 0..self.profile_take_num {
            let profile = match self.reader.read_profile() {
                Ok(profile) => profile,
                Err(_) => return Ok(ResultConverterLjxToPly::FinishData),
            };
            let pcd_profile = self.converter.make_points(profile);

            self.writer.write_points(pcd_profile)?;
        }

        Ok(ResultConverterLjxToPly::Path("test".to_string()))
    }
}

struct ProfileToPly {
    next_y: f64,
    y_pitch: f64,
    x_pitch: f64,
    z_lower_limit: i32,
    z_upper_limit: i32,
}
impl ProfileToPly {
    fn create(config: &LjxDataConverterConfig) -> Self {
        Self {
            next_y: 0.0,
            y_pitch: config.y_pitch,
            x_pitch: config.x_pitch,
            z_lower_limit: config.z_lower_limit,
            z_upper_limit: config.z_upper_limit,
        }
    }
    fn make_points(&mut self, profile: Vec<i32>) -> Vec<ProfilePoint> {
        let mut vec = Vec::<ProfilePoint>::new();
        let mut x = 0.0;
        let lower_limit = self.z_lower_limit;
        let upper_limit = self.z_upper_limit;
        for point in profile.iter() {
            let pcd_point = match *point {
                i32::MIN..=lower_limit => ProfilePoint::Failure,
                upper_limit..=i32::MAX => ProfilePoint::Failure,
                _ => ProfilePoint::Success(PlyPoint {
                    x,
                    y: self.next_y,
                    z: f64::from(*point),
                }),
            };
            vec.push(pcd_point);
            x += self.x_pitch;
        }
        self.next_y += self.y_pitch;
        vec
    }
    fn reset_next_y(&mut self) {
        self.next_y = 0.0;
    }
}

enum ProfilePoint {
    Success(PlyPoint),
    Failure,
}
struct PlyPoint {
    x: f64,
    y: f64,
    z: f64,
}
impl PlyPoint {
    fn get_point_binary(&self) -> [u8; 24] {
        let mut buf = [0; 24];
        // let x: [u8; 4] = self.x.to_le_bytes();
        // let y: [u8; 4] = self.y.to_le_bytes();
        // let z: [u8; 4] = self.z.to_le_bytes();
        buf[0..8].copy_from_slice(&self.x.to_le_bytes());
        buf[8..16].copy_from_slice(&self.y.to_le_bytes());
        buf[16..24].copy_from_slice(&self.z.to_le_bytes());
        buf
    }
}

// mesh ////////////////////////////////////////////////////////////////////
struct MeshGenerator {}
impl MeshGenerator {
    fn create(config: &LjxDataConverterConfig) -> anyhow::Result<Self> {
        Ok(Self {})
    }

    // 返り値は生成したplyファイルのパス
    fn make_mesh_from_ply(&self, input_path: String) -> anyhow::Result<String> {
        Ok("test".to_string())
    }
}

struct ImageGenerator {}
impl ImageGenerator {
    fn create(config: &LjxDataConverterConfig) -> anyhow::Result<Self> {
        Ok(Self {})
    }

    // 返り値は生成したplyファイルのパス
    fn generate_from_ply(&self, input_path: String) -> anyhow::Result<String> {
        Ok("test".to_string())
    }
}
