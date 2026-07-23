pub mod blocked;
pub mod board;
pub mod epic;
pub mod history;
pub mod hook;
pub mod issue;
pub mod mission;
pub mod note;
pub mod retro;
pub mod retrospective;
pub mod session;
pub mod sprint;
pub mod stalled;
pub mod task;

pub fn unescape_newlines(s: String) -> String {
    s.replace("\\n", "\n")
}

pub fn unescape_newlines_opt(s: Option<String>) -> Option<String> {
    s.map(unescape_newlines)
}
