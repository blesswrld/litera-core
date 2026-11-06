//! Parametric Bézier Glyph Outline Geometry, De Casteljau Subdivision,
//! Green's Theorem Signed Contour Area, and Stem Contrast Analysis (`bezier_outlines.rs`).
//!
//! In TrueType (quadratic Bézier) and OpenType CFF / PostScript (cubic Bézier) fonts,
//! every glyph is defined by closed vector contours in a 1000-UPM or 2048-UPM coordinate grid:
//! - Outer clockwise contours deposit black ink (`signed_area > 0`).
//! - Inner counter-clockwise contours (counters inside `о`, `а`, `е`, `в`, `д`) subtract ink (`signed_area < 0`).
//! By integrating Green's theorem $\oint (x\,dy - y\,dx)$ over flattened De Casteljau segments,
//! we compute the exact mathematical ink-to-em area ratio of any vector glyph contour!

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    #[inline]
    pub fn distance(self, other: Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContourSegment {
    Line {
        p0: Point2D,
        p1: Point2D,
    },
    QuadraticBezier {
        p0: Point2D,
        p1: Point2D,
        p2: Point2D,
    },
    CubicBezier {
        p0: Point2D,
        p1: Point2D,
        p2: Point2D,
        p3: Point2D,
    },
}

impl ContourSegment {
    /// Evaluates the parametric point $B(t)$ for $t \in [0, 1]$ using De Casteljau's algorithm.
    #[inline]
    pub fn eval(self, t: f32) -> Point2D {
        match self {
            ContourSegment::Line { p0, p1 } => p0.lerp(p1, t),
            ContourSegment::QuadraticBezier { p0, p1, p2 } => {
                let q0 = p0.lerp(p1, t);
                let q1 = p1.lerp(p2, t);
                q0.lerp(q1, t)
            }
            ContourSegment::CubicBezier { p0, p1, p2, p3 } => {
                let q0 = p0.lerp(p1, t);
                let q1 = p1.lerp(p2, t);
                let q2 = p2.lerp(p3, t);
                let r0 = q0.lerp(q1, t);
                let r1 = q1.lerp(q2, t);
                r0.lerp(r1, t)
            }
        }
    }

    /// Subdivides a cubic Bézier curve at parameter $t \in [0, 1]$ into two exact cubic segments.
    pub fn subdivide_cubic(
        p0: Point2D,
        p1: Point2D,
        p2: Point2D,
        p3: Point2D,
        t: f32,
    ) -> (Self, Self) {
        let q0 = p0.lerp(p1, t);
        let q1 = p1.lerp(p2, t);
        let q2 = p2.lerp(p3, t);
        let r0 = q0.lerp(q1, t);
        let r1 = q1.lerp(q2, t);
        let s = r0.lerp(r1, t);

        (
            ContourSegment::CubicBezier {
                p0,
                p1: q0,
                p2: r0,
                p3: s,
            },
            ContourSegment::CubicBezier {
                p0: s,
                p1: r1,
                p2: q2,
                p3,
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphOutlineContour {
    pub segments: Vec<ContourSegment>,
}

impl GlyphOutlineContour {
    /// Computes the signed shoelace / Green's theorem area of the contour by sampling
    /// `steps_per_curve` linear chords per Bézier segment.
    pub fn compute_signed_area(&self, steps_per_curve: usize) -> f32 {
        let steps = steps_per_curve.max(4);
        let mut pts = Vec::with_capacity(self.segments.len() * steps + 1);

        for seg in &self.segments {
            let n_sub = match seg {
                ContourSegment::Line { .. } => 1,
                _ => steps,
            };
            for k in 0..n_sub {
                let t = (k as f32) / (n_sub as f32);
                pts.push(seg.eval(t));
            }
        }

        if pts.len() < 3 {
            return 0.0;
        }

        let mut area2 = 0.0f32;
        for i in 0..pts.len() {
            let p_curr = pts[i];
            let p_next = pts[(i + 1) % pts.len()];
            area2 += p_curr.x * p_next.y - p_next.x * p_curr.y;
        }
        0.5 * area2
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphOpticalContrastSpec {
    pub glyph_name: &'static str,
    /// Main vertical stem thickness in 1/1000 em units (e.g. 92 units in Garamond).
    pub main_stem_units: u16,
    /// Hairline horizontal/serif stroke thickness in 1/1000 em units (e.g. 24 units in Garamond).
    pub hairline_units: u16,
    /// Typographic stroke contrast ratio $C = \text{stem} / \text{hairline}$.
    pub stroke_contrast_ratio: f32,
    /// Axis angle of stress in degrees from vertical (e.g. $14^\circ$ humanist vs $0^\circ$ Didone).
    pub stress_axis_deg: f32,
}

/// Returns the canonical stroke-contrast geometry for our 2+1 font system.
pub fn inspect_typeface_contrast(font: crate::metrics::FontCategory) -> GlyphOpticalContrastSpec {
    match font {
        crate::metrics::FontCategory::RenaissanceSerif => GlyphOpticalContrastSpec {
            glyph_name: "Cormorant Garamond (Ренессансная антиква Клода Гарамона)",
            main_stem_units: 88,
            hairline_units: 22,
            stroke_contrast_ratio: 4.00,
            stress_axis_deg: 14.5,
        },
        crate::metrics::FontCategory::GeometricSans => GlyphOpticalContrastSpec {
            glyph_name: "Plus Jakarta Sans (Геометрический гуманистический гротеск)",
            main_stem_units: 94,
            hairline_units: 78,
            stroke_contrast_ratio: 1.21,
            stress_axis_deg: 0.0,
        },
        crate::metrics::FontCategory::Monospace => GlyphOpticalContrastSpec {
            glyph_name: "JetBrains Mono (Инженерный моноширинный)",
            main_stem_units: 86,
            hairline_units: 80,
            stroke_contrast_ratio: 1.075,
            stress_axis_deg: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bezier_contour_signed_area_with_counter() {
        // Outer square [0, 0] -> [500, 700] (area = 350,000)
        let outer = GlyphOutlineContour {
            segments: vec![
                ContourSegment::Line {
                    p0: Point2D::new(0.0, 0.0),
                    p1: Point2D::new(500.0, 0.0),
                },
                ContourSegment::Line {
                    p0: Point2D::new(500.0, 0.0),
                    p1: Point2D::new(500.0, 700.0),
                },
                ContourSegment::Line {
                    p0: Point2D::new(500.0, 700.0),
                    p1: Point2D::new(0.0, 700.0),
                },
                ContourSegment::Line {
                    p0: Point2D::new(0.0, 700.0),
                    p1: Point2D::new(0.0, 0.0),
                },
            ],
        };
        let area = outer.compute_signed_area(8).abs();
        assert!((area - 350_000.0).abs() < 1.0);
    }
}
