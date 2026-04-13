#![allow(unused)]

use ab_glyph::{Point, Rect, point};
use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use tiny_skia::{Color, ColorU8};

use crate::arrow::{ArcOptions, Arrow, ArrowOptions, ArrowOutline, ArrowType, FluidOptions};
use crate::canvas::Canvas;
use crate::draw::{
    Drawable, FormulaType, GArray, GBox, GLine, GPadding, GSpace, GTagged, GText, border,
    compute_align, hstack, hstack_top, norm, scale, vstack_left, vstack_none,
};
use crate::draw_state::DrawState;
use crate::states::{Def, Location, NamedStruct, Program, Ptr, Region, Step, Value};
use crate::style::{
    Styling, color, dark_style, dark_transparent_style, light_style, light_transparent_style,
};

const SMALL_LOOP_THRESHOLD: f32 = 50.0;

/// Keep track of rendering stuff
// We would like to just put all the details about tags and rects in here, but
// the layout algorithms use methods and move things around. We have to wait
// until everything is drawn to know where the bounding boxes of targets for
// pointers actually are for rendering arrows. So we can't register values in
// the RenderState with bounding boxes as we encounter them, we have to somehow
// revisit them at the end. This is done with GTagged drawing elements, we can
// look up tags in the drawables after everything is laid out.
//
// We can keep track of IndexLocation values for sources and targets as we
// encounter them since they don't change during layout.
#[derive(Clone, Debug)]
pub struct RenderState {
    pub style: Styling,
    skip_heap: bool,
    /// Which ids we've seen (these are valid targets of arrows)
    ids: Vec<String>,
    /// Keep track of step location names to avoid duplicates (duplicates screw
    /// up arrow drawing, uniqueness of labels)
    step_names: Vec<String>,
    /// Set of pointers we will need to draw, containing (dst prefix, src prefix, ptr stuff)
    // The prefix stuff is needed for dst to know what step location label is
    // (e.g. L0:...) The prefix is needed for src to know precise stuff about
    // label we are coming from (e..g L0:x.0.1) The Ptr part itself has bare
    // target label, like "x" or whatever, plus information about selectors
    // (e.g. .1.0) and borrows
    ptrs: Vec<(String, String, Ptr)>,
    /// Remember IndexLocations for tagged values
    idxmap: HashMap<String, IndexLocation>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            style: Default::default(),
            skip_heap: true,
            ids: vec![],
            step_names: vec![],
            ptrs: vec![],
            idxmap: Default::default(),
        }
    }
}

impl RenderState {
    pub fn register(&mut self, id: &str, idx_loc: &IndexLocation) {
        self.ids.push(id.to_string());
        self.idxmap.insert(id.to_string(), idx_loc.clone());
    }
    pub fn ids(&self) -> Vec<String> {
        self.ids.clone()
    }
    pub fn clear_ids(&mut self) {
        self.ids = vec![];
        self.step_names = vec![];
    }
    pub fn register_step(&mut self, name: &str) {
        self.step_names.push(name.to_string());
    }
    pub fn step_names(&self) -> Vec<String> {
        self.step_names.clone()
    }
    pub fn lookup(&self, name: &str) -> Option<&IndexLocation> {
        self.idxmap.get(name)
    }
}

/// Keep track of which step and which location things are in (e.g. L0 is step 0, "Heap" might be location index 1)
// These index values help with arrow layout decisions
#[derive(Clone, Debug)]
pub struct IndexLocation {
    pub step_idx: usize,
    pub location_idx: usize,
}

impl IndexLocation {
    fn new(step_idx: usize, location_idx: usize) -> Self {
        Self {
            step_idx,
            location_idx,
        }
    }
}

fn max_height(values: &Vec<Box<dyn Drawable>>, canvas: &Canvas) -> Result<f32> {
    let mut res: f32 = 0.0;
    for value in values {
        let bb = value.bounding_box(canvas)?;
        let h = bb.max.y - bb.min.y;
        res = res.max(h);
    }
    Ok(res)
}

fn render_value_array(
    a: &[Value],
    prefix: &str,
    ptr_dst_prefix: &str,
    loc_idx: &IndexLocation,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    // Draw all the parts separately
    let mut a_draws: Vec<Box<dyn Drawable>> = vec![];
    for (idx, x) in a.iter().enumerate() {
        let draw = render_value(
            x,
            &format!("{}.{}", prefix, idx),
            ptr_dst_prefix,
            loc_idx,
            render_state,
            canvas,
        )?;
        a_draws.push(draw);
    }
    let style = &render_state.style;
    if a_draws.is_empty() {
        a_draws.push(
            GSpace::new(
                style.get_number_or("value.array.empty.w", 5.0),
                style.get_number_or("value.array.empty.h", 5.0),
            )
            .clone_box(),
        );
    }
    let mut ds = DrawState::default();
    // Now measure the height for divider lines
    let h = max_height(&a_draws, canvas)?;
    let sep_margin = style.get_number_or("value.array.separator.vmargin", 5.0);
    // intersperse vertical lines
    ds.stroke_color = style.get_color_or("value.array.separator.color", color("#000")?);
    let sep = GLine::new(point(0.0, 0.0), point(0.0, h - sep_margin), ds.clone());
    let sep_padding = style.get_padding("value.array.separator.padding", 5.0);
    let padded_sep = GPadding::new(Box::new(sep), sep_padding);
    let mut a_draws_sep: Vec<Box<dyn Drawable>> = vec![];
    let mut any_elems_yet = false;
    for x in a_draws {
        if any_elems_yet {
            a_draws_sep.push(padded_sep.clone_box());
        } else {
            any_elems_yet = true;
        }
        a_draws_sep.push(x);
    }
    let stk = hstack(a_draws_sep, canvas)?;
    ds.padding = style.get_padding("value.array.padding", 5.0);
    ds.stroke_color = style.get_color_or("value.array.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("value.array.border.width", 4.0);
    ds.border_radius = style.get_radius("value.array.border.radius", 5.0);
    let res = border(Box::new(stk), canvas, ds)?;
    Ok(res)
}

fn render_value_tuple(
    a: &[Value],
    prefix: &str,
    ptr_dst_prefix: &str,
    loc_idx: &IndexLocation,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    // Draw all the parts separately
    let mut a_draws: Vec<Box<dyn Drawable>> = vec![];
    for (idx, x) in a.iter().enumerate() {
        let draw = render_value(
            x,
            &format!("{}.{}", prefix, idx),
            ptr_dst_prefix,
            loc_idx,
            render_state,
            canvas,
        )?;
        a_draws.push(draw);
    }
    let style = &render_state.style;
    if a_draws.is_empty() {
        a_draws.push(
            GSpace::new(
                style.get_number_or("value.tuple.empty.w", 5.0),
                style.get_number_or("value.tuple.empty.h", 5.0),
            )
            .clone_box(),
        );
    }
    let mut ds = DrawState::default();
    // Now measure the height for divider lines
    let h = max_height(&a_draws, canvas)?;
    let sep_margin = style.get_number_or("value.tuple.separator.vmargin", 5.0);
    // intersperse vertical lines
    ds.stroke_color = style.get_color_or("value.tuple.separator.color", color("#000")?);
    let sep = GLine::new(point(0.0, 0.0), point(0.0, h - sep_margin), ds.clone());
    let sep_padding = style.get_padding("value.tuple.separator.padding", 5.0);
    let padded_sep = GPadding::new(Box::new(sep), sep_padding);
    let mut a_draws_sep: Vec<Box<dyn Drawable>> = vec![];
    let mut any_elems_yet = false;
    for x in a_draws {
        if any_elems_yet {
            a_draws_sep.push(padded_sep.clone_box());
        } else {
            any_elems_yet = true;
        }
        a_draws_sep.push(x);
    }
    let stk = hstack(a_draws_sep, canvas)?;
    ds.padding = style.get_padding("value.tuple.padding", 5.0);
    ds.stroke_color = style.get_color_or("value.tuple.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("value.tuple.border.width", 4.0);
    ds.border_radius = style.get_radius("value.tuple.border.radius", 5.0);
    let res = border(Box::new(stk), canvas, ds)?;
    Ok(res)
}

fn render_def(
    def: &Def,
    prefix: &str,
    ptr_dst_prefix: &str,
    loc_idx: &IndexLocation,
    render_state: &mut RenderState,
    canvas: &Canvas,
    skip_heap: bool,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let mut ds = DrawState {
        font: style.get_string_or("def.label.font", "mono"),
        font_size: style.get_number_or("def.label.font_size", 24.0),
        text_color: style.get_color_or("def.label.color", color("#000")?),
        ..DrawState::default()
    };

    let mut g_label = GText::new(&def.label, point(0.0, 0.0), ds.clone());

    ds.font = style.get_string_or("def.separator.font", "mono");
    ds.font_size = style.get_number_or("def.separator.font_size", 24.0);
    ds.text_color = style.get_color_or("def.separator.color", color("#000")?);
    let sep_text = style.get_string_or("def.separator.text", ":");
    let g_separator = GText::new(&sep_text, point(0.0, 0.0), ds.clone());

    let sep_padding = style.get_padding("def.separator.padding", 0.0);
    let g_padded_sep = GPadding::new(Box::new(g_separator), sep_padding);

    // Make sure final drawable has x=0 as the dividing line for separator
    // (so we can align them later)
    // Instead of just doing hstack_... we compute the translation and use parts of it
    // Move label left to align right side to separator
    let label_bb = g_label.bounding_box(canvas)?;
    let sep_bb = g_padded_sep.bounding_box(canvas)?;
    let p = compute_align(
        &label_bb,
        &sep_bb,
        FormulaType::Sequenced,
        FormulaType::Centered,
    );
    g_label.translate(point(-p.x, -p.y));
    let mut left = GArray::new();
    left.push(Box::new(g_label));
    left.push(Box::new(g_padded_sep));

    let left_padding = style.get_padding("def.left.padding", 0.0);
    let g_left = GPadding::new(Box::new(left), left_padding);
    let left_bb = g_left.bounding_box(canvas)?;

    ds.padding = style.get_padding("def.value.padding", 0.0);
    ds.margin = style.get_padding("def.value.margin", 0.0);
    ds.stroke_color = style.get_color_or("def.value.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("def.value.border.width", 4.0);
    ds.border_radius = style.get_radius("def.value.border.radius", 5.0);
    let prefix = &format!("{}:{}", prefix, def.label);
    // Note that ptr_dst_prefix just passes through here, it is for getting common step name prefix only.
    // Values can be L0:x.0.1 or something, even inside we still want ptr_dst_prefix to be just L0
    // Point is to convert a pointer to "x.0" into the label "L0:x.0".
    let g_value = render_value(
        &def.value,
        prefix,
        ptr_dst_prefix,
        loc_idx,
        render_state,
        canvas,
    )?;
    let mut g_border_value = border(g_value, canvas, ds.clone())?;

    // Now align the value to right of separator, centered vertically
    let value_bb = g_border_value.bounding_box(canvas)?;
    let p = compute_align(
        &left_bb,
        &value_bb,
        FormulaType::Sequenced,
        FormulaType::Centered,
    );
    g_border_value.translate(p);

    let mut g_array = GArray::new();
    // if skip_heap is off, always show label
    // if skip_heap is on, show label if it doesn't start with H
    if !skip_heap || !def.label.starts_with("H") {
        g_array.push(Box::new(g_left));
    }
    g_array.push(g_border_value);
    Ok(Box::new(g_array))
}

fn render_value_struct(
    named_struct: &NamedStruct,
    prefix: &str,
    ptr_dst_prefix: &str,
    loc_idx: &IndexLocation,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    // Draw all the inner values separately first (to measure for sep line height)
    let mut v_draws: Vec<Box<dyn Drawable>> = vec![];
    for (idx, p) in named_struct.fields.clone().into_iter().enumerate() {
        let value = &p.1;
        let prefix = &format!("{}.{}", prefix, idx);
        let draw = render_value(value, prefix, ptr_dst_prefix, loc_idx, render_state, canvas)?;
        v_draws.push(draw);
    }
    // Now measure the height for divider lines
    let h = max_height(&v_draws, canvas)?;

    let mut ds = DrawState::default();
    let style = &render_state.style;
    ds.font = style.get_string_or("value.struct.name.font", "mono");
    ds.font_size = style.get_number_or("value.struct.name.font_size", 24.0);
    ds.text_color = style.get_color_or("value.struct.name.color", color("#000")?);

    // intersperse vertical lines
    let div_margin = style.get_number_or("value.struct.divider.vmargin", 5.0);
    ds.stroke_color = style.get_color_or("value.struct.divider.color", color("#000")?);
    let div = GLine::new(point(0.0, 0.0), point(0.0, h - div_margin), ds.clone());
    let div_padding = style.get_padding("value.struct.divider.padding", 5.0);
    let padded_div = GPadding::new(Box::new(div), div_padding);

    let g_name = GText::new(&named_struct.name, point(0.0, 0.0), ds.clone());

    let mut body: Vec<Box<dyn Drawable>> = vec![];

    for (i, p) in named_struct.fields.iter().enumerate() {
        let label = &p.0;
        ds.font = style.get_string_or("value.struct.label.font", "mono");
        ds.font_size = style.get_number_or("value.struct.label.font_size", 24.0);
        ds.text_color = style.get_color_or("value.struct.label.color", color("#000")?);
        let g_label = GText::new(label, point(0.0, 0.0), ds.clone());

        ds.font = style.get_string_or("value.struct.separator.font", "mono");
        ds.font_size = style.get_number_or("value.struct.separator.font_size", 24.0);
        ds.text_color = style.get_color_or("value.struct.separator.color", color("#000")?);
        let sep_text = style.get_string_or("value.struct.separator.text", ":");
        let g_separator = GText::new(&sep_text, point(0.0, 0.0), ds.clone());

        let sep_padding = style.get_padding("value.struct.separator.padding", 0.0);
        let g_padded_sep = GPadding::new(Box::new(g_separator), sep_padding);

        let left: Vec<Box<dyn Drawable>> = vec![Box::new(g_label), Box::new(g_padded_sep)];
        let g_left = hstack(left, canvas)?;
        let left_padding = style.get_padding("value.struct.left.padding", 0.0);
        let g_padded_left = GPadding::new(Box::new(g_left), left_padding);

        if i > 0 {
            // Add dividing line after 1st element before each field
            body.push(padded_div.clone_box());
        }
        body.push(Box::new(g_padded_left));
        body.push(v_draws[i].clone_box());
    }
    if body.is_empty() {
        body.push(
            GSpace::new(
                style.get_number_or("value.struct.empty.w", 5.0),
                style.get_number_or("value.struct.empty.h", 5.0),
            )
            .clone_box(),
        );
    }
    let g_body = hstack(body, canvas)?;

    ds.padding = style.get_padding("value.struct.padding", 0.0);
    ds.margin = style.get_padding("value.struct.margin", 0.0);
    ds.stroke_color = style.get_color_or("value.struct.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("value.struct.border.width", 4.0);
    ds.border_radius = style.get_radius("value.struct.border.radius", 5.0);
    let g_border_body = border(Box::new(g_body), canvas, ds.clone())?;

    let g_array = hstack(vec![Box::new(g_name), g_border_body], canvas)?;
    Ok(Box::new(g_array))
}

fn render_value_invalid(
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let ds = DrawState {
        font: style.get_string_or("value.invalid.font", "mono"),
        text_color: style.get_color_or("value.invalid.color", color("#000")?),
        font_size: style.get_number_or("value.invalid.font_size", 24.0),
        ..DrawState::default()
    };
    let padding = style.get_padding("value.invalid.padding", 5.0);
    let text = "×".to_string();
    let gtxt = GText::new(&text, point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), padding);
    Ok(Box::new(padded_gtxt))
}

fn render_value_number(
    v: f64,
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let ds = DrawState {
        font: style.get_string_or("value.number.font", "mono"),
        text_color: style.get_color_or("value.number.color", color("#000")?),
        font_size: style.get_number_or("value.number.font_size", 24.0),
        ..DrawState::default()
    };
    let padding = style.get_padding("value.number.padding", 5.0);
    let text = format!("{}", v);
    let gtxt = GText::new(&text, point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), padding);
    Ok(Box::new(padded_gtxt))
}

fn render_value_char(
    c: char,
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let style = &render_state.style;
    let ds = DrawState {
        font: style.get_string_or("value.char.font", "mono"),
        text_color: style.get_color_or("value.char.color", color("#000")?),
        font_size: style.get_number_or("value.char.font_size", 24.0),
        ..DrawState::default()
    };
    let padding = style.get_padding("value.char.padding", 5.0);
    let text = format!("'{}'", c);
    let gtxt = GText::new(&text, point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), padding);
    Ok(Box::new(padded_gtxt))
}

fn render_value_pointer(
    p: &Ptr,
    prefix: &str,
    ptr_dst_prefix: &str,
    render_state: &mut RenderState,
    _canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    // Record the pointer information into render_state for drawing arrows after layout
    // Clone the ptr, but also extend the destination label to have the ptr_dst_prefix
    render_state
        .ptrs
        .push((ptr_dst_prefix.to_string(), prefix.to_string(), p.clone()));
    let style = &render_state.style;
    let ds = DrawState {
        font: style.get_string_or("value.pointer.font", "mono"),
        text_color: style.get_color_or("value.pointer.color", color("#000")?),
        font_size: style.get_number_or("value.pointer.font_size", 24.0),
        ..DrawState::default()
    };
    let padding = style.get_padding("value.pointer.padding", 0.0);
    // Some unicode choices related to pointers ✕✖✗✘×•●○◯
    let text = "●";
    let g_txt = GText::new(text, point(0.0, 0.0), ds);
    let g_padded = GPadding::new(Box::new(g_txt), padding);
    Ok(Box::new(g_padded))
}

pub fn render_value(
    value: &Value,
    prefix: &str,
    ptr_dst_prefix: &str,
    loc_idx: &IndexLocation,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    let item = match value {
        Value::Number(v) => render_value_number(*v, render_state, canvas)?,
        Value::Char(c) => render_value_char(*c, render_state, canvas)?,
        Value::Pointer(p) => render_value_pointer(p, prefix, ptr_dst_prefix, render_state, canvas)?,
        Value::Array(a) => {
            render_value_array(a, prefix, ptr_dst_prefix, loc_idx, render_state, canvas)?
        }
        Value::Tuple(a) => {
            render_value_tuple(a, prefix, ptr_dst_prefix, loc_idx, render_state, canvas)?
        }
        Value::Struct(a) => {
            render_value_struct(a, prefix, ptr_dst_prefix, loc_idx, render_state, canvas)?
        }
        Value::Invalid => render_value_invalid(render_state, canvas)?,
    };
    // see if tag is already present
    let mut tag = prefix.to_string();
    // see if tag is already present, keep adding ' to end if so
    while render_state.ids().contains(&tag) {
        tag.push('\'');
    }
    let tagged = GTagged::new(item, &tag);
    render_state.register(&tag, loc_idx);
    Ok(Box::new(tagged))
}

pub fn render_region(
    value: &Region,
    prefix: &str,
    ptr_dst_prefix: &str,
    idx: &IndexLocation,
    render_state: &mut RenderState,
    canvas: &Canvas,
    skip_heap: bool,
) -> Result<Box<dyn Drawable>> {
    // Header
    let style = &render_state.style;
    let ds = DrawState {
        font: style.get_string_or("region.header.font", "serif"),
        text_color: style.get_color_or("region.header.color", color("#000")?),
        font_size: style.get_number_or("region.header.font_size", 24.0),
        ..DrawState::default()
    };
    let header_padding = style.get_padding("region.header.padding", 5.0);
    let region_padding = style.get_padding("region.padding", 5.0);
    let gtxt = GText::new(&value.name.to_string(), point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), header_padding);
    // Body
    let mut body: Vec<Box<dyn Drawable>> = vec![];
    for def in &value.definitions {
        let g_def = render_def(
            def,
            prefix,
            ptr_dst_prefix,
            idx,
            render_state,
            canvas,
            skip_heap,
        )?;
        body.push(g_def);
    }
    // stack vertically without moving horizontally (to keep : aligned)
    let g_body = vstack_none(body, canvas)?;
    let g_final = vstack_left(vec![Box::new(padded_gtxt), Box::new(g_body)], canvas)?;
    let g_padded_final = GPadding::new(Box::new(g_final), region_padding);
    Ok(Box::new(g_padded_final))
}

pub fn render_location(
    value: &Location,
    prefix: &str,
    ptr_dst_prefix: &str,
    idx: &IndexLocation,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    if !value.definitions.is_empty() {
        let region = Region {
            name: "".to_string(),
            definitions: value.definitions.clone(),
        };
        return render_location(
            &Location {
                name: value.name.clone(),
                definitions: vec![],
                regions: vec![region],
            },
            prefix,
            ptr_dst_prefix,
            idx,
            render_state,
            canvas,
        );
    }

    let mut skip_heap = render_state.skip_heap;
    if value.name.starts_with("Stack") {
        skip_heap = false;
    }

    // Header
    let style = &render_state.style;
    let ds = DrawState {
        font: style.get_string_or("location.header.font", "serif"),
        text_color: style.get_color_or("location.header.color", color("#000")?),
        font_size: style.get_number_or("location.header.font_size", 24.0),
        ..DrawState::default()
    };
    let header_padding = style.get_padding("location.header.padding", 5.0);
    let gtxt = GText::new(&value.name.to_string(), point(0.0, 0.0), ds);
    let padded_gtxt = GPadding::new(Box::new(gtxt), header_padding);

    // Body
    let mut body: Vec<Box<dyn Drawable>> = vec![];
    for region in &value.regions {
        let g_region = render_region(
            region,
            prefix,
            ptr_dst_prefix,
            idx,
            render_state,
            canvas,
            skip_heap,
        )?;
        body.push(g_region);
    }
    let g_body = vstack_left(body, canvas)?;
    let g_final = vstack_left(vec![Box::new(padded_gtxt), Box::new(g_body)], canvas)?;

    Ok(Box::new(g_final))
}

pub fn render_step(
    value: &Step,
    step_idx: usize,
    render_state: &mut RenderState,
    canvas: &Canvas,
) -> Result<Box<dyn Drawable>> {
    if render_state.step_names().contains(&value.label) {
        bail!("Repeated step location name {}", &value.label);
    }
    render_state.register_step(&value.label);
    // Label
    let style = &render_state.style;
    let mut ds = DrawState {
        font: style.get_string_or("step.header.font", "serif"),
        text_color: style.get_color_or("step.header.color", color("#000")?),
        font_size: style.get_number_or("step.header.font_size", 24.0),
        stroke_color: style.get_color_or("step.separator.color", color("#000")?),
        ..DrawState::default()
    };
    let padding = style.get_padding("step.header.padding", 5.0);
    let g_text = GText::new(&value.label.to_string(), point(0.0, 0.0), ds.clone());
    let g_padded_text = GPadding::new(Box::new(g_text), padding);

    // draw border on right side sized properly
    let h = style.get_number_or("step.separator.size", 5.0);
    let sep = GLine::new(point(0.0, 0.0), point(0.0, h), ds.clone());
    let sep_padding = style.get_padding("step.separator.padding", 5.0);
    let g_padded_sep = GPadding::new(Box::new(sep), sep_padding);

    let style = &render_state.style;
    let mut body: Vec<Box<dyn Drawable>> = vec![];
    body.push(Box::new(g_padded_text));
    body.push(Box::new(g_padded_sep));
    let gap = style.get_number_or("step.location.gap", 5.0);
    for (idx, location) in value.locations.iter().enumerate() {
        let g_location = render_location(
            location,
            &value.label.to_string(),
            &value.label.to_string(),
            &IndexLocation::new(step_idx, idx),
            render_state,
            canvas,
        )?;
        if idx > 0 {
            body.push(Box::new(GSpace::new(gap, 0.0)));
        }
        body.push(g_location);
    }
    let g_body = hstack_top(body, canvas)?;

    let style = &render_state.style;
    ds.padding = style.get_padding("step.padding", 5.0);
    ds.margin = style.get_padding("step.margin", 5.0);
    ds.stroke_color = style.get_color_or("step.border.color", color("#000")?);
    ds.stroke.width = style.get_number_or("step.border.width", 4.0);
    ds.border_radius = style.get_radius("step.border.radius", 5.0);
    let res = border(Box::new(g_body), canvas, ds)?;

    Ok(res)
}

#[derive(PartialEq)]
enum Direction {
    Auto,
    Top,
    Right,
    Bottom,
    Left,
}

fn pick_side(r: &Rect, d: &Direction) -> Point {
    match d {
        Direction::Top => point((r.min.x + r.max.x) * 0.5, r.min.y),
        Direction::Right => point(r.max.x, (r.min.y + r.max.y) * 0.5),
        Direction::Bottom => point((r.min.x + r.max.x) * 0.5, r.max.y),
        Direction::Left => point(r.min.x, (r.min.y + r.max.y) * 0.5),
        _ => panic!("unimplemented"),
    }
}

fn get_direction_vector(d: &Direction) -> Point {
    match d {
        Direction::Top => point(0.0, -1.0),
        Direction::Right => point(1.0, 0.0),
        Direction::Bottom => point(0.0, 1.0),
        Direction::Left => point(-1.0, 0.0),
        _ => panic!("unimplemented"),
    }
}

fn choose_arrow(
    src_rect: &Rect,
    dst_rect: &Rect,
    src_iloc: &IndexLocation,
    dst_iloc: &IndexLocation,
    help: &[String],
    style: &Styling,
) -> Arrow {
    let mut src_direction = Direction::Auto;
    let mut dst_direction = Direction::Auto;
    let mut color = 0; // index
    let mut src_gravity = 5;
    let mut dst_gravity = 5;
    for h in help {
        match h.as_str() {
            ".sn" => src_direction = Direction::Top,
            ".se" => src_direction = Direction::Right,
            ".ss" => src_direction = Direction::Bottom,
            ".sw" => src_direction = Direction::Left,
            ".dn" => dst_direction = Direction::Top,
            ".de" => dst_direction = Direction::Right,
            ".ds" => dst_direction = Direction::Bottom,
            ".dw" => dst_direction = Direction::Left,
            ".e" => {
                src_direction = Direction::Right;
                dst_direction = Direction::Right;
            }
            ".w" => {
                src_direction = Direction::Left;
                dst_direction = Direction::Left;
            }
            ".n" => {
                src_direction = Direction::Top;
                dst_direction = Direction::Top;
            }
            ".s" => {
                src_direction = Direction::Bottom;
                dst_direction = Direction::Bottom;
            }
            ".sg0" => src_gravity = 0,
            ".sg1" => src_gravity = 1,
            ".sg2" => src_gravity = 2,
            ".sg3" => src_gravity = 3,
            ".sg4" => src_gravity = 4,
            ".sg5" => src_gravity = 5,
            ".sg6" => src_gravity = 6,
            ".sg7" => src_gravity = 7,
            ".sg8" => src_gravity = 8,
            ".sg9" => src_gravity = 9,
            ".dg0" => dst_gravity = 0,
            ".dg1" => dst_gravity = 1,
            ".dg2" => dst_gravity = 2,
            ".dg3" => dst_gravity = 3,
            ".dg4" => dst_gravity = 4,
            ".dg5" => dst_gravity = 5,
            ".dg6" => dst_gravity = 6,
            ".dg7" => dst_gravity = 7,
            ".dg8" => dst_gravity = 8,
            ".dg9" => dst_gravity = 9,
            ".g0" => {
                src_gravity = 0;
                dst_gravity = 0;
            }
            ".g1" => {
                src_gravity = 1;
                dst_gravity = 1;
            }
            ".g2" => {
                src_gravity = 2;
                dst_gravity = 2;
            }
            ".g3" => {
                src_gravity = 3;
                dst_gravity = 3;
            }
            ".g4" => {
                src_gravity = 4;
                dst_gravity = 4;
            }
            ".g5" => {
                src_gravity = 5;
                dst_gravity = 5;
            }
            ".g6" => {
                src_gravity = 6;
                dst_gravity = 6;
            }
            ".g7" => {
                src_gravity = 7;
                dst_gravity = 7;
            }
            ".g8" => {
                src_gravity = 8;
                dst_gravity = 8;
            }
            ".g9" => {
                src_gravity = 9;
                dst_gravity = 9;
            }
            ".c0" => color = 0,
            ".c1" => color = 1,
            ".c2" => color = 2,
            ".c3" => color = 3,
            ".c4" => color = 4,
            ".c5" => color = 5,
            ".c6" => color = 6,
            ".c7" => color = 7,
            ".c8" => color = 8,
            ".c9" => color = 9,
            _ => println!("WARNING: unknown pointer help found, {}", &h),
        }
    }
    // Compute auto directions
    // FIXME: Assumes src and dst rectangles don't overlap (annoying edge cases there)
    let src_mid = point(
        (src_rect.min.x + src_rect.max.x) * 0.5,
        (src_rect.min.y + src_rect.max.y) * 0.5,
    );
    let dst_mid = point(
        (dst_rect.min.x + dst_rect.max.x) * 0.5,
        (dst_rect.min.y + dst_rect.max.y) * 0.5,
    );
    let dx = dst_mid.x - src_mid.x;
    let dy = dst_mid.y - src_mid.y;
    // Default source direction is right unless set otherwise
    if src_direction == Direction::Auto {
        src_direction = Direction::Right;
    }
    let mut min_gravity = 0.0;
    // Loop heuristic: if total distance is very small, draw as a loop e-s
    if dx.abs() + dy.abs() < SMALL_LOOP_THRESHOLD {
        if dst_direction == Direction::Auto {
            dst_direction = Direction::Bottom;
        }
        min_gravity += 40.0;
    }
    // If both locations are in same location (column of values), default to E-E connection
    if src_iloc.location_idx == dst_iloc.location_idx && dst_direction == Direction::Auto {
        dst_direction = Direction::Right;
    }

    // Do horizontal connection if dx is bigger, vertical connection if dy is bigger
    if dx.abs() > dy.abs() {
        if dx > 0.0 && dst_direction == Direction::Auto {
            dst_direction = Direction::Left;
        } else if dst_direction == Direction::Auto {
            dst_direction = Direction::Right;
        }
    } else if dy > 0.0 && dst_direction == Direction::Auto {
        dst_direction = Direction::Top;
    } else if dst_direction == Direction::Auto {
        dst_direction = Direction::Bottom;
    }
    // src of arrow is drawn with square edge, so need to adjust point by 1/2 width of arrow
    let width = style.get_number_or("arrow.width", 1.0);
    let src_gap = width * 0.5 + style.get_number_or("arrow.src.gap", 1.0);
    let dst_gap = style.get_number_or("arrow.dst.gap", 1.0);
    let src_p =
        pick_side(src_rect, &src_direction) + scale(get_direction_vector(&src_direction), src_gap);
    let dst_p =
        pick_side(dst_rect, &dst_direction) + scale(get_direction_vector(&dst_direction), dst_gap);
    let dist = norm(point(src_p.x - dst_p.x, src_p.y - dst_p.y));
    let src_gravity_f = min_gravity + dist * (src_gravity as f32) / 5.0 * 0.3;
    let dst_gravity_f = min_gravity + dist * (dst_gravity as f32) / 5.0 * 0.3;
    Arrow::new(
        src_p,
        dst_p,
        ArrowType::Fluid(FluidOptions {
            start_dir: get_direction_vector(&src_direction),
            // end_dir is reversed because arrow is going into that side
            end_dir: scale(get_direction_vector(&dst_direction), -1.0),
            start_gravity: src_gravity_f,
            end_gravity: dst_gravity_f,
        }),
        ArrowOptions {
            width,
            head_length: style.get_number_or("arrow.head.length", 1.0),
            head_width: style.get_number_or("arrow.head.width", 1.0),
            dent_ratio: style.get_number_or("arrow.dent.ratio", 0.0),
            color: style.get_color_or(
                &format!("arrow.color{}", color),
                ColorU8::from_rgba(255, 255, 0, 255),
            ),
            outline: Some(ArrowOutline {
                width: style.get_number_or("arrow.outline.width", 1.0),
                color: style.get_color_or("arrow.outline.color", ColorU8::from_rgba(0, 0, 0, 255)),
            }),
        },
    )
}

pub fn render_program(
    value: &Program,
    canvas: &Canvas,
    style: &Styling,
) -> Result<Box<dyn Drawable>> {
    let mut result = GArray::new();
    let mut rs = RenderState {
        style: style.clone(),
        ..RenderState::default()
    };
    let mut body: Vec<Box<dyn Drawable>> = vec![];
    let gap = style.get_number_or("program.step.gap", 5.0);
    for (idx, step) in value.0.iter().enumerate() {
        let g_location = render_step(step, idx, &mut rs, canvas)?;
        if idx > 0 {
            body.push(Box::new(GSpace::new(0.0, gap)));
        }
        body.push(g_location);
    }
    let g_body = vstack_left(body, canvas)?;
    result.push(g_body.clone_box());

    // Layout should be finished entirely
    // Now draw arrows
    let mut arrows: Vec<Arrow> = vec![];
    for (dst_prefix, src_tag, ptr) in &rs.ptrs {
        let mut dst_tag = String::new();
        dst_tag.push_str(&format!("{}:{}", dst_prefix, ptr.name));
        for idx in &ptr.selectors {
            dst_tag.push_str(&format!(".{}", idx));
        }
        for i in 0..ptr.borrow {
            dst_tag.push('\'');
        }
        let dst_r = g_body
            .get_tagged(&dst_tag)
            .context(format!("could not find tag '{}'", dst_tag))?
            .bounding_box(canvas)?;
        let src_r = g_body
            .get_tagged(src_tag)
            .context(format!("could not find tag '{}'", src_tag))?
            .bounding_box(canvas)?;
        let dst_iloc = rs
            .lookup(&dst_tag)
            .context(format!("need index location for tag {}", &dst_tag))?;
        let src_iloc = rs
            .lookup(src_tag)
            .context(format!("need index location for tag {}", src_tag))?;
        let arrow = choose_arrow(&src_r, &dst_r, src_iloc, dst_iloc, &ptr.help, &rs.style);
        // // Some debug code to draw bounding box
        // result.push(Box::new(GBox::new_with_options(src_r, 2.0, ColorU8::from_rgba(255, 0, 0, 255))));
        result.push(Box::new(arrow));
    }
    Ok(Box::new(result))
}

pub fn draw_program(program: &Program, scale: f32, theme: &str) -> Result<Canvas> {
    let style = match theme {
        "dark" => dark_style()?,
        "light" => light_style()?,
        "dark_transparent" => dark_transparent_style()?,
        "light_transparent" => light_transparent_style()?,
        _ => dark_style()?,
        // "light" => light_style()?,
    };
    // Start with measurement, empty canvas
    let mut canvas = Canvas::new(1, 1, scale)?;
    canvas.load_fonts(&style);
    let mut v = render_program(program, &canvas, &style)?;
    let bb = v.bounding_box(&canvas)?;
    // Translate to 0, 0
    v.translate(point(-bb.min.x, -bb.min.y));
    let w = bb.max.x - bb.min.x;
    let h = bb.max.y - bb.min.y;
    // Now we know size, recreate canvas at right size with fonts
    canvas = Canvas::new(w.ceil() as u32, h.ceil() as u32, scale)?;
    canvas.load_fonts(&style);
    let bgcolor_u8 = style.get_color_or("bg", ColorU8::from_rgba(0, 0, 0, 255));
    let bgcolor = Color::from_rgba8(
        bgcolor_u8.red(),
        bgcolor_u8.green(),
        bgcolor_u8.blue(),
        bgcolor_u8.alpha(),
    );
    canvas.pixmap.fill(bgcolor);
    v.draw(&mut canvas)?;
    Ok(canvas)
}

pub fn draw_program_png(program: &Program, scale: f32, theme: &str) -> Result<Vec<u8>> {
    let canvas = draw_program(program, scale, theme)?;
    canvas.png_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_color() -> Result<()> {
        assert_eq!(color("#000")?, ColorU8::from_rgba(0, 0, 0, 0xff));
        assert_eq!(color("#f00")?, ColorU8::from_rgba(0xff, 0, 0, 0xff));
        assert_eq!(color("#080")?, ColorU8::from_rgba(0, 0x88, 0, 0xff));
        assert_eq!(color("#00a")?, ColorU8::from_rgba(0, 0, 0xaa, 0xff));
        assert_eq!(color("#0008")?, ColorU8::from_rgba(0, 0, 0, 0x88));
        assert_eq!(color("#1234")?, ColorU8::from_rgba(0x11, 0x22, 0x33, 0x44));
        assert_eq!(
            color("#123456")?,
            ColorU8::from_rgba(0x12, 0x34, 0x56, 0xff)
        );
        assert_eq!(
            color("#12345678")?,
            ColorU8::from_rgba(0x12, 0x34, 0x56, 0x78)
        );
        assert!(color("000").is_err());
        assert!(color("#12345").is_err());
        assert!(color("#0g3456").is_err());
        assert!(color("#fffffffff").is_err());
        Ok(())
    }

    #[test]
    pub fn test_render_alpha() -> Result<()> {
        let mut canvas = Canvas::new(800, 800, 1.0)?;
        canvas
            .pixmap
            .fill(Color::from_rgba(0.2, 0.1, 0.3, 1.0).unwrap());

        canvas.load_font(
            "mono",
            include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
        )?;
        canvas.load_font("serif", include_bytes!("../fonts/Lato/Lato-Regular.ttf"))?;

        let mut rs = RenderState::default();
        rs.style.add_string("value.number.font", "mono");
        rs.style.add_number("value.number.font_size", 48.0);
        rs.style
            .add_color("value.number.color", color("#bccfa980")?);

        let mut v = render_value(
            &Value::Number(42.0),
            "",
            "",
            &IndexLocation::new(0, 0),
            &mut rs,
            &canvas,
        )?;
        v.translate(point(400.0, 400.0));
        v.draw(&mut canvas)?;
        v.translate(point(10.0, 5.0));
        v.draw(&mut canvas)?;
        v.translate(point(10.0, 5.0));
        v.draw(&mut canvas)?;

        rs.style
            .add_color("value.number.color", color("#cfa9bc80")?);
        let mut v2 = render_value(
            &Value::Number(67.0),
            "",
            "",
            &IndexLocation::new(0, 0),
            &mut rs,
            &canvas,
        )?;
        v2.translate(point(400.0, 430.0));
        v2.draw(&mut canvas)?;
        v2.translate(point(10.0, -7.0));
        v2.draw(&mut canvas)?;
        v2.translate(point(10.0, -7.0));
        v2.draw(&mut canvas)?;

        canvas.save("test_render_alpha.png")?;

        Ok(())
    }

    fn demo_prg() -> Program {
        Program(vec![
            Step {
                label: "L0".to_string(),
                locations: vec![
                    Location {
                        name: "Stack".to_string(),
                        definitions: vec![],
                        regions: vec![
                            Region {
                                name: "main".to_string(),
                                definitions: vec![
                                    Def {
                                        label: "x".to_string(),
                                        value: Value::Struct(NamedStruct {
                                            name: "Rect".to_string(),
                                            fields: vec![
                                                ("pos".to_string(), Value::Number(42.0)),
                                                ("w".to_string(), Value::Number(3.0)),
                                            ],
                                        }),
                                    },
                                    Def {
                                        label: "y2".to_string(),
                                        value: Value::Array(vec![
                                            Value::Number(42.0),
                                            Value::Invalid,
                                        ]),
                                    },
                                    Def {
                                        label: "H0".to_string(),
                                        value: Value::Invalid,
                                    },
                                ],
                            },
                            Region {
                                name: "main::f".to_string(),
                                definitions: vec![
                                    Def {
                                        label: "xs".to_string(),
                                        value: Value::Number(42.0),
                                    },
                                    Def {
                                        label: "y".to_string(),
                                        value: Value::Number(2.0),
                                    },
                                ],
                            },
                        ],
                    },
                    Location {
                        name: "Heap".to_string(),
                        definitions: vec![
                            Def {
                                label: "H0".to_string(),
                                value: Value::Number(42.0),
                            },
                            Def {
                                label: "y".to_string(),
                                value: Value::Number(2.0),
                            },
                        ],
                        regions: vec![],
                    },
                ],
            },
            Step {
                label: "L1".to_string(),
                locations: vec![
                    Location {
                        name: "Stack".to_string(),
                        definitions: vec![],
                        regions: vec![Region {
                            name: "main".to_string(),
                            definitions: vec![
                                Def {
                                    label: "x".to_string(),
                                    value: Value::Struct(NamedStruct {
                                        name: "Rect".to_string(),
                                        fields: vec![
                                            ("pos".to_string(), Value::Number(42.0)),
                                            ("w".to_string(), Value::Number(3.0)),
                                        ],
                                    }),
                                },
                                Def {
                                    label: "y2".to_string(),
                                    value: Value::Array(vec![Value::Number(42.0), Value::Invalid]),
                                },
                                Def {
                                    label: "H0".to_string(),
                                    value: Value::Pointer(Ptr {
                                        name: "x".to_string(),
                                        selectors: vec![],
                                        borrow: 0,
                                        help: vec![],
                                    }),
                                },
                            ],
                        }],
                    },
                    Location {
                        name: "Heap".to_string(),
                        definitions: vec![],
                        regions: vec![],
                    },
                ],
            },
        ])
    }

    #[test]
    pub fn test_render_value() -> Result<()> {
        let mut canvas = Canvas::new(2048, 2048, 1.0)?;
        canvas
            .pixmap
            .fill(Color::from_rgba8(0x19, 0x19, 0x19, 0xff));
        canvas.load_font(
            "mono",
            include_bytes!("../fonts/DejaVu/DejaVuSansMono-Bold.ttf"),
        )?;
        canvas.load_font("serif", include_bytes!("../fonts/Lato/Lato-Regular.ttf"))?;
        canvas.load_font("serif_bold", include_bytes!("../fonts/Lato/Lato-Bold.ttf"))?;

        let mut rs = RenderState::default();
        rs.style = dark_style()?;

        let mut v = render_value(
            &Value::Number(42.0),
            "",
            "",
            &IndexLocation::new(0, 0),
            &mut rs,
            &canvas,
        )?;
        v.translate(point(200.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_value(
            &Value::Char('H'),
            "",
            "",
            &IndexLocation::new(0, 0),
            &mut rs,
            &canvas,
        )?;
        v.translate(point(250.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_value(
            &Value::Pointer(Ptr {
                name: "".to_string(),
                selectors: vec![],
                borrow: 0,
                help: vec![],
            }),
            "",
            "",
            &IndexLocation::new(0, 0),
            &mut rs,
            &canvas,
        )?;
        v.translate(point(300.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_value(
            &Value::Array(vec![
                Value::Number(42.0),
                Value::Number(67.0),
                Value::Tuple(vec![]),
                Value::Tuple(vec![
                    Value::Char('C'),
                    Value::Number(4.0),
                    Value::Array(vec![]),
                ]),
            ]),
            "",
            "",
            &IndexLocation::new(0, 0),
            &mut rs,
            &canvas,
        )?;
        v.translate(point(350.0, 200.0));
        v.draw(&mut canvas)?;

        let mut v = render_def(
            &Def {
                label: "a".to_string(),
                value: Value::Array(vec![Value::Number(42.0), Value::Invalid]),
            },
            "",
            "",
            &IndexLocation::new(0, 0),
            &mut rs,
            &canvas,
            true,
        )?;
        v.translate(point(200.0, 300.0));
        v.draw(&mut canvas)?;

        let mut v = render_def(
            &Def {
                label: "x".to_string(),
                value: Value::Struct(NamedStruct {
                    name: "Rect".to_string(),
                    fields: vec![
                        ("pos".to_string(), Value::Number(42.0)),
                        ("w".to_string(), Value::Number(3.0)),
                    ],
                }),
            },
            "",
            "",
            &IndexLocation::new(0, 0),
            &mut rs,
            &canvas,
            true,
        )?;
        v.translate(point(200.0, 380.0));
        v.draw(&mut canvas)?;

        let style = dark_style()?;
        let mut v = render_program(&demo_prg(), &canvas, &style)?;
        v.translate(point(600.0, 500.0));
        v.draw(&mut canvas)?;

        // // Show some rects, draw them
        // for id in rs.ids() {
        //     let r = v.get_tagged(&id).unwrap().bounding_box(&canvas)?;
        //     println!("{} => {:?}", id, r);
        //     let bx = GBox::new_with_options(r, 1.0, color("#f00")?);
        //     bx.draw(&mut canvas)?;
        // }
        // let vv = v.get_tagged("L1:Stack:y2").unwrap();
        // println!("vv = {:?}", &vv);
        // println!("vv.bb = {:?}", vv.bounding_box(&canvas)?);

        canvas.save("test_render_value.png")?;

        Ok(())
    }

    #[test]
    pub fn test_draw_program() -> Result<()> {
        let canvas = draw_program(&demo_prg(), 1.0, "dark")?;
        canvas.save("test_draw_program.png")?;
        Ok(())
    }

    #[test]
    pub fn test_draw_program_png_data() -> Result<()> {
        let data = draw_program_png(&demo_prg(), 1.0, "dark")?;
        // just check for PNG magic at start
        assert_eq!(data[0..4], [0x89, 0x50, 0x4e, 0x47]);
        Ok(())
    }
}
