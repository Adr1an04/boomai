use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

static TIME_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
static CALC_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepKind {
    Tool,
    Math,
    Reasoning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub trait ToolMatcher: Send + Sync {
    fn kind(&self) -> ToolKind;
    fn matches(&self, text: &str) -> bool;
}

struct SystemTimeMatcher {
    patterns: &'static [Regex],
}

impl SystemTimeMatcher {
    fn new() -> Self {
        Self { patterns: time_patterns() }
    }
}

impl ToolMatcher for SystemTimeMatcher {
    fn kind(&self) -> ToolKind {
        ToolKind::SystemTime
    }

    fn matches(&self, text: &str) -> bool {
        self.patterns.iter().any(|re| re.is_match(text))
    }
}

struct CalculatorMatcher {
    patterns: &'static [Regex],
}

impl CalculatorMatcher {
    fn new() -> Self {
        Self { patterns: calc_patterns() }
    }
}

impl ToolMatcher for CalculatorMatcher {
    fn kind(&self) -> ToolKind {
        ToolKind::Calculator
    }

    fn matches(&self, text: &str) -> bool {
        let has_digit = text.chars().any(|c| c.is_ascii_digit());
        let has_operator = text.chars().any(|c| "+-*/".contains(c));
        let has_math_word = text.contains("add")
            || text.contains("sum")
            || text.contains("plus")
            || text.contains("subtract")
            || text.contains("minus")
            || text.contains("multiply")
            || text.contains("divide")
            || text.contains("calculate")
            || text.contains("compute")
            || text.contains("times");

        self.patterns.iter().any(|re| re.is_match(text))
            && has_digit
            && (has_operator || has_math_word)
    }
}

pub struct ToolRegistry {
    pub matchers: Vec<Box<dyn ToolMatcher>>,
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry").field("matchers_len", &self.matchers.len()).finish()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self {
            matchers: vec![Box::new(SystemTimeMatcher::new()), Box::new(CalculatorMatcher::new())],
        }
    }
}

impl ToolRegistry {
    pub fn match_tool(&self, text: &str) -> Option<ToolKind> {
        let p = text.to_lowercase();
        self.matchers.iter().find_map(|matcher| matcher.matches(&p).then_some(matcher.kind()))
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
        return ExecStrategy::ToolCall(*tool);
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

fn time_patterns() -> &'static [Regex] {
    TIME_PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"\bcurrent (system )?time\b").expect("valid system time regex"),
            Regex::new(r"\bexact current time\b").expect("valid exact time regex"),
            Regex::new(r"\bwhat time is it\b").expect("valid time question regex"),
            Regex::new(r"\bnow time\b").expect("valid now time regex"),
            Regex::new(r"\bsystem time\b").expect("valid system time regex"),
        ]
    })
}

fn calc_patterns() -> &'static [Regex] {
    CALC_PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"[0-9][0-9\+\-\*/\s\(\)\.]*[0-9]").expect("valid math expression regex"),
            Regex::new(r"\b(add|sum|plus|subtract|minus|times|multiply|divide)\b")
                .expect("valid math verb regex"),
            Regex::new(r"\bcalculate\b").expect("valid calculate regex"),
            Regex::new(r"\bcompute\b").expect("valid compute regex"),
        ]
    })
}
