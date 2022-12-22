pub struct ReceiveData {
    pub data:Vec<u8>,
    pub count:u32,
    pub notify:u32,
    pub user:u32,
}



#[derive(Debug)]
pub struct DllInfo {
    pub major_number: i32,
    pub minor_number: i32,
    pub revision_number: i32,
    pub build_number: i32,
}