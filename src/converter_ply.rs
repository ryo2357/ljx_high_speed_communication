use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
pub struct LjxDataConverterConfig {
    // .envから取得しないデータはOptionで
    pub ljx_data_path: Option<String>,
    pub output_dir: String,
    pub output_name: String,

    // まだつかってない
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
pub fn convert_ljx_data_to_ply(config: LjxDataConverterConfig) -> anyhow::Result<()> {
    // ディレクトリが存在していたらエラー処理
    // 後々.plyをglobするため変換はディレクトリ単位で管理したい
    let output_dir = Path::new(&config.output_dir);

    match output_dir.is_dir() {
        true => {
            error!(".envで設定されている出力フォルダがすでに存在する");
            anyhow::bail!("出力フォルダがすでに存在する")
        }
        false => {}
    }

    fs::create_dir_all(output_dir)?;
    // 最後にinfoファイルを作る
    let info_logger = InformationLogger::new(&config)?;
    let mut converter = ConverterLjxToPly::new(&config, info_logger)?;

    converter.forward(config.y_start_num)?;

    for i in 0..config.convert_quantity {
        let result = converter.make_single_ply()?;

        match result {
            StreamConvertResult::CanContinue => {
                info!("No.{:?} is done", i)
            }
            StreamConvertResult::Terminal => {
                info!("convert_ljx_data_to_ply()のループ内でデータ末端を検知");
                break;
            }
        }
        converter.backward(config.y_overlap)?;
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
    // infoへの追記
    logger: InformationLogger,
    first_trigger_count: u32,
    last_trigger_count: u32,
    is_first_profile: bool,
}
enum StreamConvertResult {
    Terminal,
    CanContinue,
}
impl ConverterLjxToPly {
    fn new(config: &LjxDataConverterConfig, logger: InformationLogger) -> anyhow::Result<Self> {
        let reader = LjxDataStreamReader::new(config)?;
        let converter = ProfileToPly::new(config);

        Ok(Self {
            reader,
            converter,
            made_num: 0,
            profile_take_num: config.y_take_num,
            output_dir: config.output_dir.clone(),
            output_name: config.output_name.clone(),
            logger,
            first_trigger_count: 0,
            last_trigger_count: 0,
            is_first_profile: true,
        })
    }

    // 返り値は生成したplyファイルのパス
    fn make_single_ply(&mut self) -> anyhow::Result<StreamConvertResult> {
        self.converter.reset_next_y();
        self.first_trigger_count = 0;
        self.last_trigger_count = 0;
        self.is_first_profile = true;

        let file_name =
            String::new() + &self.output_name + "_" + &self.made_num.to_string() + ".ply";
        let create_file_path = String::new() + &self.output_dir + &file_name;
        fs::create_dir_all(&self.output_dir)?;
        let mut writer = PlyStreamWriter::new(&create_file_path)?;

        writer.write_header()?;
        let result = self.stream_convert(&mut writer)?;
        writer.fix_header()?;

        self.made_num += 1;

        self.logger.write_convert_success(
            file_name,
            self.first_trigger_count,
            self.last_trigger_count,
        )?;

        Ok(result)
    }

    fn stream_convert(
        &mut self,
        writer: &mut PlyStreamWriter,
    ) -> anyhow::Result<StreamConvertResult> {
        // ここで読み出しできない場合、エラーが発生する用にする必用がある

        for _i in 0..self.profile_take_num {
            let result = self.reader.read_profile()?;
            let (trigger_count, profile) = match result {
                ProfileReadResult::Terminal => return Ok(StreamConvertResult::Terminal),
                ProfileReadResult::Data(data) => (data.trigger_count, data.row_profile),
            };
            let pcd_profile = self.converter.make_points(profile);
            writer.write_points(pcd_profile)?;

            if self.is_first_profile {
                self.first_trigger_count = trigger_count;
                self.is_first_profile = false;
            } else {
                self.last_trigger_count = trigger_count;
            }
        }

        Ok(StreamConvertResult::CanContinue)
    }
    fn forward(&mut self, num: usize) -> anyhow::Result<()> {
        self.reader.forward(num)?;
        Ok(())
    }
    fn backward(&mut self, num: usize) -> anyhow::Result<()> {
        self.reader.backward(num)?;
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
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        Ok(Self {
            writer,
            point_count: 0,
        })
    }

    fn make_header(&self) -> String {
        let point_digits_size: usize = 20 - format!("{:}", self.point_count).len();
        let adjust_comment = &"xxxxxxxxxxxxxxxxxxxx"[0..point_digits_size];
        let header:String = format!(
            "ply\nformat binary_little_endian 1.0\ncomment adjust str {}\nelement vertex {}\nproperty double x\nproperty double y\nproperty double z\nend_header\n",
            adjust_comment,
            self.point_count
        );
        header
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
    // デバッグ用
    #[allow(dead_code)]
    fn read_header(&mut self) -> anyhow::Result<Vec<i32>> {
        let profile = self.parser.parse_header(&mut self.reader)?;
        Ok(profile)
    }
    fn read_profile(&mut self) -> anyhow::Result<ProfileReadResult> {
        let result = self.parser.parse_read(&mut self.reader)?;
        Ok(result)
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

enum ProfileReadResult {
    Terminal,
    Data(ProfileData),
}
struct ProfileData {
    trigger_count: u32,
    row_profile: Vec<i32>,
}
trait ParseRead {
    // トリガーカウントとプロファイルデータを返却する
    fn parse_read(&self, reader: &mut BufReader<File>) -> anyhow::Result<ProfileReadResult>;
    fn forward_reader(&self, reader: &mut BufReader<File>, num: usize) -> anyhow::Result<()>;
    fn backward_reader(&self, reader: &mut BufReader<File>, num: usize) -> anyhow::Result<()>;
    // デバッグ用
    fn parse_header(&self, reader: &mut BufReader<File>) -> anyhow::Result<Vec<i32>>;
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
    fn parse_read(&self, reader: &mut BufReader<File>) -> anyhow::Result<ProfileReadResult> {
        let mut buf = [0; (3200 + 3200 + 4) * 4];

        let len = reader.read(&mut buf)?;
        // 基本的にプロファイル単位で保存されているので0 or 25616 になるはず
        if len == 0 {
            info!("parse_read: {:?}", len);
            return Ok(ProfileReadResult::Terminal);
        }
        // info!("1");
        // デバッグ用コードを含む
        let before_len = buf.len();
        // info!("2:{:?}", before_len);
        let trigger_buf = &buf[4..8];

        // info!("trigger_buf:{:?}", trigger_buf);
        let trigger_count = u32::from_le_bytes(trigger_buf.try_into()?);
        // info!("4");
        let after_len = buf.len();
        // info!("4:{:?}", after_len);

        if before_len != after_len {
            error!("before_len: {:?},after_ken:{:?}", before_len, after_len);
            panic!();
        }

        let iter = buf.chunks(4).skip(4).skip(self.start).take(self.take_num);
        let mut row_profile = Vec::<i32>::new();
        for (i, buf) in iter.enumerate() {
            if i == 3200 {
                break;
            }
            row_profile.push(i32::from_le_bytes(buf.try_into()?));
            // 単位は100nmになる 0.1μm
        }
        Ok(ProfileReadResult::Data(ProfileData {
            trigger_count,
            row_profile,
        }))
    }
    fn forward_reader(&self, reader: &mut BufReader<File>, num: usize) -> anyhow::Result<()> {
        let mut buf = [0; (3200 + 3200 + 4) * 4];
        for _i in 0..num {
            let _len = reader.read(&mut buf)?;
        }
        Ok(())
    }
    // 多分ここでバグってる
    fn backward_reader(&self, reader: &mut BufReader<File>, num: usize) -> anyhow::Result<()> {
        let backward_num: i64 = -((3200 + 3200 + 4) * 4) * num as i64;
        let _len = reader.seek(SeekFrom::Current(backward_num))?;
        Ok(())
    }

    fn parse_header(&self, reader: &mut BufReader<File>) -> anyhow::Result<Vec<i32>> {
        let mut buf = [0; (3200 + 3200 + 4) * 4];
        let _len = reader.read(&mut buf)?;
        let iter = buf.chunks(4);
        let mut vec = Vec::<i32>::new();
        for (i, buf) in iter.enumerate() {
            if i == 4 {
                break;
            }
            vec.push(i32::from_le_bytes(buf.try_into()?));
            // 単位は100nmになる 0.1μm
        }
        Ok(vec)
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
    fn parse_read(&self, reader: &mut BufReader<File>) -> anyhow::Result<ProfileReadResult> {
        // 輝度なしの場合、[0; (3200 + 4) * 4]
        let mut buf = [0; (3200 + 4) * 4];

        let len = reader.read(&mut buf)?;
        if len == 0 {
            info!("末端に到着");
            return Ok(ProfileReadResult::Terminal);
        }

        let trigger_count = u32::from_le_bytes(buf[4..7].try_into()?);

        let iter = buf.chunks(4).skip(4).skip(self.start).take(self.take_num);
        let mut row_profile = Vec::<i32>::new();
        for (i, buf) in iter.enumerate() {
            if i == 3200 {
                break;
            }
            row_profile.push(i32::from_le_bytes(buf.try_into()?));
            // 単位は100nmになる 0.1μm
        }
        Ok(ProfileReadResult::Data(ProfileData {
            trigger_count,
            row_profile,
        }))
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
    fn parse_header(&self, reader: &mut BufReader<File>) -> anyhow::Result<Vec<i32>> {
        let mut buf = [0; (3200 + 4) * 4];
        let _len = reader.read(&mut buf)?;
        let iter = buf.chunks(4);
        let mut vec = Vec::<i32>::new();
        for (i, buf) in iter.enumerate() {
            if i == 4 {
                break;
            }
            vec.push(i32::from_le_bytes(buf.try_into()?));
            // 単位は100nmになる 0.1μm
        }
        Ok(vec)
    }
}

struct InformationLogger {
    file: File,
}
impl InformationLogger {
    fn new(config: &LjxDataConverterConfig) -> anyhow::Result<Self> {
        let info_file_path = String::new() + &config.output_dir + &config.output_name + "_info.txt";
        let mut file = File::create(info_file_path)?;

        // Configの書き込み
        writeln!(file, "[LjxDataConverterConfig]")?;
        let toml_config = toml::to_string(&config)?;
        write!(file, "{}", toml_config)?;
        writeln!(file)?;

        Ok(Self { file })
    }

    fn write_convert_success(
        &mut self,
        file_name: String,
        first: u32,
        last: u32,
    ) -> anyhow::Result<()> {
        let num = last - first + 1;
        writeln!(self.file, "[{}]", file_name)?;
        writeln!(self.file, "{:?} profiles from {:?}", num, first)?;
        Ok(())
    }
}
