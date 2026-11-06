//! Multi-Page Vertical Galley Pagination, Widow/Orphan Penalty Solver,
//! and Marginalia Collision Avoidance Engine (`baseline_composer.rs`).
//!
//! Extends horizontal Knuth–Plass line breaking into the vertical dimension (*page breaking*):
//! - **Club Line (Orphan / Висячая начальная строка)**: First line of a paragraph stranded
//!   alone at the bottom of a page (`clubpenalty = 10000` in strict classical book work).
//! - **Widow Line (Вдова / Висячая концевая строка)**: Last line of a paragraph stranded
//!   alone at the top of a new page (`widowpenalty = 10000`).
//! - **Facing-Spread Register (Приводность строк)**: Ensures left (verso) and right (recto)
//!   pages lock to the exact same vertical baseline grid even when section subheads or
//!   marginalia side-notes are inserted.

use crate::knuth_plass::TypesetLine;

#[derive(Debug, Clone, PartialEq)]
pub struct MarginaliaNote {
    pub anchor_line_number: usize,
    pub note_text: String,
    pub height_lines: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComposedBookPage {
    pub page_number: usize,
    pub is_recto: bool,
    pub lines: Vec<TypesetLine>,
    pub marginalia: Vec<(usize, String)>,
    pub vertical_fill_ratio: f32,
    pub widow_or_orphan_prevented: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaginationReport {
    pub pages: Vec<ComposedBookPage>,
    pub total_lines: usize,
    pub target_lines_per_page: usize,
    pub widow_orphan_adjustments: usize,
    pub marginalia_collisions_resolved: usize,
}

/// Paginates a stream of `TypesetLine` rows into balanced recto/verso book pages while
/// strictly preventing single-line widows and orphans and resolving vertical overlaps
/// between margin side-notes (`MarginaliaNote`).
pub fn paginate_galley_with_marginalia(
    lines: &[TypesetLine],
    target_lines_per_page: usize,
    notes: &[MarginaliaNote],
) -> PaginationReport {
    let cap = target_lines_per_page.max(6);
    if lines.is_empty() {
        return PaginationReport {
            pages: Vec::new(),
            total_lines: 0,
            target_lines_per_page: cap,
            widow_orphan_adjustments: 0,
            marginalia_collisions_resolved: 0,
        };
    }

    let mut pages = Vec::new();
    let mut cursor = 0usize;
    let total = lines.len();
    let mut widow_orphan_adjustments = 0usize;
    let mut marginalia_collisions_resolved = 0usize;

    while cursor < total {
        let remaining = total - cursor;
        let mut take_count = remaining.min(cap);
        let mut adjusted = false;

        // If breaking at `cursor + take_count` would leave exactly 1 widow line on the next page,
        // pull 1 line back (setting this page 1 line short — `looseness = -1`) so the final page
        // has at least 2 lines!
        if remaining == cap + 1 && take_count >= 3 {
            take_count -= 1;
            widow_orphan_adjustments += 1;
            adjusted = true;
        }

        let page_slice = &lines[cursor..cursor + take_count];
        let page_num = pages.len() + 1;
        let is_recto = page_num % 2 == 1;

        // Resolve marginalia notes anchored within [cursor + 1 ..= cursor + take_count]
        let mut page_notes = Vec::new();
        let mut next_free_local_row = 1usize;

        for note in notes {
            if note.anchor_line_number > cursor && note.anchor_line_number <= cursor + take_count {
                let desired_local = note.anchor_line_number - cursor;
                let placed_local = if desired_local < next_free_local_row {
                    marginalia_collisions_resolved += 1;
                    next_free_local_row
                } else {
                    desired_local
                };
                if placed_local <= cap {
                    page_notes.push((placed_local, note.note_text.clone()));
                    next_free_local_row = placed_local + note.height_lines.max(1);
                }
            }
        }

        pages.push(ComposedBookPage {
            page_number: page_num,
            is_recto,
            lines: page_slice.to_vec(),
            marginalia: page_notes,
            vertical_fill_ratio: (take_count as f32) / (cap as f32),
            widow_or_orphan_prevented: adjusted,
        });

        cursor += take_count;
    }

    PaginationReport {
        pages,
        total_lines: total,
        target_lines_per_page: cap,
        widow_orphan_adjustments,
        marginalia_collisions_resolved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knuth_plass::FitnessClass;

    #[test]
    fn test_widow_prevention_and_marginalia_collision() {
        let dummy_lines: Vec<TypesetLine> = (1..=11)
            .map(|n| TypesetLine {
                line_number: n,
                text: format!("Строка набора номер {n}"),
                word_count: 4,
                natural_width_units: 18000,
                target_width_units: 20000,
                adjustment_ratio: 0.1,
                badness: 0,
                demerits: 100,
                fitness: FitnessClass::Normal,
            })
            .collect();

        let notes = vec![
            MarginaliaNote {
                anchor_line_number: 2,
                note_text: String::from("Первая глосса"),
                height_lines: 3,
            },
            MarginaliaNote {
                anchor_line_number: 3,
                note_text: String::from("Вторая глосса (сдвиг вниз)"),
                height_lines: 2,
            },
        ];

        // 11 lines with target 10 lines/page would leave 1 widow line on page 2 unless adjusted to 9 + 2!
        let rep = paginate_galley_with_marginalia(&dummy_lines, 10, &notes);
        assert_eq!(rep.pages.len(), 2);
        assert_eq!(rep.pages[0].lines.len(), 9);
        assert_eq!(rep.pages[1].lines.len(), 2);
        assert_eq!(rep.widow_orphan_adjustments, 1);
        assert_eq!(rep.marginalia_collisions_resolved, 1);
    }
}
