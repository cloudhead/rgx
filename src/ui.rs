pub mod canvas;
pub mod context;
pub mod env;
#[cfg(test)]
pub mod tests;
pub mod text;
pub mod widgets;

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{ControlFlow, Deref, DerefMut};
use std::sync::atomic;

use crate::gfx::*;
use crate::gfx::{Paint, TextureId};
use crate::math::*;

pub use canvas::*;
pub use context::*;
pub use env::Env;
pub use widgets::align::Align;
pub use widgets::align::{align, bottom, center, left, right, top};
pub use widgets::click::Click;
pub use widgets::controller::Control;
pub use widgets::hover::Hover;
pub use widgets::hstack::hstack;
pub use widgets::painter::painter;
pub use widgets::zstack::zstack;
pub use widgets::Pod;
pub use widgets::{Widget, WidgetEvent, WidgetExt, WidgetId, WidgetTuple};

/// Off-screen render surfaces.
pub type Surfaces = HashMap<TextureId, Image>;

/// A widget lifecycle event.
#[derive(Debug, Copy, Clone)]
pub enum WidgetLifecycle<'a> {
    Initialized(&'a HashMap<TextureId, TextureInfo>),
}

/// A widget with a custom cursor.
pub struct Interactive<T> {
    widget: Box<dyn Widget<T>>,
    cursor: Option<&'static str>,
}

impl<T> Widget<T> for Interactive<T> {
    fn layout(&mut self, parent: Size, ctx: &LayoutCtx<'_>, data: &T, env: &Env) -> Size {
        self.widget.layout(parent, ctx, data, env)
    }

    fn paint(&mut self, canvas: Canvas<'_>, data: &T) {
        self.widget.paint(canvas, data);
    }

    fn update(&mut self, ctx: &Context<'_>, data: &T) {
        self.widget.update(ctx, data);
    }

    fn event(&mut self, event: &WidgetEvent, ctx: &Context<'_>, data: &mut T) -> ControlFlow<()> {
        self.widget.event(event, ctx, data)
    }

    fn contains(&self, point: Point) -> bool {
        self.widget.contains(point)
    }

    fn cursor(&self) -> Option<&'static str> {
        self.cursor
    }

    fn display(&self) -> String {
        format!("Interactive({})", self.widget.display())
    }
}

/// Widgets that can be interacted with.
pub trait Interact<T>: Sized + Widget<T> + 'static {
    fn on_click(self, action: impl Fn(&Context<'_>, &mut T) + 'static) -> Control<Self, Click<T>> {
        Control::new(self, Click::new(action))
    }

    fn on_hover(
        self,
        action: impl Fn(bool, &Context<'_>, &mut T) + 'static,
    ) -> Control<Self, Hover<T>> {
        Control::new(self, Hover::new(action))
    }

    fn set_cursor(self, cursor: Option<&'static str>) -> Interactive<T> {
        Interactive {
            widget: Box::new(self),
            cursor,
        }
    }
}

impl<T, W> Interact<T> for W where W: Widget<T> + 'static {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Id(u64);

impl Id {
    pub fn next() -> Self {
        static NEXT: atomic::AtomicU64 = atomic::AtomicU64::new(1);

        Self(NEXT.fetch_add(1, atomic::Ordering::SeqCst))
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Padding of an element inside a container.
#[derive(Default, Debug, Clone)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    pub fn all(padding: f32) -> Self {
        Self {
            top: padding,
            bottom: padding,
            right: padding,
            left: padding,
        }
    }

    pub fn bottom(mut self, bottom: f32) -> Self {
        self.bottom = bottom;
        self
    }
}

impl From<[f32; 4]> for Padding {
    fn from([top, right, bottom, left]: [f32; 4]) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl From<[f32; 2]> for Padding {
    fn from([vertical, horizontal]: [f32; 2]) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

/// Positions elements inside a container.
#[derive(Default, Debug, Clone)]
pub struct Position {
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
}

impl Position {
    pub fn top(mut self, top: f32) -> Self {
        self.top = Some(top);
        self
    }

    pub fn right(mut self, right: f32) -> Self {
        self.right = Some(right);
        self
    }

    pub fn bottom(mut self, bottom: f32) -> Self {
        self.bottom = Some(bottom);
        self
    }

    pub fn left(mut self, left: f32) -> Self {
        self.left = Some(left);
        self
    }
}
