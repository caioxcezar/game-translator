use anyhow::Result;
use gtk::cairo::Context;
use pango::{Alignment, EllipsizeMode, FontDescription, Layout, WrapMode};
use pangocairo::functions::create_layout;

pub fn draw_fitted_text_with_background(
    cr: &Context,
    text: &str,
    rect: &gtk::gdk::Rectangle,
    font_family: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if text.trim().is_empty() {
        cr.save()?;
        return Ok(());
    }

    draw_reactangle(cr, rect)?;

    let layout = create_layout(cr);

    let mut font_desc = if let Some(family) = font_family {
        pango::FontDescription::from_string(family)
    } else {
        pango::FontDescription::new()
    };
    font_desc.set_weight(pango::Weight::Bold);
    layout.set_font_description(Some(&font_desc));

    layout.set_text(text);
    layout.set_alignment(Alignment::Center);
    layout.set_ellipsize(EllipsizeMode::None);
    layout.set_wrap(WrapMode::WordChar);

    let font_size = find_optimal_font_size(&mut font_desc, &layout, rect)?;

    font_desc.set_size(font_size * pango::SCALE);
    layout.set_font_description(Some(&font_desc));

    // Calculate text position (centered)
    let text_width = layout.pixel_size().0 as f64;
    let text_height = layout.pixel_size().1 as f64;
    let x = rect.x() as f64 + (rect.width() as f64 - text_width) / 2.0;
    let y = rect.y() as f64 + (rect.height() as f64 - text_height) / 2.0;

    // Draw the text with outline
    cr.save()?;

    draw_text_outline(cr, &layout, x, y);

    // Draw white text on top
    cr.set_source_rgb(1.0, 1.0, 1.0); // White
    cr.move_to(x, y);
    pangocairo::functions::show_layout(cr, &layout);

    cr.restore()?;

    Ok(())
}

fn draw_reactangle(cr: &Context, rect: &gtk::gdk::Rectangle) -> Result<()> {
    cr.save()?;
    cr.rectangle(
        rect.x() as f64,
        rect.y() as f64,
        rect.width() as f64,
        rect.height() as f64,
    );
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
    cr.fill()?;
    cr.restore()?;
    Ok(())
}

fn find_optimal_font_size(
    font_desc: &mut FontDescription,
    layout: &Layout,
    rect: &gtk::gdk::Rectangle,
) -> Result<i32> {
    let mut min_size = 1.0;
    let mut max_size = 1000.0;
    let mut optimal_size = min_size;
    let tolerance = 0.5;

    while (max_size - min_size) > tolerance {
        let mid_size = (min_size + max_size) / 2.0;

        // Update font size
        font_desc.set_size(mid_size as i32 * pango::SCALE);
        layout.set_font_description(Some(font_desc));

        // Get the pixel extents
        let (width, height) = layout.pixel_size();

        if width <= rect.width() && height <= rect.height() {
            // Text fits, try larger size
            optimal_size = mid_size;
            min_size = mid_size;
        } else {
            // Text doesn't fit, try smaller size
            max_size = mid_size;
        }
    }
    Ok(optimal_size as i32)
}

fn draw_text_outline(cr: &Context, layout: &Layout, x: f64, y: f64) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    for dx in [x - 1.5, x + 1.5] {
        for dy in [y - 1.5, y + 1.5] {
            cr.move_to(dx, dy);
            pangocairo::functions::show_layout(cr, layout);
        }
    }
}
