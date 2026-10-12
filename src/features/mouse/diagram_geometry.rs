pub(super) struct ClickGeometry {
    pub(super) left: f32,
    pub(super) right: f32,
    pub(super) radius: f32,
    pub(super) curve_y: f32,
    pub(super) cut_y: f32,
    pub(super) inner_left: f32,
    pub(super) inner_right: f32,
    pub(super) bottom: f32,
}

pub(super) fn click_geometry(width: f32, height: f32, gutter: f32) -> Option<ClickGeometry> {
    if !width.is_finite() || !height.is_finite() || !gutter.is_finite() || gutter < 0. {
        return None;
    }
    let center = width / 2.;
    let radius = center - 1.;
    let half_gap = gutter / 2.;
    if radius <= half_gap || height <= radius + 1. {
        return None;
    }
    let curve_y = radius + 1.;
    Some(ClickGeometry {
        left: 1.,
        right: width - 1.,
        radius,
        curve_y,
        cut_y: curve_y - radius * (1. - (half_gap / radius).powi(2)).max(0.).sqrt(),
        inner_left: center - half_gap,
        inner_right: center + half_gap,
        bottom: height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_edges_follow_the_mouse_circle_at_several_scales() {
        for rem in [12., 16., 20., 28.] {
            let width = 8.5 * rem;
            let cap = click_geometry(width, 6. * rem, 1.75 * rem).unwrap();
            let center = width / 2.;
            for x in [cap.inner_left, cap.inner_right] {
                let distance = ((x - center).powi(2) + (cap.cut_y - cap.curve_y).powi(2)).sqrt();
                assert!((distance - cap.radius).abs() < 0.0001);
            }
            assert!(cap.cut_y >= 1.);
            assert!(cap.bottom > cap.curve_y);
            assert_eq!(cap.left, 1.);
            assert_eq!(cap.right, width - 1.);
        }
    }

    #[test]
    fn invalid_geometry_is_not_painted() {
        for (width, height, gutter) in [
            (0., 10., 2.),
            (10., 10., 12.),
            (100., 10., 20.),
            (100., 90., -1.),
            (f32::NAN, 90., 20.),
            (100., f32::INFINITY, 20.),
        ] {
            assert!(click_geometry(width, height, gutter).is_none());
        }
    }
}
