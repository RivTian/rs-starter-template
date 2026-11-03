use std::path::PathBuf;

#[cfg(target_os = "linux")]
const BASE_DIR: &str = "/opt/rs-starter-template/bin";

const SERVICE_NAME: &str = "rs-starter-template";

pub fn get_base_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "linux")]
    {
        Ok(PathBuf::from(BASE_DIR))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {e}"))?;
        exe_path
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| "Failed to get parent directory of the executable".to_string())
    }
}

/// 获取项目 ROOT 路径（BASE_DIR 的上一级）
///
/// 例：程序位于 `.../rs-starter-template/bin/rs-starter-template`
/// → base_dir = `.../rs-starter-template/bin`
/// → root     = `.../rs-starter-template`
fn get_root_dir() -> Result<PathBuf, String> {
    let base = get_base_dir()?;
    base.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "Failed to get project root directory (parent of base_dir)".to_string())
}

/// 日志目录：`<ROOT>/logs/<SERVICE_NAME>`
pub fn get_log_dir() -> Result<PathBuf, String> {
    Ok(get_root_dir()?.join("logs").join(SERVICE_NAME))
}

/// 配置目录：`<ROOT>/configs/<SERVICE_NAME>`
pub fn get_config_dir() -> Result<PathBuf, String> {
    Ok(get_root_dir()?.join("configs").join(SERVICE_NAME))
}

/// 数据目录：`<ROOT>/data/<SERVICE_NAME>`
///
/// 存放数据库文件、迁移前备份等持久化数据
pub fn get_data_dir() -> Result<PathBuf, String> {
    Ok(get_root_dir()?.join("data").join(SERVICE_NAME))
}
