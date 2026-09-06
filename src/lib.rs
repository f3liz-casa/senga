mod compare;
mod render;
pub use compare::{Comparison, compare, report as compare_report, side_by_side};
pub use render::{Options, Page, render, render_with};
