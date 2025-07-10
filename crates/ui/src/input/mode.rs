use std::rc::Rc;
use std::{cell::RefCell, ops::Range};

use gpui::{App, DefiniteLength, SharedString};

use crate::{highlighter::SyntaxHighlighter, input::marker::Marker};

use super::text_wrapper::TextWrapper;

#[derive(Debug, Copy, Clone)]
pub struct TabSize {
    /// Default is 2
    pub tab_size: usize,
    /// Set true to use `\t` as tab indent, default is false
    pub hard_tabs: bool,
}

impl Default for TabSize {
    fn default() -> Self {
        Self {
            tab_size: 2,
            hard_tabs: false,
        }
    }
}

impl TabSize {
    pub(super) fn to_string(&self) -> SharedString {
        if self.hard_tabs {
            "\t".into()
        } else {
            " ".repeat(self.tab_size).into()
        }
    }
}

#[derive(Default, Clone)]
pub enum InputMode {
    #[default]
    SingleLine,
    MultiLine {
        tab: TabSize,
        rows: usize,
        height: Option<DefiniteLength>,
    },
    CodeEditor {
        tab: TabSize,
        rows: usize,
        height: Option<DefiniteLength>,
        /// Show line number
        line_number: bool,
        language: SharedString,
        highlighter: Rc<RefCell<Option<SyntaxHighlighter>>>,
        markers: Rc<Vec<Marker>>,
    },
    AutoGrow {
        rows: usize,
        min_rows: usize,
        max_rows: usize,
    },
}

impl InputMode {
    pub(super) fn is_code_editor(&self) -> bool {
        matches!(self, InputMode::CodeEditor { .. })
    }

    pub(super) fn set_rows(&mut self, new_rows: usize) {
        match self {
            InputMode::MultiLine { rows, .. } => {
                *rows = new_rows;
            }
            InputMode::CodeEditor { rows, .. } => {
                *rows = new_rows;
            }
            InputMode::AutoGrow {
                rows,
                min_rows,
                max_rows,
            } => {
                *rows = new_rows.clamp(*min_rows, *max_rows);
            }
            _ => {}
        }
    }

    pub(super) fn set_height(&mut self, new_height: Option<DefiniteLength>) {
        match self {
            InputMode::MultiLine { height, .. } => {
                *height = new_height;
            }
            InputMode::CodeEditor { height, .. } => {
                *height = new_height;
            }
            _ => {}
        }
    }

    pub(super) fn update_auto_grow(&mut self, text_wrapper: &TextWrapper) {
        let wrapped_lines = text_wrapper.wrapped_lines.len();
        self.set_rows(wrapped_lines);
    }

    /// At least 1 row be return.
    pub(super) fn rows(&self) -> usize {
        match self {
            InputMode::MultiLine { rows, .. } => *rows,
            InputMode::CodeEditor { rows, .. } => *rows,
            InputMode::AutoGrow { rows, .. } => *rows,
            _ => 1,
        }
        .max(1)
    }

    /// At least 1 row be return.
    #[allow(unused)]
    pub(super) fn min_rows(&self) -> usize {
        match self {
            InputMode::MultiLine { .. } | InputMode::CodeEditor { .. } => 1,
            InputMode::AutoGrow { min_rows, .. } => *min_rows,
            _ => 1,
        }
        .max(1)
    }

    #[allow(unused)]
    pub(super) fn max_rows(&self) -> usize {
        match self {
            InputMode::MultiLine { .. } | InputMode::CodeEditor { .. } => usize::MAX,
            InputMode::AutoGrow { max_rows, .. } => *max_rows,
            _ => 1,
        }
    }

    pub(super) fn height(&self) -> Option<DefiniteLength> {
        match self {
            InputMode::MultiLine { height, .. } => *height,
            InputMode::CodeEditor { height, .. } => *height,
            _ => None,
        }
    }

    /// Return false if the mode is not [`InputMode::CodeEditor`].
    #[allow(unused)]
    #[inline]
    pub(super) fn line_number(&self) -> bool {
        match self {
            InputMode::CodeEditor { line_number, .. } => *line_number,
            _ => false,
        }
    }

    #[inline]
    pub(super) fn tab_size(&self) -> Option<&TabSize> {
        match self {
            InputMode::MultiLine { tab, .. } => Some(tab),
            InputMode::CodeEditor { tab, .. } => Some(tab),
            _ => None,
        }
    }

    pub(super) fn update_highlighter(
        &mut self,
        selected_range: &Range<usize>,
        full_text: SharedString,
        new_text: &str,
        cx: &mut App,
    ) {
        match &self {
            InputMode::CodeEditor {
                language,
                highlighter,
                ..
            } => {
                let mut highlighter = highlighter.borrow_mut();
                if highlighter.is_none() {
                    let new_highlighter = SyntaxHighlighter::new(language, cx);
                    highlighter.replace(new_highlighter);
                }

                if let Some(highlighter) = highlighter.as_mut() {
                    highlighter.update(selected_range, full_text, new_text, cx);
                }
            }
            _ => {}
        }
    }

    pub(super) fn clear_markers(&mut self) {
        match self {
            InputMode::CodeEditor { markers, .. } => *markers = Rc::new(vec![]),
            _ => {}
        }
    }

    #[allow(unused)]
    pub(super) fn markers(&self) -> Option<&Rc<Vec<Marker>>> {
        match self {
            InputMode::CodeEditor { markers, .. } => Some(markers),
            _ => None,
        }
    }

    pub(super) fn set_markers(&mut self, new_markers: Vec<Marker>) {
        match self {
            InputMode::CodeEditor { markers, .. } => *markers = Rc::new(new_markers),
            _ => {}
        }
    }

    pub(super) fn marker_for_offset(&self, offset: usize) -> Option<&Marker> {
        let Some(markers) = self.markers() else {
            return None;
        };

        for marker in markers.iter() {
            if let Some(range) = marker.range.as_ref() {
                if range.contains(&offset) {
                    return Some(marker);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::TabSize;

    #[test]
    fn test_tab_size() {
        let tab = TabSize {
            tab_size: 2,
            hard_tabs: false,
        };
        assert_eq!(tab.to_string(), "  ");
        let tab = TabSize {
            tab_size: 4,
            hard_tabs: false,
        };
        assert_eq!(tab.to_string(), "    ");

        let tab = TabSize {
            tab_size: 2,
            hard_tabs: true,
        };
        assert_eq!(tab.to_string(), "\t");
        let tab = TabSize {
            tab_size: 4,
            hard_tabs: true,
        };
        assert_eq!(tab.to_string(), "\t");
    }
}
