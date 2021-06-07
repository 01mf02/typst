use super::*;
use crate::syntax::{HeadingNode, RawNode};

/// `linebreak`: Start a new line.
///
/// # Syntax
/// This function has dedicated syntax:
/// ```typst
/// This line ends here, \
/// And a new one begins.
/// ```
///
/// # Return value
/// A template that inserts a line break.
pub fn linebreak(ctx: &mut EvalContext, _: &mut FuncArgs) -> Value {
    ctx.linebreak();
    Value::None
}

/// `parbreak`: Start a new paragraph.
///
/// # Return value
/// A template that inserts a paragraph break.
pub fn parbreak(ctx: &mut EvalContext, _: &mut FuncArgs) -> Value {
    ctx.parbreak();
    Value::None
}

/// `strong`: Strong text.
///
/// # Syntax
/// This function has dedicated syntax.
/// ```typst
/// This is *important*!
/// ```
///
/// # Positional parameters
/// - Body: optional, of type `template`.
///
/// # Return value
/// A template that flips the boldness of text. The effect is scoped to the
/// body if present.
pub fn strong(ctx: &mut EvalContext, args: &mut FuncArgs) -> Value {
    let body = args.eat::<TemplateValue>(ctx);
    let snapshot = ctx.state.clone();
    ctx.state.font.strong ^= true;

    if let Some(body) = &body {
        body.show(ctx);
        ctx.state = snapshot;
    }

    Value::None
}

/// `emph`: Emphasized text.
///
/// # Syntax
/// This function has dedicated syntax.
/// ```typst
/// I would have _never_ thought so!
/// ```
///
/// # Positional parameters
/// - Body: optional, of type `template`.
///
/// # Return value
/// A template that flips whether text is set in italics. The effect is scoped
/// to the body if present.
pub fn emph(ctx: &mut EvalContext, args: &mut FuncArgs) -> Value {
    let body = args.eat::<TemplateValue>(ctx);
    let snapshot = ctx.state.clone();
    ctx.state.font.emph ^= true;

    if let Some(body) = &body {
        body.show(ctx);
        ctx.state = snapshot;
    }

    Value::None
}

/// `heading`: A section heading.
///
/// # Syntax
/// This function has dedicated syntax.
/// ```typst
/// = Section
/// ...
///
/// == Subsection
/// ...
/// ```
///
/// # Positional parameters
/// - Body, of type `template`.
///
/// # Named parameters
/// - Section depth: `level`, of type `integer` between 1 and 6.
///
/// # Return value
/// A template that sets the body as a section heading, that is, large and in
/// bold.
pub fn heading(ctx: &mut EvalContext, args: &mut FuncArgs) -> Value {
    let level = args.eat_named(ctx, HeadingNode::LEVEL).unwrap_or(1);
    let body = args
        .eat_expect::<TemplateValue>(ctx, HeadingNode::BODY)
        .unwrap_or_default();

    let snapshot = ctx.state.clone();
    let upscale = 1.6 - 0.1 * level as f64;
    ctx.state.font.scale *= upscale;
    ctx.state.font.strong = true;

    body.show(ctx);
    ctx.state = snapshot;

    ctx.parbreak();

    Value::None
}

/// `raw`: Raw text.
///
/// # Syntax
/// This function has dedicated syntax:
/// - For inline-level raw text:
///   ```typst
///   `...`
///   ```
/// - For block-level raw text:
///   ````typst
///   ```rust
///   println!("Hello World!");
///   ```
///   ````
///
/// # Positional parameters
/// - Text, of type `string`.
///
/// # Named parameters
/// - Language for syntax highlighting: `lang`, of type `string`.
/// - Whether the item is block level (split in its own paragraph): `block`, of
///   type `boolean`.
///
/// # Return value
/// A template that sets the text raw, that is, in monospace and optionally with
/// syntax highlighting.
pub fn raw(ctx: &mut EvalContext, args: &mut FuncArgs) -> Value {
    let text = args.eat_expect::<String>(ctx, RawNode::TEXT).unwrap_or_default();
    let _lang = args.eat_named::<String>(ctx, RawNode::LANG);
    let block = args.eat_named(ctx, RawNode::BLOCK).unwrap_or(false);

    if block {
        ctx.parbreak();
    }

    let snapshot = ctx.state.clone();
    ctx.set_monospace();
    ctx.push_text(&text);
    ctx.state = snapshot;

    if block {
        ctx.parbreak();
    }

    Value::None
}
