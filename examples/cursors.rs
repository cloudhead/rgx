use rgx::gfx::*;
use rgx::math::*;
use rgx::ui::text::{FontFormat, FontId};
use rgx::ui::widgets::{Align, Painter, SizedBox, ZStack};
use rgx::ui::Interact;

const CROSS: &[u8] = include_bytes!("assets/cross.rgba");
const DEFAULT_FONT: &[u8] = include_bytes!("assets/monospace.uf2");

fn main() -> anyhow::Result<()> {
    let ui = ZStack::new()
        .push(Align::new(
            SizedBox::new(Painter::new(|mut c, _| {
                c.stroke(Rect::origin(c.size), 1., Rgba8::BLUE);
            }))
            .width(256.)
            .height(256.)
            .set_cursor(None),
        ))
        .push(Align::new(
            SizedBox::new(Painter::new(|mut c, _| {
                c.stroke(Rect::origin(c.size), 1., Rgba8::RED);
            }))
            .width(128.)
            .height(128.)
            .set_cursor(Some("cross")),
        ));

    rgx::logger::init(log::Level::Debug)?;
    rgx::Application::new("button")
        .fonts([(FontId::default(), DEFAULT_FONT, FontFormat::UF2)])?
        .cursor("cross", Image::try_from(CROSS)?, [8, 8])
        .launch(ui, ())
        .map_err(Into::into)
}
