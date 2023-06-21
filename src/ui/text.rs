pub mod font;

use crate::gfx::*;
use crate::math::*;

use super::{Canvas, Env, IntoPaint, LayoutCtx, Paint, Widget};
pub use font::{Error as FontError, Font, FontFormat, FontId};

pub struct Text {
    pub body: String,
    pub font: FontId,
    pub color: Rgba8,
    pub transform: Transform,
    pub align: TextAlign,
    pub size: Size,
}

impl Text {
    pub fn new(body: impl ToString) -> Self {
        Self {
            body: body.to_string(),
            font: FontId::default(),
            color: Rgba8::WHITE,
            transform: Transform::identity(),
            align: TextAlign::Left,
            size: Size::ZERO,
        }
    }

    pub fn color(self, color: Rgba8) -> Self {
        Self { color, ..self }
    }

    pub fn font(self, font: FontId) -> Self {
        Self { font, ..self }
    }

    pub fn transform(self, transform: Transform) -> Self {
        Self { transform, ..self }
    }

    pub fn offset(self, offset: impl Into<Offset>) -> Self {
        Self {
            transform: self.transform * Transform::translate(offset.into()),
            ..self
        }
    }

    pub fn align(self, align: TextAlign) -> Self {
        Self { align, ..self }
    }
}

impl IntoPaint for &Text {
    fn into_paint(self, canvas: &Canvas<'_>) -> Paint {
        let Some(font) = canvas.fonts.get(&self.font) else {
            panic!("Font {:?} was not found", self.font);
        };
        let texture = canvas.textures().get(&font.texture_id).unwrap();
        let vertices = Batch::new(*font, texture.size)
            .add(
                &self.body.to_string(),
                0.,
                0.,
                ZDepth::default(),
                self.color,
                self.align,
            )
            .vertices();

        Paint::Sprite {
            transform: self.transform,
            texture: font.texture_id,
            vertices,
            target: canvas.target,
        }
    }
}

impl IntoPaint for Text {
    fn into_paint(self, canvas: &Canvas<'_>) -> Paint {
        (&self).into_paint(canvas)
    }
}

impl<T> Widget<T> for Text {
    fn layout(&mut self, _parent: Size, ctx: &LayoutCtx<'_>, _data: &T, _env: &Env) -> Size {
        if let Some(font) = ctx.fonts.get(&self.font) {
            self.size = Size::new(font.text_width(&self.body), font.text_height());
        }
        self.size
    }

    fn paint(&mut self, mut canvas: Canvas<'_>, _data: &T) {
        canvas.paint(&*self)
    }

    fn contains(&self, _point: Point) -> bool {
        true
    }

    fn display(&self) -> String {
        format!("Text({:?})", self.body)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Text batch.
pub struct Batch {
    raw: sprite2d::Batch,
    font: Font,
}

impl Batch {
    pub fn new(font: Font, size: Size<u32>) -> Self {
        let raw = sprite2d::Batch::new(size);

        Self { raw, font }
    }

    pub fn add(
        mut self,
        text: &str,
        mut sx: f32,
        sy: f32,
        z: ZDepth,
        color: Rgba8,
        align: TextAlign, // TODO: Shouldn't be a property of text, should be the container!
    ) -> Self {
        let size = Size::new(16., 16.);
        let rgba = color.into();

        match align {
            TextAlign::Left => {}
            TextAlign::Right => {
                sx -= self.font.text_width(text);
            }
            TextAlign::Center => {
                sx -= self.font.text_width(text) / 2.;
            }
        }

        for c in text.bytes() {
            let w = self.font.glyph_width(c);
            let i = c as usize;
            let x = (i % 16) as f32 * self.font.tile.w;
            let y = (i / 16) as f32 * self.font.tile.h;

            self.raw.add(
                Rect::new(Point2D::new(x, y), size),
                Rect::new(Point2D::new(sx, sy), size),
                z,
                rgba,
                1.0,
                Repeat::default(),
            );
            sx += w;
        }
        self
    }

    pub fn offset(&mut self, x: f32, y: f32) {
        self.raw.offset(x, y);
    }

    pub fn glyph(&mut self, glyph: usize, sx: f32, sy: f32, z: ZDepth, color: Rgba8) {
        let rgba = color.into();

        let gw = 16.;
        let gh = 16.;
        let size = Size::new(16., 16.);

        let i: usize = glyph;
        let x: f32 = (i % 16) as f32 * gw;
        let y: f32 = (i / 16) as f32 * gh;

        self.raw.add(
            Rect::new(Point2D::new(x, y), size),
            Rect::new(Point2D::new(sx, sy), size),
            z,
            rgba,
            1.0,
            Repeat::default(),
        );
    }

    pub fn vertices(&self) -> Vec<sprite2d::Vertex> {
        self.raw.vertices()
    }

    pub fn clear(&mut self) {
        self.raw.clear()
    }
}
