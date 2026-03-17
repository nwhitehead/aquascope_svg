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
    pub fn new(x: f32, y: f32, txt: &str) -> Self {
        Self {
            x,
            y,
            charwidth: 19.0,
            lineheight: 24.0,
            style: "font: 32px monospace;".into(),
            txt: txt.into(),
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
        let tnode = svg::node::element::Text::new(self.txt.clone())
            .set("fill", "black")
            .set("style", self.style.clone())
            .set("x", self.x)
            .set("y", self.y + self.lineheight);
        doc.add(tnode)
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

pub struct SepLine {
    x: f32,
    y: f32,
    h: f32,
}

impl SepLine {
    fn new(h: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            h
        }
    }
}

impl Drawable for SepLine {
    fn translate(&mut self, tx: f32, ty: f32) {
        self.x += tx;
        self.y += ty;
    }
    fn bounding_box(&self) -> Rect {
        Rect::new(self.x, self.y, 0.0, self.h)
    }
    fn draw(&self, doc: Document) -> Document {
        let (x, y, h) = (self.x, self.y, self.h);
        let data = Data::new()
            .move_to((x, y))
            .line_by((0, h))
            .close();
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "gray")
            .set("stroke-width", 1)
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

struct Padding {
    top: f32,
    left: f32,
    bottom: f32,
    right: f32,
}

impl Padding {
    fn new(top: f32, left: f32, bottom: f32, right: f32) -> Self {
        Self { top, left, bottom, right }
    }
}

struct Padded {
    item: Box<dyn Drawable>,
    padding: Padding,
}

impl Padded {
    fn pad_uniform(item: Box<dyn Drawable>, d: f32) -> Self {
        Self {
            item,
            padding: Padding::new(d, d, d, d)
        }
    }
    fn pad_x(item: Box<dyn Drawable>, d: f32) -> Self {
        Self {
            item,
            padding: Padding::new(0.0, d, 0.0, d)
        }
    }
    fn pad(item: Box<dyn Drawable>, padding: Padding) -> Self {
        Self {
            item,
            padding,
        }
    }
}

impl Drawable for Padded {
    fn translate(&mut self, tx: f32, ty: f32) {
        self.item.translate(tx, ty);
    }
    fn bounding_box(&self) -> Rect {
        let bb = self.item.bounding_box();
        let Padding { top, left, bottom, right } = self.padding;
        Rect {
            x: bb.x - left,
            y: bb.y - top,
            w: bb.w + left + right,
            h: bb.h + top + bottom,
        }
    }
    fn draw(&self, doc: Document) -> Document {
        self.item.draw(doc)
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

fn extend_bb(left: &Rect, right: &Rect) -> Rect {
    let left_xmax = left.x + left.w;
    let left_ymax = left.y + left.h;
    let right_xmax = right.x + right.w;
    let right_ymax = right.y + right.h;
    let res_x = f32::min(left.x, right.x);
    let res_y = f32::min(left.y, right.y);
    let res_w = f32::max(left_xmax, right_xmax) - res_x;
    let res_h = f32::max(left_ymax, right_ymax) - res_y;
    Rect::new(res_x, res_y, res_w, res_h)
}

// General stacking function that can align and sequence along X and Y directions
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
            let item_bb = item.bounding_box();
            bb = Some(extend_bb(&b, &item_bb));
        } else {
            bb = Some(item.bounding_box());
        }
        c.push(item);
    }
    c
}

/// Stack centers on top of each other
pub fn stack(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::Centered, FormulaType::Centered)
}
/// Sequence to the right, aligning midlines
pub fn hstack(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::Sequenced, FormulaType::Centered)
}
/// Sequence to the right, aligning tops
pub fn hstack_top(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::Sequenced, FormulaType::AlignLow)
}
/// Sequence to the right, aligning bottoms
pub fn hstack_bottom(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::Sequenced, FormulaType::AlignHigh)
}
/// Sequence vertically down, aligning centers
pub fn vstack(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::Centered, FormulaType::Sequenced)
}
/// Sequence vertically down, aligning left
pub fn vstack_left(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::AlignLow, FormulaType::Sequenced)
}
/// Sequence vertically down, aligning right
pub fn vstack_right(items: Vec<Box<dyn Drawable>>) -> GArray {
    stack_general(items, FormulaType::AlignHigh, FormulaType::Sequenced)
}

/// Create a box around an existing drawable
pub fn box_around(item: &dyn Drawable, dist: f32) -> GBox {
    let bb = outline(item.bounding_box(), dist);
    GBox::new(bb.x, bb.y, bb.w, bb.h)
}

/// Just string of text
pub fn text(txt: &str) -> Text {
    Text::new(0.0, 0.0, &txt)
}

/// Put a string in a box
pub fn text_in_box(txt: &str, dist: f32) -> GArray {
    let txtobj = Text::new(0.0, 0.0, &txt);
    let bb = box_around(&txtobj, dist);
    stack(vec![Box::new(txtobj), Box::new(bb)])
}

/// Sequence horizontally, aligning midline, inserting vertical bars
pub fn hstack_spacers(items: Vec<Box<dyn Drawable>>, hspace: f32, vheight: f32) -> GArray {
    let mut spaced: Vec<Box<dyn Drawable>> = vec![];
    for mut item in items {
        if spaced.len() > 0 {
            spaced.push(Box::new(Padded::pad_x(Box::new(SepLine::new(vheight)), hspace)));
        }
        spaced.push(item);
    }
    hstack(spaced)
}

pub fn render(x: &dyn Drawable) -> Document {
    x.draw(Document::new())
        .set("viewBox", view_box(outline(x.bounding_box(), 100.0)))
}
