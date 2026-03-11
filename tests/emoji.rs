use std::rc::Rc;

use rushdown::{
    new_markdown_to_html_string,
    parser::{self},
    renderer::html,
    test::{MarkdownTestCase, MarkdownTestCaseOptions},
    util::AsciiWordSet,
};
use rushdown_emoji::{
    emoji_html_renderer_extension, emoji_parser_extension, EmojiHtmlRendererOptions,
    EmojiParserOptions,
};

#[test]
fn test_emoji() {
    let source = r#"
I'm :joy: `:joy:`
"#;
    let markdown_to_html = new_markdown_to_html_string(
        parser::Options::default(),
        html::Options {
            allows_unsafe: true,
            xhtml: false,
            ..html::Options::default()
        },
        emoji_parser_extension(EmojiParserOptions::default()),
        emoji_html_renderer_extension(EmojiHtmlRendererOptions::default()),
    );
    MarkdownTestCase::new(
        1,
        String::from("ok"),
        String::from(source),
        String::from(
            r#"<p>I'm 😂 <code>:joy:</code></p>
"#,
        ),
        MarkdownTestCaseOptions::default(),
    )
    .execute(&markdown_to_html);
}

#[test]
fn test_emoji_not_exists() {
    let source = r#"
I'm :joyjoy:
"#;
    let markdown_to_html = new_markdown_to_html_string(
        parser::Options::default(),
        html::Options {
            allows_unsafe: true,
            xhtml: false,
            ..html::Options::default()
        },
        emoji_parser_extension(EmojiParserOptions::default()),
        emoji_html_renderer_extension(EmojiHtmlRendererOptions::default()),
    );
    MarkdownTestCase::new(
        1,
        String::from("ok"),
        String::from(source),
        String::from(
            r#"<p>I'm :joyjoy:</p>
"#,
        ),
        MarkdownTestCaseOptions::default(),
    )
    .execute(&markdown_to_html);
}

#[test]
fn test_emoji_blacklist() {
    let source = r#"
I'm :joy:
"#;
    let bl = Rc::new(AsciiWordSet::new("joy"));

    let markdown_to_html = new_markdown_to_html_string(
        parser::Options::default(),
        html::Options {
            allows_unsafe: true,
            xhtml: false,
            ..html::Options::default()
        },
        emoji_parser_extension(EmojiParserOptions {
            blacklist: Some(bl.clone()),
            ..Default::default()
        }),
        emoji_html_renderer_extension(EmojiHtmlRendererOptions::default()),
    );
    MarkdownTestCase::new(
        1,
        String::from("ok"),
        String::from(source),
        String::from(
            r#"<p>I'm :joy:</p>
"#,
        ),
        MarkdownTestCaseOptions::default(),
    )
    .execute(&markdown_to_html);
}

#[test]
fn test_emoji_template() {
    let source = r#"
I'm :joy:
"#;
    let markdown_to_html = new_markdown_to_html_string(
        parser::Options::default(),
        html::Options {
            allows_unsafe: true,
            xhtml: false,
            ..html::Options::default()
        },
        emoji_parser_extension(EmojiParserOptions::default()),
        emoji_html_renderer_extension(EmojiHtmlRendererOptions {
            template: Some(String::from(
                "<img src=\"https://image.example.com/{shortcode}.png\" />",
            )),
            ..Default::default()
        }),
    );
    MarkdownTestCase::new(
        1,
        String::from("ok"),
        String::from(source),
        String::from(
            r#"<p>I'm <img src="https://image.example.com/joy.png" /></p>
"#,
        ),
        MarkdownTestCaseOptions::default(),
    )
    .execute(&markdown_to_html);
}
