pub fn escape(data: &str) -> String {
    let from = ["_", "*", "[", "]", "(", ")", "~", "`", ">", "#", "+", "-", "=", "|", "{", "}", ".", "!"];
    let to = ["\\_", "\\*", "\\[", "\\]", "\\(", "\\)", "\\~", "\\`", "\\>", "\\#", "\\+", "\\-", "\\=", "\\|", "\\{", "\\}", "\\.", "\\!"];
    let mut res = data.to_owned();
    for c in 0..from.len() {
        res = res.replace(from[c], to[c])
    };
    res
}

pub fn bold(data: &str) -> String {
    format!("*{}*", data)
}

pub fn full_name(first_name: &str, last_name: &str) -> String {
    match last_name.len() {
        0 => {
            first_name.to_owned()
        }
        _ => {
            format!("{} {}", first_name, last_name)
        }
    }
}
