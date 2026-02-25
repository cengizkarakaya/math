
// terminal color codes
pub static RESET: &str = "\x1b[0m";
pub static BOLD: &str = "\x1b[1m";
pub static UNDERLINE: &str = "\x1b[4m";
pub static ITALIC: &str = "\x1b[3m";
pub static DIM: &str = "\x1b[2m";
pub static BRIGHT: &str = "\x1b[0;90m";
pub static BLINK: &str = "\x1b[5m";
pub static CLEAR: &str = "\x1b[2J\x1b[1;1H";
pub static BLUE: &str = "\x1b[0;34m";
pub static GREEN: &str = "\x1b[0;32m";
pub static YELLOW: &str = "\x1b[0;33m";
pub static RED: &str = "\x1b[0;31m";
pub static PURPLE: &str = "\x1b[0;35m";
pub static CYAN: &str = "\x1b[0;36m";
pub static BLACK: &str = "\x1b[0;30m";
pub static WHITE: &str = "\x1b[0;37m";
pub static PALE_TURQUOISE: &str = "\x1b[38;2;175;238;238m";
pub static MEDIUMPURPLE: &str = "\x1b[38;2;135;135;215m";

pub fn ust_cizgi() {
    println!("\n{}{}\n{}", MEDIUMPURPLE, "-".repeat(38), PALE_TURQUOISE);
}

pub fn alt_cizgi() {
    println!("{}\n{}{}\n", MEDIUMPURPLE, "-".repeat(38), RESET);
}

