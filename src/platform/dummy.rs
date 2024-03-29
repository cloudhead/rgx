use std::io;

use crate::gfx::color::Image;
use crate::math::Point2D;
use crate::platform::{GraphicsContext, LogicalSize, WindowEvent, WindowHint};

pub struct DummyWindow(());

#[derive(Debug)]
pub struct Cursor {}

impl Cursor {
    pub fn create(_image: &Image, _origin: Point2D<u32>) -> Self {
        unreachable!()
    }
}

pub struct Events {
    #[allow(dead_code)]
    handle: (),
}

impl Events {
    pub fn wait(&mut self) {}

    pub fn wait_timeout(&mut self, _timeout: std::time::Duration) {}

    pub fn poll(&mut self) {}

    pub fn flush(&self) -> impl Iterator<Item = WindowEvent> {
        std::iter::empty::<WindowEvent>()
    }
}

pub struct Window {
    handle: DummyWindow,
}

impl Window {
    pub fn request_redraw(&self) {
        unreachable!()
    }

    pub fn handle(&self) -> &DummyWindow {
        &self.handle
    }

    pub fn get_proc_address(&mut self, _s: &str) -> *const std::ffi::c_void {
        unreachable!()
    }

    pub fn get_cursor_pos(&self) -> (f64, f64) {
        unreachable!()
    }

    pub fn set_cursor(&mut self, _cursor: Option<Cursor>) -> Option<Cursor> {
        unreachable!()
    }

    pub fn set_cursor_visible(&mut self, _visible: bool) {
        unreachable!()
    }

    pub fn scale_factor(&self) -> f64 {
        unreachable!()
    }

    pub fn size(&self) -> LogicalSize {
        unreachable!()
    }

    pub fn is_focused(&self) -> bool {
        true
    }

    pub fn is_closing(&self) -> bool {
        false
    }

    pub fn is_open(&self) -> bool {
        true
    }

    pub fn present(&self) {}

    pub fn clipboard(&self) -> Option<String> {
        None
    }
}

pub fn init(
    _title: &str,
    _w: u32,
    _h: u32,
    _hints: &[WindowHint],
    _context: GraphicsContext,
) -> io::Result<(Window, Events)> {
    panic!("`dummy` platform initialized");
}
