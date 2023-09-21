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

    let (x, width) = if target_width > container.width - 2 * min_margin {
        (container.x + min_margin, container.width - 2 * min_margin)
    } else {
        (
            container.x + (container.width - target_width) / 2,
            target_width,
        )
    };

    let (y, height) = if target_height > container.height - 2 * min_margin {
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

#[test]
fn centered_rect_same_parity_width_container() {
    let container = Rect {
        width: 8,
        height: 1,
        ..Default::default()
    };
    let rect = centered_rect(container, 4, 1, None);
    assert_eq!(
        rect,
        Rect {
            x: 2,
            y: 0,
            width: 4,
            height: 1
        }
    );
}

#[test]
fn centered_rect_opposite_parity_width_container() {
    let container = Rect {
        width: 7,
        height: 1,
        ..Default::default()
    };
    let rect = centered_rect(container, 4, 1, None);
    assert_eq!(
        rect,
        Rect {
            x: 1,
            y: 0,
            width: 4,
            height: 1
        }
    );
}

#[test]
fn centered_rect_squashed_by_margins() {
    let container = Rect {
        width: 8,
        height: 7,
        ..Default::default()
    };
    let rect = centered_rect(container, 4, 1, Some(3));
    assert_eq!(
        rect,
        Rect {
            x: 3,
            y: 3,
            width: 2,
            height: 1
        }
    );
}
