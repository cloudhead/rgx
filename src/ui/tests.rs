use crate::platform::MouseButton;

use super::text::*;
use super::widgets::{WidgetExt, ZStack};
use super::*;

struct Environment<'a, W, T> {
    root: W,
    layout_ctx: LayoutCtx<'a>,
    ctx: Context<'a>,
    env: Env,
    data: PhantomData<T>,
}

impl<'a, T, W: Widget<T>> Environment<'a, W, T> {
    fn new(
        widget: fn() -> W,
        store: &'a HashMap<TextureId, Image>,
        fonts: &'a HashMap<FontId, Font>,
    ) -> Self {
        let ctx = Context::new(Point::default(), store);
        let env = Env::default();
        let layout_ctx = LayoutCtx::new(fonts);

        Self {
            root: widget(),
            layout_ctx,
            ctx,
            env,
            data: PhantomData,
        }
    }

    fn hover(&mut self, point: impl Into<Point2D>, data: &mut T) -> &mut Self {
        let point = point.into();

        self.ctx = Context::new(point, self.ctx.surfaces);
        self.root
            .event(&WidgetEvent::MouseMove(point), &self.ctx, data);
        self
    }

    fn click(&mut self, data: &mut T) -> &mut Self {
        self.root
            .event(&WidgetEvent::MouseDown(MouseButton::Left), &self.ctx, data);
        self.root
            .event(&WidgetEvent::MouseUp(MouseButton::Left), &self.ctx, data);
        self
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Data {
    clicks: u64,
    hot: bool,
}

fn simple_hstack() -> impl Widget<(Data, Data, Data)> + 'static {
    let items = vec![
        Rgba8::RED
            .sized([32., 32.])
            .on_hover(|hot, _, data: &mut (Data, Data, Data)| data.0.hot = hot)
            .boxed(),
        Rgba8::GREEN
            .sized([32., 32.])
            .on_hover(|hot, _, data: &mut (Data, Data, Data)| data.1.hot = hot)
            .boxed(),
        Rgba8::BLUE
            .sized([32., 32.])
            .on_hover(|hot, _, data: &mut (Data, Data, Data)| data.2.hot = hot)
            .boxed(),
    ];
    center(hstack(items).spacing(8.))
}

fn simple_zstack() -> ZStack<(Data, Data)> {
    zstack((
        center(
            Rgba8::BLUE
                .sized([256., 256.])
                .on_click(|_, data: &mut (Data, Data)| {
                    data.1.clicks += 1;
                })
                .on_hover(|hot, _, data| {
                    data.1.hot = hot;
                }),
        ),
        center(
            Rgba8::RED
                .sized([128., 128.])
                .on_click(|_, data: &mut (Data, Data)| {
                    data.0.clicks += 1;
                })
                .on_hover(|hot, _, data| {
                    data.0.hot = hot;
                }),
        ),
    ))
}

#[test]
fn test_simple_zstack_hover() {
    let (store, fonts) = (HashMap::new(), HashMap::new());
    let mut e = Environment::new(simple_zstack, &store, &fonts);
    let mut data: (Data, Data) = Default::default();

    crate::logger::init(log::Level::Debug).unwrap();

    e.root.update(&e.ctx, &data);
    e.root
        .layout(Size::new(512., 512.), &e.layout_ctx, &data, &e.env);

    e.hover([64., 64.], &mut data);
    assert!(!data.1.hot);
    assert!(!data.0.hot);

    e.hover([160., 160.], &mut data);
    assert!(data.1.hot);
    assert!(!data.0.hot);

    e.hover([256., 256.], &mut data);
    assert!(!data.1.hot);
    assert!(data.0.hot);
}

#[test]
fn test_simple_zstack_click() {
    let (store, fonts) = (HashMap::new(), HashMap::new());
    let mut e = Environment::new(simple_zstack, &store, &fonts);
    let mut data = Default::default();

    e.root
        .layout(Size::new(512., 512.), &e.layout_ctx, &data, &e.env);

    e.hover([64., 64.], &mut data).click(&mut data);
    assert_eq!(data.1.clicks, 0);
    assert_eq!(data.0.clicks, 0);

    e.hover([160., 160.], &mut data).click(&mut data);
    assert_eq!(data.1.clicks, 1);
    assert_eq!(data.0.clicks, 0);

    e.hover([256., 256.], &mut data).click(&mut data);
    assert_eq!(data.1.clicks, 1);
    assert_eq!(data.0.clicks, 1);
}

#[test]
fn test_simple_hstack_hover() {
    let (store, fonts) = (HashMap::new(), HashMap::new());
    let mut e = Environment::new(simple_hstack, &store, &fonts);
    let mut data = Default::default();

    e.root
        .layout(Size::new(512., 512.), &e.layout_ctx, &data, &e.env);

    e.hover([0., 0.], &mut data);
    assert!(!data.0.hot);
    assert!(!data.1.hot);
    assert!(!data.2.hot);

    e.hover([216., 256.], &mut data);
    assert!(data.0.hot);
    assert!(!data.1.hot);
    assert!(!data.2.hot);

    e.hover([256., 256.], &mut data);
    assert!(!data.0.hot);
    assert!(data.1.hot);
    assert!(!data.2.hot);

    e.hover([296., 256.], &mut data);
    assert!(!data.0.hot);
    assert!(!data.1.hot);
    assert!(data.2.hot);
}
