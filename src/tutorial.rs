const TEMPLATE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/md/tutorial.md"));

pub fn render_cli() -> String {
    TEMPLATE.to_string()
}

pub fn render_mcp() -> String {
    TEMPLATE.to_string()
}
