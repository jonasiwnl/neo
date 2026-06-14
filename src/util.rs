pub fn humansize(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB << 10;
    const GB: u64 = MB << 10;
    const TB: u64 = GB << 10;

    if bytes < KB {
        format!("{}B", bytes)
    } else if bytes < MB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes < TB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.1}TB", bytes as f64 / TB as f64)
    }
}
