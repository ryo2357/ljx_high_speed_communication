use log::info;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

use std::fs;

use crate::profile_writer;

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

    pub have_brightness: bool,
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
    // 最後にinfoファイルを作る
    let mut converter = ConverterLjxToPly::new(&config)?;
    let mesh_generater = MeshGenerator::new(&config)?;
    let image_generater = ImageGenerator::new(&config)?;

    converter.forward(config.y_start_num)?;
    loop {
        let plz_path = match converter.make_single_plz() {
            Ok(path) => path,
            Err(_err) => break,
        };
        // 本来、シークは遅いため、オーバーラップ分はキャッシュに入れたい
        converter.backward(config.y_overlap)?;

        let mesh_path = mesh_generater.make_mesh_from_ply(plz_path)?;
        let image_path = image_generater.generate_from_ply(mesh_path)?;
    }

    Ok(())
}

// 非公開構造体
struct ConverterLjxToPly {
    //　読み取り位置を保持する
    reader: LjxDataStreamReader,
    // writer: Option<PlyStreamWriter>,
    // プロファイルからPCDデータへの変換器
    converter: ProfileToPly,
    // リーダーに投げる変数。
    // reader生成時に使う
    made_num: usize,
    profile_take_num: usize,

    output_dir: String,
    output_name: String,
}
impl ConverterLjxToPly {
    fn new(config: &LjxDataConverterConfig) -> anyhow::Result<Self> {
        let reader = LjxDataStreamReader::new(&config)?;
        let converter = ProfileToPly::new(&config);

        Ok(Self {
            reader,
            converter,
            made_num: 0,
            profile_take_num: config.y_take_num,
            output_dir: config.output_dir.clone(),
            output_name: config.output_name.clone(),
        })
    }

    // 返り値は生成したplzファイルのパス
    fn make_single_plz(&mut self) -> anyhow::Result<String> {
        self.converter.reset_next_y();

        let create_file_path = String::new()
            + &self.output_dir
            + &self.output_name
            + "_"
            + &self.made_num.to_string()
            + ".plz";
        let mut writer = PlyStreamWriter::new(&create_file_path)?;

        writer.write_header();
        self.stream_convert(&mut writer)?;
        writer.fix_header();

        self.made_num += 1;

        Ok(create_file_path)
    }

    fn stream_convert(&mut self, writer: &mut PlyStreamWriter) -> anyhow::Result<()> {
        for _i in 0..self.profile_take_num {
            let profile = match self.reader.read_profile() {
                Ok(profile) => profile,
                Err(_) => break,
            };
            let pcd_profile = self.converter.make_points(profile);

            writer.write_points(pcd_profile)?;
        }

        Ok(())
    }
    fn forward(&mut self, num: usize) -> anyhow::Result<()> {
        self.reader.forward(num)?;
        Ok(())
    }
    fn backward(&mut self, num: usize) -> anyhow::Result<()> {
        self.backward(num)?;
        Ok(())
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
    fn new(config: &LjxDataConverterConfig) -> Self {
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
        for point in profile.iter() {
            let mut profile_point = ProfilePoint::Failure;
            if (self.z_lower_limit <= *point) && (*point <= self.z_upper_limit) {
                profile_point = ProfilePoint::Success(PlyPoint {
                    x,
                    y: self.next_y,
                    z: f64::from(*point),
                });
            }
            vec.push(profile_point);
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

struct PlyStreamWriter {
    writer: BufWriter<File>,
    point_count: usize,
}

impl PlyStreamWriter {
    fn new(file_path: &str) -> anyhow::Result<Self> {
        let folder_file = std::path::Path::new(&file_path).parent().unwrap();
        fs::create_dir_all(folder_file)?;
        let file = File::create(&file_path)?;
        let writer = BufWriter::new(file);
        Ok(Self {
            writer,
            point_count: 0,
        })
    }

    fn make_header(&self) -> String {
        let point_digits_size: usize = 20 - format!("{:}", self.point_count).to_string().len();
        let adjust_comment = &"xxxxxxxxxxxxxxxxxxxx"[0..point_digits_size];
        let header:String = format!(
            "ply\nformat binary_little_endian 1.0\ncomment adjust str {}\nelement vertex {}\nproperty double x\nproperty double y\nproperty double z\nend_header\n",
            adjust_comment,
            self.point_count
        );
        header
    }

    fn get_point_count(&self) -> usize {
        self.point_count
    }

    fn write_points(&mut self, points: Vec<ProfilePoint>) -> anyhow::Result<()> {
        for pt in points {
            match pt {
                ProfilePoint::Failure => {
                    continue;
                }
                ProfilePoint::Success(point) => {
                    self.writer.write_all(&point.get_point_binary())?;
                    self.point_count += 1;
                }
            }
        }
        Ok(())
    }

    fn write_header(&mut self) -> anyhow::Result<()> {
        let header = self.make_header();
        self.writer.write_all(header.as_bytes())?;
        Ok(())
    }

    fn fix_header(&mut self) -> anyhow::Result<()> {
        let point_num = self.get_point_count();
        let header = self.make_header();

        self.writer.seek(SeekFrom::Start(0))?;
        self.writer.write_all(header.as_bytes())?;
        self.writer.flush()?;
        self.writer.seek(SeekFrom::End(0))?;
        Ok(())
    }
}

struct LjxDataStreamReader {
    reader: BufReader<File>,
    parser: Box<dyn ParseRead>,
}
impl LjxDataStreamReader {
    fn new(config: &LjxDataConverterConfig) -> anyhow::Result<Self> {
        let path = config.ljx_data_path.clone().unwrap();
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let parser: Box<dyn ParseRead> = match &config.have_brightness {
            true => Box::new(LjxBufParseWithBrightness::new(
                config.x_start_num,
                config.x_take_num,
            )?),
            false => Box::new(LjxBufParseNoBrightness::new(
                config.x_start_num,
                config.x_take_num,
            )?),
        };
        Ok(Self { reader, parser })
    }
    fn read_profile(&mut self) -> anyhow::Result<Vec<i32>> {
        let profile = self.parser.parse_read(&mut self.reader)?;
        Ok(profile)
    }

    fn forward(&mut self, num: usize) -> anyhow::Result<()> {
        self.parser.forward_reader(&mut self.reader, num)?;
        Ok(())
    }

    fn backward(&mut self, num: usize) -> anyhow::Result<()> {
        self.parser.backward_reader(&mut self.reader, num)?;
        Ok(())
    }
}

trait ParseRead {
    fn parse_read(&self, reader: &mut BufReader<File>) -> anyhow::Result<Vec<i32>>;
    fn forward_reader(&self, reader: &mut BufReader<File>, num: usize) -> anyhow::Result<()>;
    fn backward_reader(&self, reader: &mut BufReader<File>, num: usize) -> anyhow::Result<()>;
}

struct LjxBufParseWithBrightness {
    start: usize,
    take_num: usize,
}
impl LjxBufParseWithBrightness {
    fn new(start: usize, x_take_num: usize) -> anyhow::Result<Self> {
        // TODO:条件式が本当にあっているか確認が必要
        if start + x_take_num > 3200 {
            anyhow::bail!("RowDataToProfileの入力値が不正")
        }
        Ok(Self {
            start,
            take_num: x_take_num,
        })
    }
}
impl ParseRead for LjxBufParseWithBrightness {
    fn parse_read(&self, reader: &mut BufReader<File>) -> anyhow::Result<Vec<i32>> {
        let mut buf = [0; (3200 + 3200 + 4) * 4];

        let _len = reader.read(&mut buf)?;
        // len == 0　でエラーハンドリングするべき?

        let iter = buf.chunks(4).skip(4).skip(self.start).take(self.take_num);
        let mut vec = Vec::<i32>::new();
        for (i, buf) in iter.enumerate() {
            if i == 3200 {
                break;
            }
            vec.push(i32::from_le_bytes(buf.try_into()?));
            // 単位は100nmになる 0.1μm
        }
        Ok(vec)
    }
    fn forward_reader(&self, reader: &mut BufReader<File>, num: usize) -> anyhow::Result<()> {
        let mut buf = [0; (3200 + 3200 + 4) * 4];
        for _i in 0..num {
            let _len = reader.read(&mut buf)?;
        }
        Ok(())
    }
    fn backward_reader(&self, reader: &mut BufReader<File>, num: usize) -> anyhow::Result<()> {
        let backward_num: i64 = -((3200 + 3200 + 4) * 4) * num as i64;
        let _len = reader.seek(SeekFrom::Current(backward_num))?;
        Ok(())
    }
}

struct LjxBufParseNoBrightness {
    start: usize,
    take_num: usize,
}
impl LjxBufParseNoBrightness {
    fn new(start: usize, x_take_num: usize) -> anyhow::Result<Self> {
        // TODO:条件式が本当にあっているか確認が必要
        if start + x_take_num > 3200 {
            anyhow::bail!("RowDataToProfileの入力値が不正")
        }
        Ok(Self {
            start,
            take_num: x_take_num,
        })
    }
}
impl ParseRead for LjxBufParseNoBrightness {
    fn parse_read(&self, reader: &mut BufReader<File>) -> anyhow::Result<Vec<i32>> {
        // 輝度なしの場合、[0; (3200 + 4) * 4]
        let mut buf = [0; (3200 + 4) * 4];

        let _len = reader.read(&mut buf)?;
        // len == 0　でエラーハンドリングするべき?

        let iter = buf.chunks(4).skip(4).skip(self.start).take(self.take_num);
        let mut vec = Vec::<i32>::new();
        for (i, buf) in iter.enumerate() {
            if i == 3200 {
                break;
            }
            vec.push(i32::from_le_bytes(buf.try_into()?));
            // 単位は100nmになる 0.1μm
        }
        Ok(vec)
    }
    fn forward_reader(&self, reader: &mut BufReader<File>, num: usize) -> anyhow::Result<()> {
        let mut buf = [0; (3200 + 4) * 4];
        for _i in 0..num {
            let _len = reader.read(&mut buf)?;
        }
        Ok(())
    }
    fn backward_reader(&self, reader: &mut BufReader<File>, num: usize) -> anyhow::Result<()> {
        let backward_num: i64 = -((3200 + 4) * 4) * num as i64;
        let _len = reader.seek(SeekFrom::Current(backward_num))?;
        Ok(())
    }
}

// mesh ////////////////////////////////////////////////////////////////////
struct MeshGenerator {}
impl MeshGenerator {
    fn new(config: &LjxDataConverterConfig) -> anyhow::Result<Self> {
        Ok(Self {})
    }

    // 返り値は生成したplyファイルのパス
    fn make_mesh_from_ply(&self, input_path: String) -> anyhow::Result<String> {
        Ok("test".to_string())
    }
}

struct ImageGenerator {}
impl ImageGenerator {
    fn new(config: &LjxDataConverterConfig) -> anyhow::Result<Self> {
        Ok(Self {})
    }

    // 返り値は生成したplyファイルのパス
    fn generate_from_ply(&self, input_path: String) -> anyhow::Result<String> {
        Ok("test".to_string())
    }
}
