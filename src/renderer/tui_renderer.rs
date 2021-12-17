use super::colors::get_crossterm_color;
use super::primitives::{Cell, CursorPosition, Primitive};
use super::style::{CursorShape, Style};
use super::virtual_buffer::VirtualBuffer;
use crossterm::{cursor, execute, queue, terminal};
use iced_native::Renderer;

pub struct TuiRenderer {}

impl TuiRenderer {
    pub fn begin_screen(&mut self, stdout: &mut std::io::Stdout) {
        terminal::enable_raw_mode().unwrap();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )
        .unwrap();
    }

    pub fn end_screen(&mut self, stdout: &mut std::io::Stdout) {
        execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();
        execute!(stdout, crossterm::style::ResetColor, cursor::Show).unwrap();
    }

    fn make_vbuffer(&self, primitive: Primitive) -> VirtualBuffer {
        let (width, height) = terminal::size().unwrap();
        let mut vbuffer = VirtualBuffer::from_size(width, height);
        vbuffer.merge_primitive(primitive);
        vbuffer
    }

    pub fn render(
        &mut self,
        stdout: &mut std::io::Stdout,
        primitive: Primitive,
        last_vbuffer: &Option<VirtualBuffer>,
    ) -> Option<VirtualBuffer> {
        let vbuffer = self.make_vbuffer(primitive);

        //if let Some(last_vbuffer) = last_vbuffer {
        //    if vbuffer.hash == last_vbuffer.hash {
        //        return None;
        //    }
        //}

        Some(self.render_vbuffer(vbuffer, stdout))
    }

    fn render_vbuffer<O>(&self, vbuffer: VirtualBuffer, output: &mut O) -> VirtualBuffer
    where
        O: std::io::Write,
    {
        queue!(output, crossterm::style::ResetColor, cursor::Hide,).unwrap();

        let splited_rows = vbuffer.rows.iter().map(|row| split_by_style(row));

        for (i, results_by_style) in splited_rows.enumerate() {
            queue!(
                output,
                cursor::MoveTo(0, i as u16),
                //terminal::Clear(terminal::ClearType::CurrentLine),
            )
            .unwrap();

            for (style, content) in results_by_style {
                let mut fg_changed = false;
                let mut bg_changed = false;
                let mut attribute_changed = false;

                if let Some(fg_color) = style.fg_color {
                    queue!(
                        output,
                        crossterm::style::SetForegroundColor(get_crossterm_color(fg_color))
                    )
                    .unwrap();
                    fg_changed = true;
                }

                if let Some(bg_color) = style.bg_color {
                    queue!(
                        output,
                        crossterm::style::SetBackgroundColor(get_crossterm_color(bg_color))
                    )
                    .unwrap();
                    bg_changed = true;
                }

                if style.is_bold {
                    queue!(
                        output,
                        crossterm::style::SetAttribute(crossterm::style::Attribute::Bold)
                    )
                    .unwrap();
                    attribute_changed = true;
                }

                queue!(output, crossterm::style::Print(content)).unwrap();

                if fg_changed || bg_changed {
                    queue!(output, crossterm::style::ResetColor).unwrap();
                }

                if attribute_changed {
                    queue!(
                        output,
                        crossterm::style::SetAttribute(crossterm::style::Attribute::Reset),
                    )
                    .unwrap();
                }
            }
        }

        match vbuffer.cursor_position {
            Some(CursorPosition { x, y, style }) => {
                queue!(
                    output,
                    cursor::Show,
                    cursor::MoveTo(x, y),
                    cursor::SetCursorShape(match style.shape {
                        CursorShape::Line => cursor::CursorShape::Line,
                        CursorShape::Block => cursor::CursorShape::Block,
                        CursorShape::UnderScore => cursor::CursorShape::UnderScore,
                    }),
                )
                .unwrap();

                if style.blinking {
                    queue!(output, cursor::EnableBlinking).unwrap();
                } else {
                    queue!(output, cursor::DisableBlinking).unwrap();
                }
            }
            None => {
                queue!(output, cursor::MoveTo(0, 0), cursor::Hide).unwrap();
            }
        }

        output.flush().unwrap();

        vbuffer
    }
}

fn split_by_style(cells: &[Cell]) -> Vec<(Style, String)> {
    let mut last_style = Style::default();
    let mut results = vec![];
    let mut last_string = "".to_string();

    for cell in cells {
        if !last_string.is_empty() && last_style != cell.style {
            results.push((last_style, last_string));

            last_string = "".to_string()
        }

        last_string.push(cell.content.unwrap_or(' '));

        last_style = cell.style
    }

    if !last_string.is_empty() {
        results.push((last_style, last_string));
    }

    results
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
        base
    }
}

impl Default for TuiRenderer {
    fn default() -> Self {
        Self {}
    }
}

// #[cfg(test)]
// mod tests {
//     extern crate test;
//
//     use super::super::primitives::{Cell, Primitive, Style, VirtualBuffer};
//     use super::TuiRenderer;
//     use iced_native::Color;
//     use test::{black_box, Bencher};
//
//     #[bench]
//     fn bench_render_vbuffer(b: &mut Bencher) {
//         let mut virtual_buffer = VirtualBuffer::from_size(100, 100);
//
//         for x in 0_u8..100_u8 {
//             for y in 0_u8..25_u8 {
//                 for add_y in 0_u8..5_u8 {
//                     virtual_buffer.merge_primitive(Primitive::Cell(
//                         x as u16,
//                         (y + add_y) as u16,
//                         Cell {
//                             content: Some('a'),
//                             style: Style {
//                                 fg_color: Some(Color::from_rgb8(x, x + 10_u8, y + 5_u8)),
//                                 bg_color: Some(Color::from_rgb8(x, x + 8_u8, y + 7_u8)),
//                                 is_bold: x % 2 == 0,
//                             },
//                         },
//                     ));
//                 }
//             }
//         }
//
//         b.iter(|| {
//             black_box({
//                 let virtual_buffer = virtual_buffer.clone();
//                 let renderer = TuiRenderer::default();
//                 let mut output: Vec<u8> = vec![];
//                 renderer.render_vbuffer(virtual_buffer, &mut output);
//             });
//         });
//     }
// }
