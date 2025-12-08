use chrono::Local;

/// run internal deterministic stubs
#[allow(dead_code)]
pub fn run_internal_stub(tool: &str, input: &str) -> Option<String> {
    match tool {
        "system_time" => Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        "calculator" => {
            // loosen input: strip non-math tokens so "calculate 42 / 7 + 3" still works
            let sanitized: String = input
                .chars()
                .map(|c| if c.is_ascii_digit() || "+-*/(). ".contains(c) { c } else { ' ' })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            let expr = if sanitized.is_empty() { input } else { sanitized.as_str() };

            if let Ok(v) = evalexpr::eval(expr) {
                if let Ok(f) = v.as_float() {
                    Some(f.to_string())
                } else if let Ok(i) = v.as_int() {
                    Some(i.to_string())
                } else {
                    None
                }
            } else if let Ok(i) = evalexpr::eval_int(expr) {
                Some(i.to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}
