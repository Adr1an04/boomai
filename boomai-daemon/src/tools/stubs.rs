use chrono::Local;

/// Deterministic internal tools with no external side effects.
pub fn run_internal_stub(tool: &str, input: Option<&str>) -> Option<String> {
    match tool {
        "system_time" => Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        "calculator" => {
            let source = input.unwrap_or_default();
            let sanitized: String = source
                .chars()
                .map(|c| if c.is_ascii_digit() || "+-*/(). ".contains(c) { c } else { ' ' })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            let expr = if sanitized.is_empty() { source } else { sanitized.as_str() };
            if expr.trim().is_empty() {
                return None;
            }

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
