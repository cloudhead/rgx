use std::collections::HashMap;
use std::{io, time};

use thiserror::Error;

use crate::clock::Clock;
use crate::gfx;
use crate::gfx::{Image, Renderer, TextureId};
use crate::math::*;
use crate::platform;
use crate::platform::{Cursor, WindowEvent, WindowHint};
use crate::timer::FrameTimer;
use crate::ui::text::{FontError, FontFormat, FontId};
use crate::ui::*;

/// Default UI scale.
pub const DEFAULT_SCALE: f32 = 2.;
/// Default target frames per second.
pub const DEFAULT_TARGET_FPS: f64 = 60.;

#[derive(Error, Debug)]
pub enum Error {
    #[error("font: {0}")]
    Font(#[from] FontError),
}

#[derive(Default, Clone, Debug)]
pub struct ImageOpts {
    /// Cursor origin.
    origin: Option<Point2D<u32>>,
}

impl ImageOpts {
    pub fn origin(mut self, origin: impl Into<Point2D<u32>>) -> Self {
        self.origin = Some(origin.into());
        self
    }
}

/// Application launcher.
pub struct Application {
    title: String,
    graphics: Graphics,
    /// Target frames per second.
    fps: f64,
    env: Env,
    cursors: Vec<(&'static str, Image, Point2D<u32>)>,
}

impl Application {
    pub fn new(title: &str) -> Self {
        let graphics = Graphics::default();
        let env = Env::default();

        Self {
            title: title.to_owned(),
            fps: DEFAULT_TARGET_FPS,
            graphics,
            env,
            cursors: Vec::new(),
        }
    }

    pub fn fonts(
        mut self,
        fonts: impl IntoIterator<Item = (impl Into<FontId>, impl AsRef<[u8]>, FontFormat)>,
    ) -> Result<Self, Error> {
        for (id, data, format) in fonts {
            let id = id.into();
            debug!("loading font {id:?}..");

            self.graphics.font(id, data.as_ref(), format)?;
        }
        Ok(self)
    }

    pub fn fps(mut self, target: f64) -> Self {
        self.fps = target;
        self
    }

    pub fn cursor(
        mut self,
        name: &'static str,
        image: Image,
        origin: impl Into<Point2D<u32>>,
    ) -> Self {
        self.cursors.push((name, image, origin.into()));
        self
    }

    pub fn image(mut self, name: &'static str, image: Image) -> Self {
        let id = TextureId::next();

        self.graphics.texture(id, image);
        self.env.set(env::Key::<TextureId>::new(name), id);
        self
    }

    /// Launch the UI by passing in the root widget and initial data.
    pub fn launch<T>(mut self, widget: impl Widget<T> + 'static, mut data: T) -> io::Result<()> {
        let hints = &[WindowHint::Resizable(true), WindowHint::Visible(true)];
        let (mut win, mut win_events) =
            platform::init(&self.title, 640, 480, hints, platform::GraphicsContext::Gl)?;

        if win.scale_factor() != 1. {
            warn!(
                "non-standard pixel scaling factor detected: {}",
                win.scale_factor()
            );
        }

        let win_scale = 1.;
        let win_size = win.size();
        let ui_scale = DEFAULT_SCALE;

        info!("window size: {}x{}", win_size.width, win_size.height);
        info!("window scale: {win_scale}");
        info!("ui scale: {ui_scale}");
        info!(
            "ui size: {}x{}",
            win_size.width as f32 / ui_scale,
            win_size.height as f32 / ui_scale
        );

        let mut renderer: gfx::backends::gl::Renderer =
            Renderer::new(&mut win, win_size, win_scale, ui_scale)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut root: Pod<T, Box<dyn Widget<T>>> = Pod::new(Box::new(widget));
        let mut store = HashMap::new();
        let mut render_timer = FrameTimer::new();
        let mut update_timer = FrameTimer::new();
        let mut paint_timer = FrameTimer::new();
        let mut events = Vec::with_capacity(16);
        let mut clock = Clock::new(time::Instant::now());

        // Window state.
        let mut resized = false;
        let mut minimized = false;

        root.lifecycle(
            &WidgetLifecycle::Initialized(&self.graphics.textures),
            &Context::new(Point::ORIGIN, &store),
            &data,
            &self.env,
        );
        // Initial update and layout so that the first events, eg. `CursorMove` work.
        // If we don't do this, widget sizes will be zero when the first events land.
        // It's important however that in the general case, update and layout are run
        // *after* events are processed.
        root.update(&Context::new(Point::ORIGIN, &store), &data);
        root.layout(
            Size::from(win.size()) / ui_scale,
            &LayoutCtx::new(&self.graphics.fonts),
            &data,
            &self.env,
        );

        for (name, image, origin) in self.cursors {
            if !image.rect().contains(origin) {
                warn!("cursor '{name}' has out-of-bounds origin");
            }
            let scaled = image.scaled(ui_scale as u32);
            let cursor = Cursor::create(&scaled, origin * ui_scale as u32);

            self.graphics.cursors.insert(name, cursor);
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // Game loop
        ////////////////////////////////////////////////////////////////////////////////////////

        while win.is_open() {
            let delta = clock.tick(self.fps);
            win_events.poll();

            let cursor = Point2D::<f64>::from(win.get_cursor_pos()) / ui_scale as f64;
            let cursor = cursor.map(|n| n.floor());
            let win_size_logical = win.size();
            let win_size_ui = Size::from(win_size_logical) / ui_scale;
            let ctx = Context::new(Point::from(cursor), &store);

            for event in win_events.flush() {
                if event.is_input() {
                    trace!("event: {event:?}");
                }

                match event {
                    WindowEvent::Resized(size) => {
                        if size.is_zero() {
                            // On certain operating systems, the window size will be set to
                            // zero when the window is minimized. Since a zero-sized framebuffer
                            // is not valid, we don't render anything in this case.
                            minimized = true;
                        } else {
                            minimized = false;
                            resized = true;
                        }
                    }
                    WindowEvent::CursorEntered { .. } => {
                        // Do nothing, we generate these based on `CursorMoved`.
                    }
                    WindowEvent::CursorLeft { .. } => {
                        // Do nothing, we generate these based on `CursorMoved`.
                    }
                    WindowEvent::Minimized => {
                        minimized = true;
                    }
                    WindowEvent::Restored => {
                        minimized = false;
                    }
                    WindowEvent::Focused(state) => {
                        events.push(WidgetEvent::Focus(state));
                    }
                    WindowEvent::RedrawRequested => {
                        // All events currently trigger a redraw, we don't need to
                        // do anything special here.
                    }
                    WindowEvent::ScaleFactorChanged(factor) => {
                        renderer.handle_scale_factor_changed(factor);
                    }
                    WindowEvent::CloseRequested => {
                        // Ignore.
                    }
                    WindowEvent::CursorMoved { .. } => {
                        // Nb. The position given in the event can be delayed by a frame sometimes.
                        // Therefore, we use the position gotten at the start of the render loop.
                        events.push(WidgetEvent::MouseMove(Point::from(cursor)));
                    }
                    WindowEvent::MouseInput { state, button, .. } => match state {
                        platform::InputState::Pressed => {
                            events.push(WidgetEvent::MouseDown(button));
                        }
                        platform::InputState::Released => {
                            events.push(WidgetEvent::MouseUp(button));
                        }
                        _ => {}
                    },
                    WindowEvent::Scroll { delta, .. } => {
                        events.push(WidgetEvent::MouseScroll(delta));
                    }
                    WindowEvent::KeyboardInput(input) => {
                        // Intercept `<insert>` key for pasting.
                        //
                        // Reading from the clipboard causes the loop to wake up for some strange
                        // reason I cannot comprehend. So we only read from clipboard when we
                        // need to paste.
                        match input {
                            platform::KeyboardInput {
                                key: Some(platform::Key::Insert),
                                state: platform::InputState::Pressed,
                                modifiers: platform::ModifiersState { shift: true, .. },
                            } => events.push(WidgetEvent::Paste(win.clipboard())),

                            platform::KeyboardInput {
                                state,
                                key: Some(key),
                                modifiers,
                            } => match state {
                                platform::InputState::Pressed => {
                                    events.push(WidgetEvent::KeyDown {
                                        key,
                                        modifiers,
                                        repeat: false,
                                    });
                                }
                                platform::InputState::Repeated => {
                                    events.push(WidgetEvent::KeyDown {
                                        key,
                                        modifiers,
                                        repeat: true,
                                    });
                                }
                                platform::InputState::Released => {
                                    events.push(WidgetEvent::KeyUp { key, modifiers });
                                }
                            },
                            _ => {
                                debug!("Ignored keyboard input with unknown key: {:?}", input);
                            }
                        }
                    }
                    WindowEvent::ReceivedCharacter(c, mods) => {
                        events.push(WidgetEvent::CharacterReceived(c, mods));
                    }
                    _ => {}
                };
            }

            // If minimized, don't update or render.
            if minimized {
                continue;
            }

            // Since we may receive multiple resize events at once, instead of responded to each
            // resize event, we handle the resize only once.
            if resized {
                resized = false;
                renderer.handle_resized(win_size_logical);
                events.push(WidgetEvent::Resized(win_size_ui));
            }
            root.event(&WidgetEvent::Tick(delta), &ctx, &mut data);

            // A common case is that we have multiple `CursorMoved` events
            // in one update. In that case we keep only the last one,
            // since the in-betweens will never be seen.
            if events.len() > 1
                && events
                    .iter()
                    .all(|e| matches!(e, WidgetEvent::MouseMove(_)))
            {
                events.drain(..events.len() - 1);
            }

            for ev in events.drain(..) {
                root.event(&ev, &ctx, &mut data);
            }
            if let Some(cursor) = root.cursor() {
                if self.graphics.cursor != Some(cursor) {
                    if let Some(c) = self.graphics.cursors.remove(cursor) {
                        if let Some(prev) = win.set_cursor(Some(c)) {
                            if let Some(name) = self.graphics.cursor {
                                self.graphics.cursors.insert(name, prev);
                            }
                        }
                        self.graphics.cursor = Some(cursor);
                    }
                }
            } else if let Some(prev) = win.set_cursor(None) {
                if let Some(name) = self.graphics.cursor {
                    self.graphics.cursors.insert(name, prev);
                }
                self.graphics.cursor = None;
            }

            update_timer.run(|_avg| {
                root.update(&ctx, &data);
                root.layout(
                    win_size_ui,
                    &LayoutCtx::new(&self.graphics.fonts),
                    &data,
                    &self.env,
                );
            });

            paint_timer.run(|_avg| {
                root.paint(
                    Canvas::new(&ctx, &mut self.graphics, Transform::identity(), win_size_ui),
                    &data,
                );
            });

            render_timer.run(|_avg| {
                renderer
                    .frame(self.graphics.effects(), &mut store)
                    .unwrap_or_else(|err| {
                        error!("error rendering frame: {err}");
                    });

                root.frame(&store, &mut data);
            });

            win.present();
        }
        Ok(())
    }
}
