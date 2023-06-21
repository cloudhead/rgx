use std::time;

use crate::gfx::*;
use crate::platform;
use crate::ui::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct WidgetId(Id);

impl WidgetId {
    pub fn root() -> Self {
        WidgetId(Id(0))
    }

    pub fn next() -> Self {
        Self(Id::next())
    }
}

impl fmt::Display for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Widget event.
#[derive(Debug, Clone)]
pub enum WidgetEvent {
    MouseDown(platform::MouseButton),
    MouseUp(platform::MouseButton),
    MouseScroll(platform::LogicalDelta),
    MouseMove(Point),
    Resized(Size),
    MouseEnter,
    MouseExit,
    Focus(bool),
    KeyDown {
        key: platform::Key,
        modifiers: platform::ModifiersState,
        repeat: bool,
    },
    KeyUp {
        key: platform::Key,
        modifiers: platform::ModifiersState,
    },
    CharacterReceived(char, platform::ModifiersState),
    Paste(Option<String>),
    Tick(time::Duration),
    Frame,
}

/// A UI widget that can be painted on screen.
#[allow(unused_variables)]
pub trait Widget<T> {
    /// Compute the widget layout.
    fn layout(&mut self, parent: Size, ctx: &LayoutCtx<'_>, data: &T, env: &Env) -> Size {
        parent
    }

    /// Paint the widget.
    fn paint(&mut self, canvas: Canvas<'_>, data: &T);

    /// Update the widget's appearance.
    fn update(&mut self, ctx: &Context<'_>, data: &T) {}

    /// Process an external event.
    fn event(&mut self, event: &WidgetEvent, ctx: &Context<'_>, data: &mut T) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    /// Process a lifecycle event.
    fn lifecycle(
        &mut self,
        lifecycle: &WidgetLifecycle<'_>,
        ctx: &Context<'_>,
        data: &T,
        env: &Env,
    ) {
    }

    /// Handle the end of the frame.
    fn frame(&mut self, surfaces: &Surfaces, data: &mut T) {}

    /// Get the cursor to display over this widget.
    fn cursor(&self) -> Option<&'static str> {
        None
    }

    /// Check whether this widget contains a point.
    fn contains(&self, point: Point) -> bool {
        // The `Pod` around the widget will do a preliminary bounds check, so unless the widget
        // has "holes", this should always return `true`.
        true
    }

    /// Display this widget in text.
    fn display(&self) -> String {
        self.type_name().to_owned()
    }

    #[doc(hidden)]
    /// Get the identity of the widget; this is basically only implemented by
    /// `IdentityWrapper`. Widgets should not implement this on their own.
    fn id(&self) -> Option<WidgetId> {
        None
    }

    #[doc(hidden)]
    /// Get the (verbose) type name of the widget for debugging purposes.
    /// You should not override this method.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T> Widget<T> for Box<dyn Widget<T>> {
    fn layout(&mut self, parent: Size, ctx: &LayoutCtx<'_>, data: &T, env: &Env) -> Size {
        self.deref_mut().layout(parent, ctx, data, env)
    }

    fn update(&mut self, ctx: &Context<'_>, data: &T) {
        self.deref_mut().update(ctx, data)
    }

    fn paint(&mut self, canvas: Canvas<'_>, data: &T) {
        self.deref_mut().paint(canvas, data)
    }

    fn event(&mut self, event: &WidgetEvent, ctx: &Context<'_>, data: &mut T) -> ControlFlow<()> {
        self.deref_mut().event(event, ctx, data)
    }

    fn lifecycle(
        &mut self,
        lifecycle: &WidgetLifecycle<'_>,
        ctx: &Context<'_>,
        data: &T,
        env: &Env,
    ) {
        self.deref_mut().lifecycle(lifecycle, ctx, data, env)
    }

    fn frame(&mut self, surfaces: &Surfaces, data: &mut T) {
        self.deref_mut().frame(surfaces, data)
    }

    fn cursor(&self) -> Option<&'static str> {
        self.deref().cursor()
    }

    fn contains(&self, point: Point) -> bool {
        self.deref().contains(point)
    }

    fn display(&self) -> String {
        self.deref().display()
    }
}

impl<T> Widget<T> for Rgba8 {
    fn paint(&mut self, mut canvas: Canvas<'_>, _data: &T) {
        canvas.fill(canvas.bounds(), *self);
    }

    fn display(&self) -> String {
        self.to_string()
    }
}

/// A widget tuple that can be converted to a widget vector.
pub trait WidgetTuple<T> {
    /// Convert tuple to vector.
    fn to_vec(self) -> Vec<Box<dyn Widget<T> + 'static>>;
}

impl<T, W1, W2> WidgetTuple<T> for (W1, W2)
where
    W1: Widget<T> + 'static,
    W2: Widget<T> + 'static,
{
    fn to_vec(self) -> Vec<Box<dyn Widget<T> + 'static>> {
        vec![Box::new(self.0), Box::new(self.1)]
    }
}

impl<T, W1, W2, W3> WidgetTuple<T> for (W1, W2, W3)
where
    W1: Widget<T> + 'static,
    W2: Widget<T> + 'static,
    W3: Widget<T> + 'static,
{
    fn to_vec(self) -> Vec<Box<dyn Widget<T> + 'static>> {
        vec![Box::new(self.0), Box::new(self.1), Box::new(self.2)]
    }
}

impl<T, W> WidgetTuple<T> for Vec<W>
where
    W: Widget<T> + 'static,
{
    fn to_vec(self) -> Vec<Box<dyn Widget<T> + 'static>> {
        self.into_iter()
            .map(|e| Box::new(e) as Box<dyn Widget<T>>)
            .collect()
    }
}

/// Widget extension methods.
pub trait WidgetExt<T>: Sized + Widget<T> + 'static {
    /// Box a widget.
    fn boxed(self) -> Box<dyn Widget<T> + 'static>;
    /// Size a widget.
    fn sized<S: Into<Size>>(self, size: S) -> widgets::SizedBox<T>;
}

impl<T, W: 'static> WidgetExt<T> for W
where
    W: Widget<T>,
{
    fn boxed(self) -> Box<dyn Widget<T> + 'static> {
        Box::new(self)
    }

    fn sized<S: Into<Size>>(self, size: S) -> widgets::SizedBox<T> {
        let size = size.into();
        widgets::SizedBox::new(self).width(size.w).height(size.h)
    }
}
