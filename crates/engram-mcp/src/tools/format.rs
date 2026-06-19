use serde_json::Value;

pub fn make_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    
    // Headers
    out.push_str("| ");
    out.push_str(&headers.join(" | "));
    out.push_str(" |\n");

    // Divider
    out.push_str("| ");
    out.push_str(&headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
    out.push_str(" |\n");

    // Rows
    for row in rows {
        out.push_str("| ");
        out.push_str(&row.join(" | "));
        out.push_str(" |\n");
    }

    out
}

pub fn make_details(title: &str, fields: &[(&str, String)]) -> String {
    let mut out = format!("### {}\n", title);
    for (k, v) in fields {
        out.push_str(&format!("- **{}**: {}\n", k, v));
    }
    out
}

pub fn success(msg: &str) -> Value {
    Value::String(format!("Success: {}", msg))
}
