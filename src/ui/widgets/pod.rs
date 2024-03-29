use std::ops::{ControlFlow, Deref};
use std::{env, fmt};

use crate::ui::canvas::*;
use crate::ui::*;

/// Widget pod. Wraps all widgets, making them smart.
pub struct Pod<T, W> {
    pub id: WidgetId,
    pub size: Size,
    pub offset: Offset,
    pub hot: bool,
    pub active: bool,

    widget: W,
    data: PhantomData<T>,
}

impl<T, W: Widget<T>> Pod<T, W> {
    pub fn new(widget: W) -> Self {
        Self {
            id: WidgetId::next(),
            size: Size::ZERO,
            offset: Offset::ZERO,
            hot: false,
            active: false,
            widget,
            data: PhantomData,
        }
    }

    fn context<'a>(&self, parent: &'a Context<'_>) -> Context<'a> {
        parent.offset(self.offset).hot(self.hot).active(self.active)
    }

    fn bounds(&self) -> Rect<f32> {
        Rect::origin(self.size)
    }

    fn transform(&self) -> Transform {
        Transform::translate(self.offset)
    }
}

impl<T, W: Widget<T>> fmt::Display for Pod<T, W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.display(), self.id)
    }
}

impl<T, W: Widget<T>> Widget<T> for Pod<T, W> {
    fn layout(&mut self, parent: Size, ctx: &LayoutCtx<'_>, data: &T, env: &Env) -> Size {
        self.size = self.widget.layout(parent, ctx, data, env);
        self.size
    }

    fn paint(&mut self, mut canvas: Canvas<'_>, data: &T) {
        self.widget.paint(canvas.transform(self.transform()), data);

        if env::var("DEBUG").is_ok() {
            canvas.paint(
                Rectangle::new(self.offset, self.size)
                    .stroke(1., Rgba8::GREEN.alpha(0x44))
                    .fill(Rgba8::GREEN.alpha(if self.hot { 0x22 } else { 0x11 })),
            );
        }
    }

    fn update(&mut self, ctx: &Context<'_>, data: &T) {
        self.widget.update(&self.context(ctx), data)
    }

    fn cursor(&self) -> Option<&'static str> {
        self.widget.cursor()
    }

    fn event(&mut self, event: &WidgetEvent, ctx: &Context<'_>, data: &mut T) -> ControlFlow<()> {
        let ctx = self.context(ctx);

        match event {
            WidgetEvent::MouseEnter => {
                let contains =
                    self.bounds().contains(ctx.cursor) && self.widget.contains(ctx.cursor);

                if contains {
                    self.hot = true;
                    self.widget.event(&WidgetEvent::MouseEnter, &ctx, data)
                } else {
                    ControlFlow::Continue(())
                }
            }
            WidgetEvent::MouseExit => {
                if self.hot {
                    self.hot = false;
                    self.widget.event(&WidgetEvent::MouseExit, &ctx, data)
                } else {
                    ControlFlow::Continue(())
                }
            }
            WidgetEvent::MouseMove(point) => {
                let cursor = point.untransform(self.transform());
                let contains = self.bounds().contains(cursor) && self.widget.contains(cursor);

                if contains {
                    // If the widget wasn't hot before, we send a `MouseEnter`.
                    if self.hot {
                        self.widget
                            .event(&WidgetEvent::MouseMove(cursor), &ctx, data)
                    } else {
                        self.hot = true;
                        self.widget.event(&WidgetEvent::MouseEnter, &ctx, data)
                    }
                } else if self.hot {
                    self.hot = false;
                    self.widget.event(&WidgetEvent::MouseExit, &ctx, data)
                } else {
                    ControlFlow::Continue(())
                }
            }
            WidgetEvent::MouseDown(_) => {
                // Only propagate event if hot.
                if self.hot {
                    self.active = true;
                    self.widget.event(event, &ctx, data)
                } else {
                    ControlFlow::Continue(())
                }
            }
            WidgetEvent::MouseUp(_) => {
                // Only propagate event if active.
                if self.active {
                    // It may look wrong that we're setting active to `false`
                    // here while telling our widget that we're active, but
                    // it's not!
                    self.active = false;
                    self.widget.event(event, &ctx, data)
                } else {
                    ControlFlow::Continue(())
                }
            }
            _ => self.widget.event(event, &ctx, data),
        }
    }

    fn lifecycle(
        &mut self,
        lifecycle: &WidgetLifecycle<'_>,
        ctx: &Context<'_>,
        data: &T,
        env: &Env,
    ) {
        self.widget.lifecycle(lifecycle, ctx, data, env)
    }

    fn frame(&mut self, surfaces: &Surfaces, data: &mut T) {
        self.widget.frame(surfaces, data);
    }

    fn contains(&self, point: Point) -> bool {
        self.widget.contains(point - self.offset)
    }

    fn type_name(&self) -> &'static str {
        self.widget.type_name()
    }

    fn display(&self) -> String {
        self.widget.display()
    }
}

impl<T, W> Deref for Pod<T, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}
