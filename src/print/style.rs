use std::sync::LazyLock;

use console::Style;

pub static MESSAGE: LazyLock<Style> = LazyLock::new(|| Style::new().green().bold());
pub static ERROR: LazyLock<Style> = LazyLock::new(|| Style::new().bright().red().bold());
pub static QUESTION: LazyLock<Style> = LazyLock::new(|| Style::new().yellow().bold());
