use common::config::Settings;

mod storage;
mod subscriber;

pub fn get_settings() -> Settings {
    let settings = Settings::new("../../config").expect("Failed to read config");
    let _ = env_logger::Builder::new().parse_filters(&settings.log_filter).try_init();
    settings
}
