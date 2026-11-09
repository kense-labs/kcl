//! Parse KCL text, inspect the AST, then generate canonical-looking source.

use kcl::{generate, parse, BlockBuilder, DocumentBuilder, boolean, number, string};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"
providers "openai" {
    api_key = env("OPENAI_API_KEY")
    base_url = "https://api.openai.com/v1"
}

default_model = "gpt-4"
"#;

    let document = parse(input)?;
    println!("blocks: {}", document.blocks.len());

    let rebuilt = DocumentBuilder::new()
        .block(
            BlockBuilder::new("settings")
                .attr("temperature", number(0.7))
                .attr("stream", boolean(true))
                .attr("label", string("demo"))
                .build(),
        )
        .build();

    print!("{}", generate(&rebuilt));
    Ok(())
}
