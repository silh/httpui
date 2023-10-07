/// Simple pretty format for json
///
pub fn pretty_format_json(json: &str) -> String {
    let mut result = String::new();
    let indent_level_increment = 2;
    let mut indent_level = 0;
    let mut in_quotes = false;

    for c in json.chars() {
        match c {
            '{' | '[' => {
                if !in_quotes {
                    result.push(c);
                    result.push('\n');
                    indent_level += indent_level_increment;
                    result.push_str(&"  ".repeat(indent_level));
                } else {
                    result.push(c);
                }
            }
            '}' | ']' => {
                if !in_quotes {
                    result.push('\n');
                    indent_level -= indent_level_increment;
                    result.push_str(&"  ".repeat(indent_level));
                    result.push(c);
                } else {
                    result.push(c);
                }
            }
            '"' => {
                result.push(c);
                in_quotes = !in_quotes;
            }
            ',' => {
                if !in_quotes {
                    result.push(c);
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent_level));
                } else {
                    result.push(c);
                }
            }
            ':' => {
                result.push(c);
                result.push(' ');
            }
            _ => result.push(c),
        }
    }

    result
}