use std::panic::{self, PanicInfo};
use chrono::{DateTime, Local};
use log::{LevelFilter,error};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use log4rs::{
    append::console::{ConsoleAppender, Target},
    filter::threshold::ThresholdFilter,
};

// const NAME: &'static str = env!("CARGO_PKG_DESCRIPTION");

pub fn init() {
    setup_panic();
    setup_logger();
}

fn setup_panic() {
    panic::set_hook(Box::new(move |panic_info: &PanicInfo<'_>| {
        let details = format!("{}", panic_info);
        // println!("println:{}", details);
        error!("{}", details);
    }));
}

fn setup_logger() {
    let now: DateTime<Local> = Local::now();
    // CHECK:ログの名前に実行ファイル名を入れたかったが取得方法が思いつかないので保留
    // examplesの実行ログについて分別を行いたい
    let log_file_path = "log/".to_string() + &now.format("%Y-%m-%d_%H%M%S").to_string() + ".log";
    make_logger(&log_file_path);
}

// #[cfg(debug_assertions)]
// debug,releaseで挙動を変えない
fn make_logger(log_file_path: &str) {
    // println!("デバッグ用のロガー");
    // ファイル・コマンドライン
    let stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(Box::new(PatternEncoder::new("{l} - {f},{L} - {m}\n")))
        .build();

    // ファイル出力の設定
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h({l})},{d},{f},{L},{M},{m},{n}",
        )))
        .build(log_file_path)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Debug),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
}
