const TEMPLATE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/md/tutorial.md"));

pub fn render_cli() -> String {
    render("`symposium cargo`", |args| {
        format!("`symposium cargo {args}`")
    })
}

pub fn render_mcp() -> String {
    render("the `rust` tool with a `cargo ...` command", |args| {
        format!("the `rust` tool with command `cargo {args}`")
    })
}

fn render(cargo_sub: &str, example_fn: impl Fn(&str) -> String) -> String {
    let mut output = TEMPLATE.replace("{{CARGO}}", cargo_sub);

    // Replace {{CARGO_EXAMPLE:...}} placeholders
    while let Some(start) = output.find("{{CARGO_EXAMPLE:") {
        let rest = &output[start + "{{CARGO_EXAMPLE:".len()..];
        if let Some(end) = rest.find("}}") {
            let args = &rest[..end];
            let replacement = example_fn(args);
            output = format!("{}{}{}", &output[..start], replacement, &rest[end + 2..]);
        } else {
            break;
        }
    }

    output
}
