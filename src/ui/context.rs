use super::*;

/// Widget layout context.
#[derive(Debug, Copy, Clone)]
pub struct LayoutCtx<'a> {
    pub fonts: &'a HashMap<text::FontId, text::Font>,
}

impl<'a> LayoutCtx<'a> {
    pub fn new(fonts: &'a HashMap<text::FontId, text::Font>) -> Self {
        Self { fonts }
    }
}

/// Widget general context.
#[derive(Debug, Copy, Clone)]
pub struct Context<'a> {
    /// Widget transform.
    pub transform: Transform,
    /// Cursor position relative to widget.
    pub cursor: Point,
    /// Loaded textures.
    pub surfaces: &'a HashMap<TextureId, Image>,
    /// Whether this widget is hot.
    pub hot: bool,
    /// Whether this widget is active.
    pub active: bool,
}

impl<'a> Context<'a> {
    pub fn new(cursor: Point, surfaces: &'a HashMap<TextureId, Image>) -> Self {
        Self {
            transform: Transform::identity(),
            cursor,
            surfaces,
            hot: false,
            active: false,
        }
    }

    /// Offset this context.
    pub fn offset(self, offset: Offset) -> Self {
        self.transform(Transform::translate(offset))
    }

    /// Set widget "hot" state.
    pub fn hot(self, hot: bool) -> Self {
        Self { hot, ..self }
    }

    /// Set widget "active" state.
    pub fn active(self, active: bool) -> Self {
        Self { active, ..self }
    }

    /// Transform context.
    pub fn transform(self, t: impl Into<Transform>) -> Self {
        let t = t.into();
        let transform = self.transform * t;

        Self {
            transform,
            cursor: self.cursor.untransform(t),
            ..self
        }
    }

    /// Check hot state.
    pub fn is_hot(&self) -> bool {
        self.hot
    }
}
