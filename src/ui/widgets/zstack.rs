use std::ops::ControlFlow;

use crate::ui::*;

/// Z-stack.
#[derive(Default)]
pub struct ZStack<T> {
    widgets: Vec<Pod<T, Box<dyn Widget<T>>>>,
}

impl<T> ZStack<T> {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }

    pub fn push(mut self, widget: impl Widget<T> + 'static) -> Self {
        self.widgets.push(Pod::new(Box::new(widget)));
        self
    }
}

impl<T> Widget<T> for ZStack<T> {
    fn update(&mut self, ctx: &Context<'_>, data: &T) {
        for widget in &mut self.widgets {
            widget.update(ctx, data);
        }
    }

    fn layout(&mut self, parent: Size, ctx: &LayoutCtx<'_>, data: &T, env: &Env) -> Size {
        for widget in &mut self.widgets {
            widget.layout(parent, ctx, data, env);
        }
        parent
    }

    fn paint(&mut self, mut canvas: Canvas<'_>, data: &T) {
        for widget in self.widgets.iter_mut() {
            widget.paint(canvas.clone(), data);
        }
    }

    fn contains(&self, point: Point) -> bool {
        self.widgets.iter().rev().any(|w| w.contains(point))
    }

    fn event(&mut self, event: &WidgetEvent, ctx: &Context<'_>, data: &mut T) -> ControlFlow<()> {
        let mut flow = ControlFlow::Continue(());
        let mut hot = None;

        for widget in self.widgets.iter_mut().rev() {
            match event {
                WidgetEvent::MouseMove(point) => {
                    if widget.contains(*point) {
                        flow = widget.event(event, ctx, data);
                        hot = Some(widget.id);

                        break;
                    }
                }
                _ => {
                    flow = widget.event(event, ctx, data);
                }
            }

            if let ControlFlow::Break(_) = flow {
                break;
            }
        }

        if let Some(id) = hot {
            for w in self.widgets.iter_mut().filter(|w| w.id != id) {
                if w.hot {
                    w.event(&WidgetEvent::MouseExit, ctx, data);
                }
            }
        }
        flow
    }

    fn lifecycle(
        &mut self,
        lifecycle: &WidgetLifecycle<'_>,
        ctx: &Context<'_>,
        data: &T,
        env: &Env,
    ) {
        for widget in &mut self.widgets {
            widget.lifecycle(lifecycle, ctx, data, env);
        }
    }

    fn cursor(&self) -> Option<&'static str> {
        for widget in self.widgets.iter().rev() {
            if widget.hot {
                if let Some(cursor) = widget.cursor() {
                    return Some(cursor);
                }
            }
        }
        None
    }

    fn frame(&mut self, surfaces: &Surfaces, data: &mut T) {
        for widget in &mut self.widgets {
            widget.frame(surfaces, data);
        }
    }

    fn display(&self) -> String {
        format!("ZStack({})", self.widgets.len())
    }
}

pub fn zstack<W, T>(children: W) -> ZStack<T>
where
    W: WidgetTuple<T>,
{
    ZStack {
        widgets: children.to_vec().into_iter().map(Pod::new).collect(),
    }
}
