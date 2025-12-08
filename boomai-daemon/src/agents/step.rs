use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepKind {
    Tool,
    Math,
    Reasoning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolKind {
    SystemTime,
    Calculator,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub id: usize,
    pub text: String,
    pub kind: StepKind,
    pub tool: Option<ToolKind>,
}

#[derive(Debug, Clone)]
pub struct ToolSignature {
    pub kind: ToolKind,
    pub patterns: Vec<Regex>,
}

#[derive(Debug, Clone)]
pub struct ToolRegistry {
    pub signatures: Vec<ToolSignature>,
}

impl ToolRegistry {
    pub fn default() -> Self {
        let time_patterns = vec![
            Regex::new(r"\bcurrent (system )?time\b").unwrap(),
            Regex::new(r"\bexact current time\b").unwrap(),
            Regex::new(r"\bwhat time is it\b").unwrap(),
            Regex::new(r"\bnow time\b").unwrap(),
            Regex::new(r"\bsystem time\b").unwrap(),
        ];

        let calc_patterns = vec![
            Regex::new(r"[0-9][0-9\+\-\*/\s\(\)\.]*[0-9]").unwrap(),
            Regex::new(r"\b(add|sum|plus|subtract|minus|times|multiply|divide)\b").unwrap(),
            Regex::new(r"\bcalculate\b").unwrap(),
            Regex::new(r"\bcompute\b").unwrap(),
        ];

        Self {
            signatures: vec![
                ToolSignature { kind: ToolKind::SystemTime, patterns: time_patterns },
                ToolSignature { kind: ToolKind::Calculator, patterns: calc_patterns },
            ],
        }
    }

    pub fn match_tool(&self, text: &str) -> Option<ToolKind> {
        let p = text.to_lowercase();
        let has_digit = p.chars().any(|c| c.is_ascii_digit());
        let has_operator = p.chars().any(|c| "+-*/".contains(c));
        let has_math_word = p.contains("add")
            || p.contains("sum")
            || p.contains("plus")
            || p.contains("subtract")
            || p.contains("minus")
            || p.contains("multiply")
            || p.contains("divide")
            || p.contains("calculate")
            || p.contains("compute")
            || p.contains("times");
        for sig in &self.signatures {
            if sig.patterns.iter().any(|re| re.is_match(&p)) {
                if matches!(sig.kind, ToolKind::Calculator) {
                    if has_digit && (has_operator || has_math_word) {
                        return Some(sig.kind.clone());
                    }
                } else {
                    return Some(sig.kind.clone());
                }
            }
        }
        None
    }
}

#[derive(Default, Debug)]
pub struct ExecutionContext {
    pub step_results: HashMap<usize, String>,
}

#[derive(Debug, Clone)]
pub enum ExecStrategy {
    ToolCall(ToolKind),
    SingleProbe,
    MakerRace { n: usize, k: usize },
}

pub fn classify_step(text: &str, registry: &ToolRegistry) -> Step {
    // reasoning detection
    if is_reasoning_like(text) {
        return Step { id: 0, text: text.to_string(), kind: StepKind::Reasoning, tool: None };
    }

    // tool signatures
    if let Some(tool) = registry.match_tool(text) {
        return Step { id: 0, text: text.to_string(), kind: StepKind::Tool, tool: Some(tool) };
    }

    // if math
    if looks_like_math_instruction(text) {
        return Step {
            id: 0,
            text: text.to_string(),
            kind: StepKind::Math,
            tool: Some(ToolKind::Calculator),
        };
    }

    Step { id: 0, text: text.to_string(), kind: StepKind::Reasoning, tool: None }
}

pub fn decide_strategy(step: &Step) -> ExecStrategy {
    if let Some(tool) = &step.tool {
        return ExecStrategy::ToolCall(tool.clone());
    }

    match step.kind {
        StepKind::Tool => ExecStrategy::SingleProbe,
        StepKind::Math => ExecStrategy::ToolCall(ToolKind::Calculator),
        StepKind::Reasoning => ExecStrategy::MakerRace { n: 5, k: 2 },
    }
}

pub fn looks_like_math_instruction(text: &str) -> bool {
    let lower = text.to_lowercase();
    let has_math_word = lower.contains("sum")
        || lower.contains("add")
        || lower.contains("subtract")
        || lower.contains("multiply")
        || lower.contains("divide")
        || lower.contains("calculate")
        || lower.contains("compute")
        || lower.contains("plus")
        || lower.contains("minus")
        || lower.contains("times")
        || lower.contains("power")
        || lower.contains("sqrt");

    let has_operator = lower.chars().any(|c| "+-*/".contains(c));
    let has_digit = lower.chars().any(|c| c.is_ascii_digit());
    has_digit && (has_math_word || has_operator)
}

pub fn render_step_text(step: &Step, ctx: &ExecutionContext) -> String {
    let mut rendered = step.text.clone();

    // Replace {stepN}
    for (id, val) in &ctx.step_results {
        let placeholder = format!("{{step{}}}", id);
        if rendered.contains(&placeholder) {
            rendered = rendered.replace(&placeholder, val);
        }
    }

    if rendered.contains("{prev}") {
        if let Some((_, last)) = ctx.step_results.iter().last() {
            rendered = rendered.replace("{prev}", last);
        }
    }

    rendered
}

pub fn extract_numeric_from_ctx(ctx: &ExecutionContext) -> Option<f64> {
    ctx.step_results.values().last().and_then(|v| v.trim().parse::<f64>().ok())
}

fn is_reasoning_like(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("pros")
        || lower.contains("cons")
        || lower.contains("list")
        || lower.contains("concise")
        || lower.contains("compare")
        || lower.contains("contrast")
        || lower.contains("explain")
        || lower.contains("summary")
        || lower.contains("summarize")
        || lower.contains("advantages")
        || lower.contains("disadvantages")
        || lower.contains("greater than")
}
