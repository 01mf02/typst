//! The compiler for the _Typst_ typesetting language.
//!
//! # Steps
//! - **Parsing:** The parsing step first transforms a plain string into an
//!   [iterator of tokens][tokens]. This token stream is [parsed] into a [syntax
//!   tree]. The structures describing the tree can be found in the [syntax]
//!   module.
//! - **Evaluation:** The next step is to [evaluate] the syntax tree. This
//!   computes the value of each node in the document and produces a [module].
//! - **Execution:** Now, we can [execute] the parsed and evaluated module. This
//!   results in a [layout tree], a high-level, fully styled representation of
//!   the document. The nodes of this tree are self-contained and
//!   order-independent and thus much better suited for layouting than the
//!   syntax tree.
//! - **Layouting:** Next, the tree is [layouted] into a portable version of the
//!   typeset document. The output of this is a collection of [`Frame`]s (one
//!   per page), ready for exporting.
//! - **Exporting:** The finished layout can be exported into a supported
//!   format. Currently, the only supported output format is [PDF].
//!
//! [tokens]: parse::Tokens
//! [parsed]: parse::parse
//! [syntax tree]: syntax::SyntaxTree
//! [evaluate]: eval::eval
//! [module]: eval::Module
//! [execute]: exec::exec
//! [layout tree]: layout::LayoutTree
//! [layouted]: layout::layout
//! [PDF]: export::pdf

#[macro_use]
pub mod diag;
#[macro_use]
pub mod eval;
pub mod color;
pub mod exec;
pub mod export;
pub mod font;
pub mod geom;
pub mod image;
pub mod layout;
pub mod library;
pub mod loading;
pub mod paper;
pub mod parse;
pub mod source;
pub mod syntax;
pub mod util;

use std::rc::Rc;

use crate::diag::TypResult;
use crate::eval::Scope;
use crate::exec::Env;
use crate::font::FontStore;
use crate::image::ImageStore;
use crate::layout::Frame;
#[cfg(feature = "layout-cache")]
use crate::layout::LayoutCache;
use crate::loading::Loader;
use crate::source::{SourceId, SourceStore};

/// The core context which holds the loader, configuration and cached artifacts.
pub struct Context {
    /// The loader the context was created with.
    pub loader: Rc<dyn Loader>,
    /// Stores loaded source files.
    pub sources: SourceStore,
    /// Stores parsed font faces.
    pub fonts: FontStore,
    /// Stores decoded images.
    pub images: ImageStore,
    /// Caches layouting artifacts.
    #[cfg(feature = "layout-cache")]
    pub layouts: LayoutCache,
    /// The standard library scope.
    std: Scope,
    /// The default environment.
    env: Env,
}

impl Context {
    /// Create a new context with the default settings.
    pub fn new(loader: Rc<dyn Loader>) -> Self {
        Self::builder().build(loader)
    }

    /// Create a new context with advanced settings.
    pub fn builder() -> ContextBuilder {
        ContextBuilder::default()
    }

    /// Typeset a source file into a collection of layouted frames.
    ///
    /// Returns either a vector of frames representing individual pages or
    /// diagnostics in the form of a vector of error message with file and span
    /// information.
    pub fn typeset(&mut self, id: SourceId) -> TypResult<Vec<Rc<Frame>>> {
        let source = self.sources.get(id);
        let ast = parse::parse(source)?;
        let module = eval::eval(self, id, Rc::new(ast))?;
        let tree = exec::exec(self, &module.template);
        let frames = layout::layout(self, &tree);
        Ok(frames)
    }

    /// Garbage-collect caches.
    pub fn turnaround(&mut self) {
        #[cfg(feature = "layout-cache")]
        self.layouts.turnaround();
    }
}

/// A builder for a [`Context`].
///
/// This struct is created by [`Context::builder`].
#[derive(Default)]
pub struct ContextBuilder {
    std: Option<Scope>,
    env: Option<Env>,
}

impl ContextBuilder {
    /// The scope containing definitions that are available everywhere
    /// (the standard library).
    pub fn std(mut self, std: Scope) -> Self {
        self.std = Some(std);
        self
    }

    /// The initial properties for page size, font selection and so on.
    pub fn env(mut self, env: Env) -> Self {
        self.env = Some(env);
        self
    }

    /// Finish building the context by providing the `loader` used to load
    /// fonts, images, source files and other resources.
    pub fn build(self, loader: Rc<dyn Loader>) -> Context {
        Context {
            sources: SourceStore::new(Rc::clone(&loader)),
            fonts: FontStore::new(Rc::clone(&loader)),
            images: ImageStore::new(Rc::clone(&loader)),
            loader,
            #[cfg(feature = "layout-cache")]
            layouts: LayoutCache::new(),
            std: self.std.unwrap_or(library::new()),
            env: self.env.unwrap_or_default(),
        }
    }
}
