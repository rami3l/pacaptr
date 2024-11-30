use std::sync::LazyLock;

use crate::print::style;

type StyledStr<'a> = console::StyledObject<&'a str>;

pub static CANCELED: LazyLock<StyledStr> = LazyLock::new(|| style::MESSAGE.apply_to("Canceled"));
pub static PENDING: LazyLock<StyledStr> = LazyLock::new(|| style::MESSAGE.apply_to("Pending"));
pub static RUNNING: LazyLock<StyledStr> = LazyLock::new(|| style::MESSAGE.apply_to("Running"));
pub static INFO: LazyLock<StyledStr> = LazyLock::new(|| style::MESSAGE.apply_to("Info"));
pub static ERROR: LazyLock<StyledStr> = LazyLock::new(|| style::ERROR.apply_to("Error"));
