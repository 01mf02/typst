use super::*;

/// `par`: Configure paragraphs.
///
/// # Named parameters
/// - Paragraph spacing: `spacing`, of type `linear` relative to current font size.
/// - Line leading: `leading`, of type `linear` relative to current font size.
/// - Word spacing: `word-spacing`, of type `linear` relative to current font size.
///
/// # Return value
/// A template that configures paragraph properties.
pub fn par(ctx: &mut EvalContext, args: &mut FuncArgs) -> Value {
    let spacing = args.eat_named(ctx, "spacing");
    let leading = args.eat_named(ctx, "leading");
    let word_spacing = args.eat_named(ctx, "word-spacing");

    if let Some(spacing) = spacing {
        ctx.state.par.spacing = spacing;
    }

    if let Some(leading) = leading {
        ctx.state.par.leading = leading;
    }

    if let Some(word_spacing) = word_spacing {
        ctx.state.par.word_spacing = word_spacing;
    }

    ctx.parbreak();

    Value::None
}
