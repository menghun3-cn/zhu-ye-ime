//! 词典构建 CLI 入口骨架。

use zhu_ye_dict::{dict_schema_version, pipeline_status};

fn main() {
    eprintln!("zhu-ye-dict 构建器骨架");
    eprintln!("格式版本: {}", dict_schema_version());
    eprintln!("状态: {}", pipeline_status());
}
