use ratatui::prelude::Rect;

/// Computes a rectangle with a desired width and height that is centered within a container rectangle.
/// The rectangle is constrained to fit within its container and (optionally) be inset by a given margin.
#[must_use]
pub fn centered_rect(
    container: Rect,
    target_width: u16,
    target_height: u16,
    min_margin: Option<u16>,
) -> Rect {
    let min_margin = min_margin.unwrap_or(0);

    let (x, width) = if target_width > container.width - min_margin {
        (container.x + min_margin, container.width - 2 * min_margin)
    } else {
        (
            container.x + (container.width - target_width) / 2,
            target_width,
        )
    };

    let (y, height) = if target_height > container.height - min_margin {
        (container.y + min_margin, container.height - 2 * min_margin)
    } else {
        (
            container.y + (container.height - target_height) / 2,
            target_height,
        )
    };

    Rect {
        x,
        y,
        width,
        height,
    }
}

// TODO: tests
