#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::TypeId;
use core::fmt;
use core::fmt::Write;
use rushdown::as_extension_data;
use rushdown::ast::pp_indent;
use rushdown::ast::Arena;
use rushdown::ast::KindData;
use rushdown::ast::NodeKind;
use rushdown::ast::NodeRef;
use rushdown::ast::NodeType;
use rushdown::ast::PrettyPrint;
use rushdown::ast::WalkStatus;
use rushdown::parser;
use rushdown::parser::AnyInlineParser;
use rushdown::parser::InlineParser;
use rushdown::parser::Parser;
use rushdown::parser::ParserExtension;
use rushdown::parser::ParserExtensionFn;
use rushdown::parser::ParserOptions;
use rushdown::parser::PRIORITY_EMPHASIS;
use rushdown::renderer;
use rushdown::renderer::html;
use rushdown::renderer::html::Renderer;
use rushdown::renderer::html::RendererExtension;
use rushdown::renderer::html::RendererExtensionFn;
use rushdown::renderer::BoxRenderNode;
use rushdown::renderer::NodeRenderer;
use rushdown::renderer::NodeRendererRegistry;
use rushdown::renderer::RenderNode;
use rushdown::renderer::RendererOptions;
use rushdown::renderer::TextWrite;
use rushdown::text;
use rushdown::text::Reader;
use rushdown::util::AsciiWordSet;
use rushdown::Result;

// AST {{{

/// Represents an emoji in the AST.
#[derive(Debug)]
pub struct Emoji {
    emoji: &'static emojis::Emoji,
}

impl Emoji {
    /// Creates a new `Emoji` node with the given emoji data.
    pub fn new(emoji: &'static emojis::Emoji) -> Self {
        Self { emoji }
    }

    /// Returns the name of the emoji.
    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.emoji.name()
    }

    /// Returns the first GitHub shortcode for this emoji.
    #[inline(always)]
    pub fn shortcode(&self) -> Option<&str> {
        self.emoji.shortcode()
    }

    /// Returns an iterator over the GitHub shortcodes for this emoji.
    #[inline(always)]
    pub fn shortcodes(&self) -> impl Iterator<Item = &str> + Clone {
        self.emoji.shortcodes()
    }

    /// Returns the Unicode character(s) for this emoji.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self.emoji.as_str()
    }

    /// Returns the Unicode character(s) for this emoji as bytes.
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        self.emoji.as_str().as_bytes()
    }
}

impl NodeKind for Emoji {
    fn typ(&self) -> NodeType {
        NodeType::Inline
    }

    fn kind_name(&self) -> &'static str {
        "Emoji"
    }
}

impl PrettyPrint for Emoji {
    fn pretty_print(&self, w: &mut dyn Write, _source: &str, level: usize) -> fmt::Result {
        writeln!(w, "{}name: {:?}", pp_indent(level), self.emoji.name())?;
        writeln!(
            w,
            "{}shortcodes: {:?}",
            pp_indent(level),
            self.emoji.shortcodes().collect::<Vec<_>>()
        )
    }
}

impl From<Emoji> for KindData {
    fn from(e: Emoji) -> Self {
        KindData::Extension(Box::new(e))
    }
}

// }}} AST

// Parser {{{

/// Options for the emoji parser.
#[derive(Debug, Clone, Default)]
pub struct EmojiParserOptions {
    /// An optional set of shortcodes to ignore when parsing emojis. If provided, any shortcode in
    /// this set will not be parsed as an emoji.
    pub blacklist: Option<Rc<AsciiWordSet>>,
}

impl ParserOptions for EmojiParserOptions {}

#[derive(Debug, Default)]
struct EmojiParser {
    options: EmojiParserOptions,
}

impl EmojiParser {
    fn with_options(options: EmojiParserOptions) -> Self {
        Self { options }
    }
}

impl InlineParser for EmojiParser {
    fn trigger(&self) -> &[u8] {
        b":"
    }

    fn parse(
        &self,
        arena: &mut Arena,
        _parent_ref: NodeRef,
        reader: &mut text::BlockReader,
        _ctx: &mut parser::Context,
    ) -> Option<NodeRef> {
        let (line, _) = reader.peek_line_bytes()?;
        if line.len() < 2 {
            return None;
        }
        let mut i = 1;
        while i < line.len() {
            let c = line[i];
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'+' {
                i += 1;
            } else {
                break;
            }
        }
        if i >= line.len() || line[i] != b':' {
            return None;
        }
        reader.advance(i + 1);
        let shortcode = unsafe { str::from_utf8_unchecked(&line[1..i]) };
        if self
            .options
            .blacklist
            .as_ref()
            .is_some_and(|blacklist| blacklist.contains(shortcode))
        {
            return None;
        }
        emojis::get_by_shortcode(shortcode).map(|emoji| arena.new_node(Emoji::new(emoji)))
    }
}

impl From<EmojiParser> for AnyInlineParser {
    fn from(p: EmojiParser) -> Self {
        AnyInlineParser::Extension(Box::new(p))
    }
}

// }}}

// Renderer {{{

/// Options for the emoji HTML renderer.
#[derive(Debug, Clone, Default)]
pub struct EmojiHtmlRendererOptions {
    /// An optional template string for rendering emojis. If provided, this template will be used
    /// to render emojis instead of the default behavior. The template can include a `{shortcode}`,
    /// `{emoji}`, or `{name}` placeholder.
    pub template: Option<String>,
}

impl RendererOptions for EmojiHtmlRendererOptions {}

struct EmojiHtmlRenderer<W: TextWrite> {
    _phantom: core::marker::PhantomData<W>,
    writer: html::Writer,
    options: EmojiHtmlRendererOptions,
}

impl<W: TextWrite> EmojiHtmlRenderer<W> {
    fn new(html_opts: html::Options, options: EmojiHtmlRendererOptions) -> Self {
        Self {
            _phantom: core::marker::PhantomData,
            writer: html::Writer::with_options(html_opts),
            options,
        }
    }
}

impl<W: TextWrite> RenderNode<W> for EmojiHtmlRenderer<W> {
    fn render_node<'a>(
        &self,
        w: &mut W,
        _source: &'a str,
        arena: &'a Arena,
        node_ref: NodeRef,
        entering: bool,
        _context: &mut renderer::Context,
    ) -> Result<WalkStatus> {
        if entering {
            let emoji = as_extension_data!(arena, node_ref, Emoji);
            match &self.options.template {
                Some(template) => {
                    let rendered = template::render(
                        template,
                        &[
                            ("emoji", emoji.as_str()),
                            ("shortcode", emoji.shortcode().unwrap_or("")),
                            ("name", emoji.name()),
                        ],
                    );
                    self.writer.write_html(w, &rendered)?
                }
                None => self.writer.write_html(w, emoji.as_str())?,
            }
        }
        Ok(WalkStatus::Continue)
    }
}

impl<'cb, W> NodeRenderer<'cb, W> for EmojiHtmlRenderer<W>
where
    W: TextWrite + 'cb,
{
    fn register_node_renderer_fn(self, nrr: &mut impl NodeRendererRegistry<'cb, W>) {
        nrr.register_node_renderer_fn(TypeId::of::<Emoji>(), BoxRenderNode::new(self));
    }
}
// }}} Renderer

// Extension {{{

/// Returns a parser extension that parses emojis.
pub fn emoji_parser_extension(options: EmojiParserOptions) -> impl ParserExtension {
    ParserExtensionFn::new(|p: &mut Parser| {
        p.add_inline_parser(EmojiParser::with_options, options, PRIORITY_EMPHASIS - 100);
    })
}

/// Returns a renderer extension that renders emojis as HTML.
pub fn emoji_html_renderer_extension<'cb, W>(
    options: EmojiHtmlRendererOptions,
) -> impl RendererExtension<'cb, W>
where
    W: TextWrite + 'cb,
{
    RendererExtensionFn::new(move |r: &mut Renderer<'cb, W>| {
        r.add_node_renderer(EmojiHtmlRenderer::new, options);
    })
}

// }}}

// template {{{
mod template {
    use alloc::string::String;

    pub(crate) fn render(tpl: &str, vars: &[(&str, &str)]) -> String {
        let mut out = String::with_capacity(tpl.len());

        let mut i = 0;
        while let Some(open_rel) = tpl[i..].find('{') {
            let open = i + open_rel;
            out.push_str(&tpl[i..open]);

            let rest = &tpl[open + 1..];
            if let Some(close_rel) = rest.find('}') {
                let key = &rest[..close_rel];
                if let Some((_, v)) = vars.iter().find(|(k, _)| *k == key) {
                    out.push_str(v);
                } else {
                    out.push('{');
                    out.push_str(key);
                    out.push('}');
                }
                i = open + 1 + close_rel + 1;
            } else {
                out.push_str(&tpl[open..]);
                return out;
            }
        }

        out.push_str(&tpl[i..]);
        out
    }
}
// }}}
