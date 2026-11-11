use gpui::{Bounds, Pixels, Point};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OverlayPlacement {
    AboveSource,
    BelowSource,
}

pub(super) fn overlay_anchor(
    caret: Option<Bounds<Pixels>>,
    block: Option<Bounds<Pixels>>,
    input: Option<Bounds<Pixels>>,
    placement: OverlayPlacement,
) -> Option<Point<Pixels>> {
    match placement {
        OverlayPlacement::BelowSource => caret
            .map(|bounds| Point::new(bounds.left(), bounds.bottom()))
            .or_else(|| block.map(|bounds| Point::new(bounds.left(), bounds.bottom())))
            .or_else(|| input.map(|bounds| Point::new(bounds.left(), bounds.top()))),
        OverlayPlacement::AboveSource => block
            .map(|bounds| Point::new(bounds.left(), bounds.top()))
            .or_else(|| caret.map(|bounds| Point::new(bounds.left(), bounds.top())))
            .or_else(|| input.map(|bounds| Point::new(bounds.left(), bounds.top()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px, size};

    fn bounds(x: f32, y: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(20.), px(10.)))
    }

    #[test]
    fn anchor_prefers_current_geometry_and_requested_side() {
        let caret = bounds(10., 20.);
        let block = bounds(30., 40.);
        let input = bounds(50., 60.);
        assert_eq!(
            overlay_anchor(
                Some(caret),
                Some(block),
                Some(input),
                OverlayPlacement::BelowSource,
            ),
            Some(point(px(10.), px(30.)))
        );
        assert_eq!(
            overlay_anchor(
                None,
                Some(block),
                Some(input),
                OverlayPlacement::AboveSource
            ),
            Some(point(px(30.), px(40.)))
        );
        assert_eq!(
            overlay_anchor(None, None, Some(input), OverlayPlacement::BelowSource),
            Some(point(px(50.), px(60.)))
        );
    }
}
