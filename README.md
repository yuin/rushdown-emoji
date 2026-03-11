# rushdown-emoji
rushdown-emoji is an extension for the rushdown that parses :joy: style emojis.

## Installation
Add dependency to your `Cargo.toml`:

```toml
[dependencies]
rushdown-emoji = "x.y.z"
```

rushdown-emoji can also be used in `no_std` environments. To enable this feature, add the following line to your `Cargo.toml`:

```toml
rushdown-emoji = { version = "x.y.z", default-features = false, features = ["no-std"] }
```

## Syntax

```markdown
I am :joy: emoji.
```

## Usage
### Example

```rust
use core::fmt::Write;
use rushdown::{
    new_markdown_to_html,
    parser::{self, ParserExtension},
    renderer::html::{self, RendererExtension},
    Result,
};
use rushdown_emoji::{
    emoji_html_renderer_extension, emoji_parser_extension, 
    EmojiParserOptions,
    EmojiHtmlRendererOptions,
};

let markdown_to_html = new_markdown_to_html(
    parser::Options::default(),
    html::Options::default(),
    emoji_parser_extension(EmojiParserOptions::default()),
    emoji_html_renderer_extension(EmojiHtmlRendererOptions::default()),
);
let mut output = String::new();
let input = r#"
I am :joy: emoji.
"#;
match markdown_to_html(&mut output, input) {
    Ok(_) => {
        println!("HTML output:\n{}", output);
    }
    Err(e) => {
        println!("Error: {:?}", e);
    }
}
```

### Options
#### Parser options

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| `blacklist`| `Option<Rc<AsciiWordSet>>` | `None` | A set of emoji shortcodes that should not be parsed as emojis. |

#### HTML renderer options

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| `template` | `Option<String>` | `None` | A template string for rendering emojis. The template can include `{emoji}`, `{shortcode}` and `{name}` which will be replaced with the actual data. If `None`, the default template is used, which simply outputs the emoji character. |

## Donation
BTC: 1NEDSyUmo4SMTDP83JJQSWi1MvQUGGNMZB

Github sponsors also welcome.

## License
MIT

## Author
Yusuke Inuzuka
