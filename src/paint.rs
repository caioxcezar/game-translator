use anyhow::Result;
use gtk::cairo::Context;
use pango::{Alignment, Layout, WrapMode};
use pangocairo::functions::create_layout;

use crate::area_object::AreaData;

pub fn draw_fitted_text_with_background(cr: &Context, area: &AreaData) -> Result<()> {
    if area.text.trim().is_empty() {
        cr.save()?;
        return Ok(());
    }

    let rect = gtk::gdk::Rectangle::new(area.x, area.y, area.width, area.height);
    let font_family = "Sans";

    draw_rectangle(cr, &rect)?;
    draw_text(cr, &area.text, &rect, Some(font_family))?;

    Ok(())
}

pub fn draw_text(
    cr: &Context,
    text: &str,
    rect: &gtk::gdk::Rectangle,
    font_family: Option<&str>,
) -> Result<()> {
    let layout = create_layout(cr);
    let mut font_desc = font_family.map_or_else(
        pango::FontDescription::new,
        pango::FontDescription::from_string,
    );
    font_desc.set_weight(pango::Weight::Bold);
    layout.set_font_description(Some(&font_desc));
    layout.set_text(text);
    layout.set_alignment(Alignment::Center);

    let (single_line_size, _) = text_size(&layout, rect, false)?;

    // Second try: Multi-line if single line is too small and would benefit from wrapping
    let (final_size, should_wrap) = if single_line_size < rect.height() as f64 / 3.0 {
        text_size(&layout, rect, true)?
    } else {
        (single_line_size, false)
    };

    // Configure final layout
    font_desc.set_size((final_size * pango::SCALE as f64) as i32);
    layout.set_font_description(Some(&font_desc));
    layout.set_wrap(if should_wrap {
        WrapMode::Word
    } else {
        WrapMode::Char
    });
    layout.set_width(rect.width() * pango::SCALE);

    // Calculate position
    let (text_width, text_height) = layout.pixel_size();
    let x = rect.x() as f64 + (rect.width() as f64 - text_width as f64) / 2.0;
    let y = rect.y() as f64 + (rect.height() as f64 - text_height as f64) / 2.0;

    cr.save()?;

    draw_text_with_outline(cr, &layout, x, y)?;

    Ok(())
}

pub fn draw_rectangle(cr: &Context, rect: &gtk::gdk::Rectangle) -> Result<()> {
    cr.save()?;
    cr.rectangle(
        rect.x() as f64,
        rect.y() as f64,
        rect.width() as f64,
        rect.height() as f64,
    );
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.75);
    cr.fill()?;
    cr.restore()?;

    Ok(())
}

fn text_size(layout: &Layout, rect: &gtk::gdk::Rectangle, allow_wrap: bool) -> Result<(f64, bool)> {
    let mut min_size = 1.0;
    let mut max_size = 1000.0;
    let mut optimal_size = min_size;
    let tolerance = 0.5;
    let mut needs_wrap = false;

    layout.set_wrap(if allow_wrap {
        WrapMode::Word
    } else {
        WrapMode::Char
    });
    layout.set_width(if allow_wrap {
        rect.width() * pango::SCALE
    } else {
        -1
    });

    while (max_size - min_size) > tolerance {
        let mid_size = (min_size + max_size) / 2.0;

        if let Some(mut font_desc) = layout.font_description() {
            font_desc.set_size((mid_size * pango::SCALE as f64) as i32);
            layout.set_font_description(Some(&font_desc));
        }

        let (width, height) = layout.pixel_size();
        let fits_width = width <= rect.width();
        let fits_height = height <= rect.height();

        if fits_width && fits_height {
            optimal_size = mid_size;
            min_size = mid_size;
            needs_wrap = allow_wrap && height < rect.height() * 3 / 4;
        } else {
            max_size = mid_size;
        }
    }

    Ok((optimal_size, needs_wrap))
}

fn draw_text_with_outline(cr: &Context, layout: &Layout, x: f64, y: f64) -> Result<()> {
    // Draw thicker black outline (8 directions)
    for dx in [x - 1.5, x + 1.5] {
        for dy in [y - 1.5, y + 1.5] {
            cr.move_to(dx, dy);
            pangocairo::functions::show_layout(cr, layout);
        }
    }

    // Draw white text
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.move_to(x, y);
    pangocairo::functions::show_layout(cr, layout);

    Ok(cr.restore()?)
}
