use typst::geom::Transform;

use crate::prelude::*;

/// # Move
/// Move content without affecting layout.
///
/// ## Parameters
/// - body: Content (positional, required)
///   The content to move.
///
/// - dx: Rel<Length> (named)
///   The horizontal displacement of the content.
///
/// - dy: Rel<Length> (named)
///   The vertical displacement of the content.
///
/// ## Category
/// layout
#[func]
#[capable(Layout, Inline)]
#[derive(Debug, Hash)]
pub struct MoveNode {
    /// The offset by which to move the content.
    pub delta: Axes<Rel<Length>>,
    /// The content that should be moved.
    pub body: Content,
}

#[node]
impl MoveNode {
    fn construct(_: &Vm, args: &mut Args) -> SourceResult<Content> {
        let dx = args.named("dx")?.unwrap_or_default();
        let dy = args.named("dy")?.unwrap_or_default();
        Ok(Self {
            delta: Axes::new(dx, dy),
            body: args.expect("body")?,
        }
        .pack())
    }
}

impl Layout for MoveNode {
    fn layout(
        &self,
        vt: &mut Vt,
        styles: StyleChain,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        let mut fragment = self.body.layout(vt, styles, regions)?;
        for frame in &mut fragment {
            let delta = self.delta.resolve(styles);
            let delta = delta.zip(frame.size()).map(|(d, s)| d.relative_to(s));
            frame.translate(delta.to_point());
        }
        Ok(fragment)
    }
}

impl Inline for MoveNode {}

/// # Rotate
/// Rotate content with affecting layout.
///
/// ## Parameters
/// - body: Content (positional, required)
///   The content to rotate.
///
/// - angle: Angle (named)
///   The amount of rotation.
///
/// ## Category
/// layout
#[func]
#[capable(Layout, Inline)]
#[derive(Debug, Hash)]
pub struct RotateNode {
    /// The angle by which to rotate the node.
    pub angle: Angle,
    /// The content that should be rotated.
    pub body: Content,
}

#[node]
impl RotateNode {
    /// The origin of the rotation.
    #[property(resolve)]
    pub const ORIGIN: Axes<Option<GenAlign>> = Axes::default();

    fn construct(_: &Vm, args: &mut Args) -> SourceResult<Content> {
        Ok(Self {
            angle: args.named_or_find("angle")?.unwrap_or_default(),
            body: args.expect("body")?,
        }
        .pack())
    }
}

impl Layout for RotateNode {
    fn layout(
        &self,
        vt: &mut Vt,
        styles: StyleChain,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        let mut fragment = self.body.layout(vt, styles, regions)?;
        for frame in &mut fragment {
            let origin = styles.get(Self::ORIGIN).unwrap_or(Align::CENTER_HORIZON);
            let Axes { x, y } = origin.zip(frame.size()).map(|(o, s)| o.position(s));
            let transform = Transform::translate(x, y)
                .pre_concat(Transform::rotate(self.angle))
                .pre_concat(Transform::translate(-x, -y));
            frame.transform(transform);
        }
        Ok(fragment)
    }
}

impl Inline for RotateNode {}

/// # Scale
/// Scale content without affecting layout.
///
/// ## Parameters
/// - body: Content (positional, required)
///   The content to scale.
///
/// - x: Ratio (named)
///   The horizontal scaling factor.
///
/// - y: Ratio (named)
///   The vertical scaling factor.
///
/// ## Category
/// layout
#[func]
#[capable(Layout, Inline)]
#[derive(Debug, Hash)]
pub struct ScaleNode {
    /// Scaling factor.
    pub factor: Axes<Ratio>,
    /// The content that should be scaled.
    pub body: Content,
}

#[node]
impl ScaleNode {
    /// The origin of the transformation.
    #[property(resolve)]
    pub const ORIGIN: Axes<Option<GenAlign>> = Axes::default();

    fn construct(_: &Vm, args: &mut Args) -> SourceResult<Content> {
        let all = args.find()?;
        let x = args.named("x")?.or(all).unwrap_or(Ratio::one());
        let y = args.named("y")?.or(all).unwrap_or(Ratio::one());
        Ok(Self {
            factor: Axes::new(x, y),
            body: args.expect("body")?,
        }
        .pack())
    }
}

impl Layout for ScaleNode {
    fn layout(
        &self,
        vt: &mut Vt,
        styles: StyleChain,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        let mut fragment = self.body.layout(vt, styles, regions)?;
        for frame in &mut fragment {
            let origin = styles.get(Self::ORIGIN).unwrap_or(Align::CENTER_HORIZON);
            let Axes { x, y } = origin.zip(frame.size()).map(|(o, s)| o.position(s));
            let transform = Transform::translate(x, y)
                .pre_concat(Transform::scale(self.factor.x, self.factor.y))
                .pre_concat(Transform::translate(-x, -y));
            frame.transform(transform);
        }
        Ok(fragment)
    }
}

impl Inline for ScaleNode {}
