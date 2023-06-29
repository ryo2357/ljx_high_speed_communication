use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProfileConverterConfig {
    // .envから取得しないデータはOptionで
    pub profile_path: Option<String>,
    pub output_dir: String,
    // profile_to_pcd
    pub convert_quantity: usize,

    pub y_start_num: usize,
    pub y_pitch: f64,
    pub y_take_num: usize,

    pub y_overlap: usize,

    pub x_start_num: usize,
    pub x_pitch: f64,
    pub x_take_num: usize,
}

impl ProfileConverterConfig {
    pub fn set_profile_path(&mut self, path: String) {
        self.profile_path = Some(path);
    }
    pub fn check_completeness(&self) -> bool {
        if self.profile_path.is_none() {
            return false;
        }

        true
    }
}
