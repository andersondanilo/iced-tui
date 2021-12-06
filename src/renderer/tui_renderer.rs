use super::primitives::{Cell, Primitive, Style, VirtualBuffer};
use crossterm::{cursor, execute, queue, terminal};
use iced_native::{Color, Renderer};
use std::io::Write;

pub struct TuiRenderer {}

impl TuiRenderer {
    pub fn begin_screen(&mut self, stdout: &mut std::io::Stdout) {
        terminal::enable_raw_mode().unwrap();
        execute!(stdout, terminal::EnterAlternateScreen).unwrap();
    }

    pub fn end_screen(&mut self, stdout: &mut std::io::Stdout) {
        execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();
        execute!(stdout, crossterm::style::ResetColor, cursor::Show);
    }

    fn make_vbuffer(&self, primitive: Primitive) -> VirtualBuffer {
        let (width, height) = terminal::size().unwrap();
        let mut vbuffer = VirtualBuffer::from_size(width, height);
        vbuffer.merge_primitive(primitive);
        vbuffer.calc_hash();
        vbuffer
    }

    pub fn render(
        &mut self,
        stdout: &mut std::io::Stdout,
        primitive: Primitive,
        last_vbuffer: &Option<VirtualBuffer>,
    ) -> Option<VirtualBuffer> {
        let vbuffer = self.make_vbuffer(primitive);

        if let Some(last_vbuffer) = last_vbuffer {
            if vbuffer.hash == last_vbuffer.hash {
                return None;
            }
        }

        queue!(stdout, crossterm::style::ResetColor, cursor::Hide,).unwrap();

        let splited_rows = vbuffer.rows.iter().map(|row| split_by_style(row));

        for (i, results_by_style) in splited_rows.enumerate() {
            queue!(
                stdout,
                cursor::MoveTo(0, i as u16),
                //terminal::Clear(terminal::ClearType::CurrentLine),
            );

            for (style, content) in results_by_style {
                let mut fg_changed = false;
                let mut bg_changed = false;
                let mut attribute_changed = false;

                if let Some(fg_color) = style.fg_color {
                    queue!(
                        stdout,
                        crossterm::style::SetForegroundColor(to_term_color(fg_color))
                    );
                    fg_changed = true;
                }

                if let Some(bg_color) = style.bg_color {
                    queue!(
                        stdout,
                        crossterm::style::SetBackgroundColor(to_term_color(bg_color))
                    );
                    bg_changed = true;
                }

                if style.is_bold {
                    queue!(
                        stdout,
                        crossterm::style::SetAttribute(crossterm::style::Attribute::Bold)
                    );
                    attribute_changed = true;
                }

                queue!(stdout, crossterm::style::Print(content));

                if fg_changed || bg_changed {
                    queue!(stdout, crossterm::style::ResetColor);
                }

                if attribute_changed {
                    queue!(
                        stdout,
                        crossterm::style::SetAttribute(crossterm::style::Attribute::Reset),
                    );
                }
            }
        }

        queue!(stdout, cursor::MoveTo(0, 0));

        stdout.flush().unwrap();

        Some(vbuffer)
    }
}

fn split_by_style(cells: &Vec<Cell>) -> Vec<(Style, String)> {
    let mut last_style = Style::default();
    let mut results = vec![];
    let mut last_string = "".to_string();

    for cell in cells {
        if last_string.len() > 0 && last_style != cell.style {
            results.push((last_style, last_string));

            last_string = "".to_string()
        }

        last_string.push(match cell.content {
            Some(c) => c,
            None => ' ',
        });

        last_style = cell.style
    }

    if last_string.len() > 0 {
        results.push((last_style, last_string));
    }

    results
}

fn to_term_color(color: Color) -> crossterm::style::Color {
    crossterm::style::Color::Rgb {
        r: to_term_color_channel(color.r),
        g: to_term_color_channel(color.g),
        b: to_term_color_channel(color.b),
    }
}

fn to_term_color_channel(color_channel: f32) -> u8 {
    (255.0 * color_channel).round() as u8
}

impl Renderer for TuiRenderer {
    type Output = Primitive;
    type Defaults = Style;

    fn overlay(
        &mut self,
        base: <Self as iced_native::Renderer>::Output,
        _overlay: <Self as iced_native::Renderer>::Output,
        _overlay_bounds: iced_native::Rectangle,
    ) -> <Self as iced_native::Renderer>::Output {
        return base;
    }
}

impl Default for TuiRenderer {
    fn default() -> Self {
        Self {}
    }
}
