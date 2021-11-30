use std::fmt::Write;

use crate::{
    graphics::{Color, Modifier, Style},
    Document, Editor, Theme, View,
};

pub type GutterFn<'doc> = Box<dyn Fn(usize, bool, &mut String) -> Option<Style> + 'doc>;
pub type Gutter =
    for<'doc> fn(&'doc Editor, &'doc Document, &View, &Theme, bool, usize) -> GutterFn<'doc>;

pub fn diagnostic<'doc>(
    _editor: &'doc Editor,
    doc: &'doc Document,
    _view: &View,
    theme: &Theme,
    _is_focused: bool,
    _width: usize,
) -> GutterFn<'doc> {
    let warning = theme.get("warning");
    let error = theme.get("error");
    let info = theme.get("info");
    let hint = theme.get("hint");
    let diagnostics = doc.diagnostics();

    Box::new(move |line: usize, _selected: bool, out: &mut String| {
        use helix_core::diagnostic::Severity;
        if let Some(diagnostic) = diagnostics.iter().find(|d| d.line == line) {
            write!(out, "●").unwrap();
            return Some(match diagnostic.severity {
                Some(Severity::Error) => error,
                Some(Severity::Warning) | None => warning,
                Some(Severity::Info) => info,
                Some(Severity::Hint) => hint,
            });
        }
        None
    })
}

pub fn line_number<'doc>(
    editor: &'doc Editor,
    doc: &'doc Document,
    view: &View,
    theme: &Theme,
    is_focused: bool,
    width: usize,
) -> GutterFn<'doc> {
    let text = doc.text().slice(..);
    let last_line = view.last_line(doc);
    // Whether to draw the line number for the last line of the
    // document or not.  We only draw it if it's not an empty line.
    let draw_last = text.line_to_byte(last_line) < text.len_bytes();

    let linenr = theme.get("ui.linenr");
    let linenr_select: Style = theme.try_get("ui.linenr.selected").unwrap_or(linenr);

    let current_line = doc
        .text()
        .char_to_line(doc.selection(view.id).primary().cursor(text));

    let config = editor.config.line_number;

    Box::new(move |line: usize, selected: bool, out: &mut String| {
        if line == last_line && !draw_last {
            write!(out, "{:>1$}", '~', width).unwrap();
            Some(linenr)
        } else {
            use crate::editor::LineNumber;
            let line = match config {
                LineNumber::Absolute => line + 1,
                LineNumber::Relative => {
                    if current_line == line {
                        line + 1
                    } else {
                        abs_diff(current_line, line)
                    }
                }
            };
            let style = if selected && is_focused {
                linenr_select
            } else {
                linenr
            };
            write!(out, "{:>1$}", line, width).unwrap();
            Some(style)
        }
    })
}

#[inline(always)]
const fn abs_diff(a: usize, b: usize) -> usize {
    if a > b {
        a - b
    } else {
        b - a
    }
}

pub fn breakpoints<'doc>(
    editor: &'doc Editor,
    doc: &'doc Document,
    _view: &View,
    theme: &Theme,
    _is_focused: bool,
    _width: usize,
) -> GutterFn<'doc> {
    let warning = theme.get("warning");
    let error = theme.get("error");
    let info = theme.get("info");

    let breakpoints = doc
        .path()
        .and_then(|path| editor.breakpoints.get(path))
        .unwrap();

    Box::new(move |line: usize, _selected: bool, out: &mut String| {
        let breakpoint = breakpoints
            .iter()
            .find(|breakpoint| breakpoint.line == line);

        let breakpoint = match breakpoint {
            Some(b) => b,
            None => return None,
        };

        let mut style = if breakpoint.condition.is_some() && breakpoint.log_message.is_some() {
            error.add_modifier(Modifier::UNDERLINED)
        } else if breakpoint.condition.is_some() {
            error
        } else if breakpoint.log_message.is_some() {
            info
        } else {
            warning
        };

        if !breakpoint.verified {
            // Faded colors
            style = if let Some(Color::Rgb(r, g, b)) = style.fg {
                style.fg(Color::Rgb(
                    ((r as f32) * 0.4).floor() as u8,
                    ((g as f32) * 0.4).floor() as u8,
                    ((b as f32) * 0.4).floor() as u8,
                ))
            } else {
                style.fg(Color::Gray)
            }
        };

        // TODO: also handle breakpoints only present in the user struct
        let sym = if breakpoint.verified { "▲" } else { "⊚" };
        write!(out, "{}", sym).unwrap();
        Some(style)
    })
}