use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

#[derive(Clone)]
pub struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}

pub trait Drawable {
    fn translate(&mut self, tx: f32, ty: f32);
    fn bounding_box(&self) -> Rect;
    fn draw(&self, doc: Document) -> Document;
}

#[derive(Clone)]
pub struct Text {
    x: f32,
    y: f32,
    charwidth: f32,
    lineheight: f32,
    txt: String,
    style: String,
}

impl Text {
    pub fn new(x: f32, y: f32, txt: String) -> Self {
        Self {
            x,
            y,
            charwidth: 19.0,
            lineheight: 24.0,
            style: "font: 32px monospace;".into(),
            txt,
        }
    }
}

impl Drawable for Text {
    fn translate(&mut self, tx: f32, ty: f32) {
        self.x += tx;
        self.y += ty;
    }
    fn bounding_box(&self) -> Rect {
        Rect::new(
            self.x,
            self.y,
            self.charwidth * (self.txt.len() as f32),
            self.lineheight,
        )
    }
    fn draw(&self, doc: Document) -> Document {
        println!("Drawing text at p=({}, {})", self.x, self.y);
        let tnode = svg::node::element::Text::new(self.txt.clone())
            .set("fill", "black")
            .set("style", self.style.clone())
            .set("x", self.x)
            .set("y", self.y + self.lineheight);
        doc.add(tnode)
    }
}

#[derive(Clone)]
pub struct GBox {
    r: Rect,
}

impl GBox {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            r: Rect::new(x, y, w, h),
        }
    }
}

fn translate(r: &Rect, tx: f32, ty: f32) -> Rect {
    Rect {
        x: r.x + tx,
        y: r.y + ty,
        w: r.w,
        h: r.h,
    }
}

impl Drawable for GBox {
    fn translate(&mut self, tx: f32, ty: f32) {
        self.r = translate(&self.r, tx, ty);
    }
    fn bounding_box(&self) -> Rect {
        self.r.clone()
    }
    fn draw(&self, doc: Document) -> Document {
        let (x, y, w, h) = (self.r.x, self.r.y, self.r.w, self.r.h);
        let data = Data::new()
            .move_to((x, y))
            .line_by((w, 0))
            .line_by((0, h))
            .line_by((-w, 0))
            .close();

        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 3)
            .set("d", data);

        doc.add(path)
    }
}

pub fn view_box(r: Rect) -> (f32, f32, f32, f32) {
    (r.x, r.y, r.w, r.h)
}

pub fn outline(rect: Rect, d: f32) -> Rect {
    Rect {
        x: rect.x - d,
        y: rect.y - d,
        w: rect.w + 2.0 * d,
        h: rect.h + 2.0 * d,
    }
}

pub struct GArray {
    items: Vec<Box<dyn Drawable>>,
}

impl GArray {
    pub fn push(&mut self, item: Box<dyn Drawable>) {
        self.items.push(item);
    }
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}

impl Drawable for GArray {
    fn translate(&mut self, tx: f32, ty: f32) {
        for item in &mut self.items {
            item.translate(tx, ty);
        }
    }

    fn bounding_box(&self) -> Rect {
        let mut x: f32 = 1000.0;
        let mut y: f32 = 1000.0;
        let mut x2: f32 = -1000.0;
        let mut y2: f32 = -1000.0;
        for item in &self.items {
            let bb = item.bounding_box();
            x = x.min(bb.x);
            y = y.min(bb.y);
            x2 = x2.max(bb.x + bb.w);
            y2 = y2.max(bb.y + bb.h);
        }
        Rect {
            x,
            y,
            w: x2 - x,
            h: y2 - y,
        }
    }
    fn draw(&self, doc: Document) -> Document {
        let mut d = doc;
        for item in &self.items {
            d = item.draw(d);
        }
        d
    }
}

enum FormulaType {
    AlignLow,
    AlignHigh,
    Centered,
    Sequenced,
}

fn apply_formula(formula: &FormulaType, x: f32, w: f32, ix: f32, iw: f32) -> f32 {
    match formula {
        FormulaType::Sequenced => (x + w) - ix,
        FormulaType::Centered => (x + 0.5 * w) - (ix + 0.5 * iw),
        FormulaType::AlignLow => x - ix,
        FormulaType::AlignHigh => x + w - ix,
    }
}

fn stack_general(
    items: Vec<Box<dyn Drawable>>,
    tx_formula: FormulaType,
    ty_formula: FormulaType,
) -> GArray {
    let mut bb: Option<Rect> = None;
    let mut c = GArray::new();
    for mut item in items {
        let item_bb = item.bounding_box().clone();
        if let Some(ref b) = bb {
            let tx = apply_formula(&tx_formula, b.x, b.w, item_bb.x, item_bb.w);
            let ty = apply_formula(&ty_formula, b.y, b.h, item_bb.y, item_bb.h);
            item.translate(tx, ty);
        } else {
            bb = Some(item.bounding_box());
        }
        c.push(item);
    }
    c
}

pub fn stack(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::Centered, FormulaType::Centered)
}
pub fn hstack(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::Sequenced, FormulaType::Centered)
}
pub fn hstack_top(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::Sequenced, FormulaType::AlignLow)
}
pub fn hstack_bottom(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::Sequenced, FormulaType::AlignHigh)
}
pub fn vstack(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::Centered, FormulaType::Sequenced)
}
pub fn vstack_left(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::AlignLow, FormulaType::Sequenced)
}
pub fn vstack_right(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::AlignHigh, FormulaType::Sequenced)
}

pub fn box_around(item: &dyn Drawable, dist: f32) -> GBox {
    let bb = outline(item.bounding_box(), dist);
    GBox::new(bb.x, bb.y, bb.w, bb.h)
}

pub fn text_in_box(txt: String, dist: f32) -> GArray {
    let txtobj = Text::new(0.0, 0.0, txt);
    let bb = box_around(&txtobj, dist);
    stack(vec![Box::new(txtobj), Box::new(bb)])
}

pub fn render(x: &dyn Drawable) -> Document {
    x.draw(Document::new())
        .set("viewBox", view_box(outline(x.bounding_box(), 100.0)))
}
