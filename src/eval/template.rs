use std::convert::TryFrom;
use std::fmt::{self, Debug, Formatter};
use std::mem;
use std::ops::{Add, AddAssign};
use std::rc::Rc;

use crate::diag::StrResult;
use crate::geom::{Align, Dir, Gen, GenAxis, Length, Linear, Sides, Size};
use crate::layout::{
    Decoration, LayoutNode, LayoutTree, PadNode, PageRun, ParChild, ParNode, StackChild,
    StackNode,
};
use crate::style::Style;
use crate::util::EcoString;

/// A template value: `[*Hi* there]`.
#[derive(Default, Clone)]
pub struct Template(Rc<Vec<TemplateNode>>);

/// One node in a template.
#[derive(Clone)]
enum TemplateNode {
    /// A word space.
    Space(Vec<Decoration>),
    /// A line break.
    Linebreak,
    /// A paragraph break.
    Parbreak,
    /// A page break.
    Pagebreak(bool),
    /// Plain text.
    Text(EcoString, Vec<Decoration>),
    /// Spacing.
    Spacing(GenAxis, Linear),
    /// An inline node builder.
    Inline(Rc<dyn Fn(&Style) -> LayoutNode>, Vec<Decoration>),
    /// An block node builder.
    Block(Rc<dyn Fn(&Style) -> LayoutNode>),
    /// Save the current style.
    Save,
    /// Restore the last saved style.
    Restore,
    /// A function that can modify the current style.
    Modify(Rc<dyn Fn(&mut Style)>),
}

impl Template {
    /// Create a new, empty template.
    pub fn new() -> Self {
        Self(Rc::new(vec![]))
    }

    /// Create a template from a builder for an inline-level node.
    pub fn from_inline<F, T>(f: F) -> Self
    where
        F: Fn(&Style) -> T + 'static,
        T: Into<LayoutNode>,
    {
        let node = TemplateNode::Inline(Rc::new(move |s| f(s).into()), vec![]);
        Self(Rc::new(vec![node]))
    }

    /// Create a template from a builder for a block-level node.
    pub fn from_block<F, T>(f: F) -> Self
    where
        F: Fn(&Style) -> T + 'static,
        T: Into<LayoutNode>,
    {
        let node = TemplateNode::Block(Rc::new(move |s| f(s).into()));
        Self(Rc::new(vec![node]))
    }

    /// Add a word space to the template.
    pub fn space(&mut self) {
        self.make_mut().push(TemplateNode::Space(vec![]));
    }

    /// Add a line break to the template.
    pub fn linebreak(&mut self) {
        self.make_mut().push(TemplateNode::Linebreak);
    }

    /// Add a paragraph break to the template.
    pub fn parbreak(&mut self) {
        self.make_mut().push(TemplateNode::Parbreak);
    }

    /// Add a page break to the template.
    pub fn pagebreak(&mut self, keep: bool) {
        self.make_mut().push(TemplateNode::Pagebreak(keep));
    }

    /// Add text to the template.
    pub fn text(&mut self, text: impl Into<EcoString>) {
        self.make_mut().push(TemplateNode::Text(text.into(), vec![]));
    }

    /// Add text, but in monospace.
    pub fn monospace(&mut self, text: impl Into<EcoString>) {
        self.save();
        self.modify(|style| style.text_mut().monospace = true);
        self.text(text);
        self.restore();
    }

    /// Add spacing along an axis.
    pub fn spacing(&mut self, axis: GenAxis, spacing: Linear) {
        self.make_mut().push(TemplateNode::Spacing(axis, spacing));
    }

    /// Add a decoration to all contained nodes.
    pub fn decorate(&mut self, deco: Decoration) {
        for node in self.make_mut() {
            let decos = match node {
                TemplateNode::Space(decos) => decos,
                TemplateNode::Text(_, decos) => decos,
                TemplateNode::Inline(_, decos) => decos,
                _ => continue,
            };
            decos.push(deco.clone());
        }
    }

    /// Register a restorable snapshot.
    pub fn save(&mut self) {
        self.make_mut().push(TemplateNode::Save);
    }

    /// Ensure that later nodes are untouched by style modifications made since
    /// the last snapshot.
    pub fn restore(&mut self) {
        self.make_mut().push(TemplateNode::Restore);
    }

    /// Modify the style.
    pub fn modify<F>(&mut self, f: F)
    where
        F: Fn(&mut Style) + 'static,
    {
        self.make_mut().push(TemplateNode::Modify(Rc::new(f)));
    }

    /// Return a new template which is modified from start to end.
    pub fn modified<F>(self, f: F) -> Self
    where
        F: Fn(&mut Style) + 'static,
    {
        let mut wrapper = Self::new();
        wrapper.save();
        wrapper.modify(f);
        wrapper += self;
        wrapper.restore();
        wrapper
    }

    /// Build the stack node resulting from instantiating the template with the
    /// given style.
    pub fn to_stack(&self, style: &Style) -> StackNode {
        let mut builder = Builder::new(style, false);
        builder.template(self);
        builder.build_stack()
    }

    /// Build the layout tree resulting from instantiating the template with the
    /// given style.
    pub fn to_tree(&self, style: &Style) -> LayoutTree {
        let mut builder = Builder::new(style, true);
        builder.template(self);
        builder.build_tree()
    }

    /// Repeat this template `n` times.
    pub fn repeat(&self, n: i64) -> StrResult<Self> {
        let count = usize::try_from(n)
            .ok()
            .and_then(|n| self.0.len().checked_mul(n))
            .ok_or_else(|| format!("cannot repeat this template {} times", n))?;

        Ok(Self(Rc::new(
            self.0.iter().cloned().cycle().take(count).collect(),
        )))
    }

    /// Return a mutable reference to the inner vector.
    fn make_mut(&mut self) -> &mut Vec<TemplateNode> {
        Rc::make_mut(&mut self.0)
    }
}

impl Debug for Template {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.pad("<template>")
    }
}

impl PartialEq for Template {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Add for Template {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for Template {
    fn add_assign(&mut self, rhs: Template) {
        let sink = Rc::make_mut(&mut self.0);
        match Rc::try_unwrap(rhs.0) {
            Ok(source) => sink.extend(source),
            Err(rc) => sink.extend(rc.iter().cloned()),
        }
    }
}

/// Transforms from template to layout representation.
struct Builder {
    /// The current style.
    style: Style,
    /// Snapshots of the style.
    snapshots: Vec<Style>,
    /// The tree of finished page runs.
    tree: LayoutTree,
    /// When we are building the top-level layout trees, this contains metrics
    /// of the page. While building a stack, this is `None`.
    page: Option<PageBuilder>,
    /// The currently built stack of paragraphs.
    stack: StackBuilder,
}

impl Builder {
    /// Create a new builder with a base style.
    fn new(style: &Style, pages: bool) -> Self {
        Self {
            style: style.clone(),
            snapshots: vec![],
            tree: LayoutTree { runs: vec![] },
            page: pages.then(|| PageBuilder::new(style, true)),
            stack: StackBuilder::new(style),
        }
    }

    /// Build a template.
    fn template(&mut self, template: &Template) {
        for node in template.0.iter() {
            self.node(node);
        }
    }

    /// Build a template node.
    fn node(&mut self, node: &TemplateNode) {
        match node {
            TemplateNode::Save => self.snapshots.push(self.style.clone()),
            TemplateNode::Restore => {
                let style = self.snapshots.pop().unwrap();
                let newpage = style.page != self.style.page;
                self.style = style;
                if newpage {
                    self.pagebreak(true, false);
                }
            }
            TemplateNode::Space(decos) => self.space(decos),
            TemplateNode::Linebreak => self.linebreak(),
            TemplateNode::Parbreak => self.parbreak(),
            TemplateNode::Pagebreak(keep) => self.pagebreak(*keep, true),
            TemplateNode::Text(text, decos) => self.text(text, decos),
            TemplateNode::Spacing(axis, amount) => self.spacing(*axis, *amount),
            TemplateNode::Inline(f, decos) => self.inline(f(&self.style), decos),
            TemplateNode::Block(f) => self.block(f(&self.style)),
            TemplateNode::Modify(f) => f(&mut self.style),
        }
    }

    /// Push a word space into the active paragraph.
    fn space(&mut self, decos: &[Decoration]) {
        self.stack.par.push_soft(self.make_text_node(' ', decos.to_vec()));
    }

    /// Apply a forced line break.
    fn linebreak(&mut self) {
        self.stack.par.push_hard(self.make_text_node('\n', vec![]));
    }

    /// Apply a forced paragraph break.
    fn parbreak(&mut self) {
        let amount = self.style.par_spacing();
        self.stack.finish_par(&self.style);
        self.stack.push_soft(StackChild::Spacing(amount.into()));
    }

    /// Apply a forced page break.
    fn pagebreak(&mut self, keep: bool, hard: bool) {
        if let Some(builder) = &mut self.page {
            let page = mem::replace(builder, PageBuilder::new(&self.style, hard));
            let stack = mem::replace(&mut self.stack, StackBuilder::new(&self.style));
            self.tree.runs.extend(page.build(stack.build(), keep));
        }
    }

    /// Push text into the active paragraph.
    fn text(&mut self, text: impl Into<EcoString>, decos: &[Decoration]) {
        self.stack.par.push(self.make_text_node(text, decos.to_vec()));
    }

    /// Push an inline node into the active paragraph.
    fn inline(&mut self, node: impl Into<LayoutNode>, decos: &[Decoration]) {
        let align = self.style.aligns.inline;
        self.stack.par.push(ParChild::Any(node.into(), align, decos.to_vec()));
    }

    /// Push a block node into the active stack, finishing the active paragraph.
    fn block(&mut self, node: impl Into<LayoutNode>) {
        self.parbreak();
        let aligns = self.style.aligns;
        self.stack.push(StackChild::Any(node.into(), aligns));
        self.parbreak();
    }

    /// Push spacing into the active paragraph or stack depending on the `axis`.
    fn spacing(&mut self, axis: GenAxis, amount: Linear) {
        match axis {
            GenAxis::Block => {
                self.stack.finish_par(&self.style);
                self.stack.push_hard(StackChild::Spacing(amount));
            }
            GenAxis::Inline => {
                self.stack.par.push_hard(ParChild::Spacing(amount));
            }
        }
    }

    /// Finish building and return the created stack.
    fn build_stack(self) -> StackNode {
        assert!(self.page.is_none());
        self.stack.build()
    }

    /// Finish building and return the created layout tree.
    fn build_tree(mut self) -> LayoutTree {
        assert!(self.page.is_some());
        self.pagebreak(true, false);
        self.tree
    }

    /// Construct a text node with the given text and settings from the current
    /// style.
    fn make_text_node(
        &self,
        text: impl Into<EcoString>,
        decos: Vec<Decoration>,
    ) -> ParChild {
        ParChild::Text(
            text.into(),
            self.style.aligns.inline,
            Rc::clone(&self.style.text),
            decos,
        )
    }
}

struct PageBuilder {
    size: Size,
    padding: Sides<Linear>,
    hard: bool,
}

impl PageBuilder {
    fn new(style: &Style, hard: bool) -> Self {
        Self {
            size: style.page.size,
            padding: style.page.margins(),
            hard,
        }
    }

    fn build(self, child: StackNode, keep: bool) -> Option<PageRun> {
        let Self { size, padding, hard } = self;
        (!child.children.is_empty() || (keep && hard)).then(|| PageRun {
            size,
            child: PadNode { padding, child: child.into() }.into(),
        })
    }
}

struct StackBuilder {
    dirs: Gen<Dir>,
    children: Vec<StackChild>,
    last: Last<StackChild>,
    par: ParBuilder,
}

impl StackBuilder {
    fn new(style: &Style) -> Self {
        Self {
            dirs: Gen::new(style.dir, Dir::TTB),
            children: vec![],
            last: Last::None,
            par: ParBuilder::new(style),
        }
    }

    fn push(&mut self, child: StackChild) {
        self.children.extend(self.last.any());
        self.children.push(child);
    }

    fn push_soft(&mut self, child: StackChild) {
        self.last.soft(child);
    }

    fn push_hard(&mut self, child: StackChild) {
        self.last.hard();
        self.children.push(child);
    }

    fn finish_par(&mut self, style: &Style) {
        let par = mem::replace(&mut self.par, ParBuilder::new(style));
        if let Some(par) = par.build() {
            self.push(par);
        }
    }

    fn build(self) -> StackNode {
        let Self { dirs, mut children, par, mut last } = self;
        if let Some(par) = par.build() {
            children.extend(last.any());
            children.push(par);
        }
        StackNode { dirs, children }
    }
}

struct ParBuilder {
    aligns: Gen<Align>,
    dir: Dir,
    line_spacing: Length,
    children: Vec<ParChild>,
    last: Last<ParChild>,
}

impl ParBuilder {
    fn new(style: &Style) -> Self {
        Self {
            aligns: style.aligns,
            dir: style.dir,
            line_spacing: style.line_spacing(),
            children: vec![],
            last: Last::None,
        }
    }

    fn push(&mut self, child: ParChild) {
        if let Some(soft) = self.last.any() {
            self.push_inner(soft);
        }
        self.push_inner(child);
    }

    fn push_soft(&mut self, child: ParChild) {
        self.last.soft(child);
    }

    fn push_hard(&mut self, child: ParChild) {
        self.last.hard();
        self.push_inner(child);
    }

    fn push_inner(&mut self, child: ParChild) {
        if let ParChild::Text(curr_text, curr_align, curr_props, curr_decos) = &child {
            if let Some(ParChild::Text(prev_text, prev_align, prev_props, prev_decos)) =
                self.children.last_mut()
            {
                if prev_align == curr_align
                    && Rc::ptr_eq(prev_props, curr_props)
                    && curr_decos == prev_decos
                {
                    prev_text.push_str(&curr_text);
                    return;
                }
            }
        }

        self.children.push(child);
    }

    fn build(self) -> Option<StackChild> {
        let Self { aligns, dir, line_spacing, children, .. } = self;
        (!children.is_empty()).then(|| {
            let node = ParNode { dir, line_spacing, children };
            StackChild::Any(node.into(), aligns)
        })
    }
}

/// Finite state machine for spacing coalescing.
enum Last<N> {
    None,
    Any,
    Soft(N),
}

impl<N> Last<N> {
    fn any(&mut self) -> Option<N> {
        match mem::replace(self, Self::Any) {
            Self::Soft(soft) => Some(soft),
            _ => None,
        }
    }

    fn soft(&mut self, soft: N) {
        if let Self::Any = self {
            *self = Self::Soft(soft);
        }
    }

    fn hard(&mut self) {
        *self = Self::None;
    }
}
