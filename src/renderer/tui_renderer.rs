use super::colors::get_crossterm_color;
use super::primitives::{Cell, Primitive};
use super::style::{CursorShape, Style};
use super::virtual_buffer::VirtualBuffer;
use crossterm::{cursor, execute, queue, terminal};
use iced_native::Renderer;

pub struct TuiRenderer {}

pub struct RenderResult {
    primitive: Primitive,
    size: (u16, u16),
    vbuffer: VirtualBuffer,
}

impl TuiRenderer {
    pub fn begin_screen(&self, stdout: &mut std::io::Stdout) {
        terminal::enable_raw_mode().unwrap();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )
        .unwrap();
    }

    pub fn end_screen(&self, stdout: &mut std::io::Stdout) {
        execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();
        execute!(stdout, crossterm::style::ResetColor, cursor::Show).unwrap();
    }

    fn make_vbuffer(&self, primitive: &Primitive, width: u16, height: u16) -> VirtualBuffer {
        let mut vbuffer = VirtualBuffer::from_size(width, height);
        vbuffer.merge_primitive(&primitive);
        vbuffer
    }

    pub fn render<O>(
        &self,
        output: &mut O,
        primitive: Primitive,
        last_render: Option<RenderResult>,
    ) -> RenderResult
    where
        O: std::io::Write,
    {
        let size = terminal::size().unwrap();
        let mut last_vbuffer: Option<VirtualBuffer> = None;

        if let Some(last_render) = last_render {
            if last_render.primitive == primitive && last_render.size == size {
                return RenderResult {
                    vbuffer: last_render.vbuffer,
                    primitive,
                    size,
                };
            }

            last_vbuffer = Some(last_render.vbuffer);
        }

        let vbuffer = self.make_vbuffer(&primitive, size.0, size.1);

        RenderResult {
            vbuffer: self.render_vbuffer(output, vbuffer, last_vbuffer.as_ref()),
            primitive,
            size,
        }
    }

    fn get_diff_rows<'a>(
        &self,
        vbuffer: &'a VirtualBuffer,
        last_vbuffer: &VirtualBuffer,
    ) -> Vec<(usize, &'a Vec<Cell>)> {
        vbuffer
            .rows
            .iter()
            .enumerate()
            .filter_map(|(i, cells)| {
                let old_cells = last_vbuffer.rows.get(i);

                if let Some(old_cells) = old_cells {
                    if old_cells == cells {
                        return None;
                    }
                }

                Some((i, cells))
            })
            .collect()
    }

    pub fn render_vbuffer<O>(
        &self,
        output: &mut O,
        vbuffer: VirtualBuffer,
        last_vbuffer: Option<&VirtualBuffer>,
    ) -> VirtualBuffer
    where
        O: std::io::Write,
    {
        queue!(output, crossterm::style::ResetColor).unwrap();
        let mut rendered_anything = false;

        let diff_rows: Vec<(usize, &Vec<Cell>)> = if let Some(last_vbuffer) = last_vbuffer {
            self.get_diff_rows(&vbuffer, last_vbuffer)
        } else {
            vbuffer.rows.iter().enumerate().into_iter().collect()
        };

        let splited_rows = diff_rows.iter().map(|(i, row)| (i, split_by_style(row)));

        for (i, results_by_style) in splited_rows {
            queue!(output, cursor::MoveTo(0, *i as u16),).unwrap();

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
                rendered_anything = true;

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

        let is_same_cursor_position = if let Some(last_vbuffer) = last_vbuffer {
            last_vbuffer.cursor_position == vbuffer.cursor_position
        } else {
            false
        };

        if !is_same_cursor_position || rendered_anything {
            match vbuffer.cursor_position {
                Some((x, y, style)) => {
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

#[cfg(test)]
mod tests {
    extern crate test;

    use super::super::colors::TermColor;
    use super::super::primitives::{Cell, Primitive};
    use super::super::style::Style;
    use super::super::virtual_buffer::VirtualBuffer;
    use super::*;
    use test::Bencher;

    fn make_example_primitive() -> Primitive {
        let mut primitive_cells = vec![];
        for x in 0_u8..100_u8 {
            for y in 0_u8..25_u8 {
                for add_y in 0_u8..5_u8 {
                    primitive_cells.push(Primitive::Cell(
                        x as u16,
                        (y + add_y) as u16,
                        Cell {
                            content: Some('a'),
                            style: Style {
                                fg_color: Some(TermColor::Rgb(x, x + 10_u8, y + 5_u8)),
                                bg_color: Some(TermColor::Rgb(x, x + 8_u8, y + 7_u8)),
                                is_bold: x % 2 == 0,
                            },
                        },
                    ));
                }
            }
        }

        Primitive::Group(primitive_cells)
    }

    fn make_example_vbuffer() -> VirtualBuffer {
        let mut vbuffer = VirtualBuffer::from_size(100, 100);
        vbuffer.merge_primitive(&make_example_primitive());
        vbuffer
    }

    #[bench]
    fn bench_clone_vbuffer(b: &mut Bencher) {
        let vbuffer = make_example_vbuffer();
        b.iter(|| vbuffer.clone())
    }

    #[bench]
    fn bench_split_rows_by_style_one_line(b: &mut Bencher) {
        let vbuffer = make_example_vbuffer();
        b.iter(|| {
            let row = &vbuffer.rows[0].clone();
            split_by_style(&row)
        })
    }

    #[bench]
    fn bench_split_rows_by_style_multiple(b: &mut Bencher) {
        let vbuffer = make_example_vbuffer();
        b.iter(|| {
            let _lines: Vec<Vec<(Style, String)>> = vbuffer
                .clone()
                .rows
                .iter()
                .map(|row| split_by_style(row))
                .collect();
        })
    }

    #[bench]
    fn bench_render_vbuffer_first(b: &mut Bencher) {
        let vbuffer = make_example_vbuffer();

        b.iter(|| {
            let vbuffer = vbuffer.clone();
            let renderer = TuiRenderer::default();
            let mut output: Vec<u8> = vec![];
            renderer.render_vbuffer(&mut output, vbuffer, None);
        });
    }

    #[bench]
    fn bench_render_vbuffer_diff_one_line(b: &mut Bencher) {
        let last_vbuffer = make_example_vbuffer();
        let mut vbuffer = make_example_vbuffer();
        vbuffer.merge_primitive(&Primitive::Rectangle(3, 5, 10, 1, Cell::from_char('Z')));
        vbuffer.merge_primitive(&Primitive::Rectangle(40, 5, 10, 1, Cell::from_char('Z')));

        b.iter(|| {
            let vbuffer = vbuffer.clone();
            let last_vbuffer = last_vbuffer.clone();
            let renderer = TuiRenderer::default();
            let mut output: Vec<u8> = vec![];
            renderer.render_vbuffer(&mut output, vbuffer, Some(&last_vbuffer));
        });
    }

    //#[bench]
    //fn bench_render_first(b: &mut Bencher) {
    //    let primitive = make_example_primitive();

    //    b.iter(|| {
    //        let renderer = TuiRenderer::default();
    //        let mut output: Vec<u8> = vec![];
    //        renderer.render(&mut output, primitive, &None, false);
    //    });
    //}

    #[bench]
    fn bench_make_vbuffer(b: &mut Bencher) {
        let primitive = make_example_primitive();

        b.iter(|| {
            let renderer = TuiRenderer::default();
            renderer.make_vbuffer(&primitive, 100, 100);
        });
    }

    //#[bench]
    //fn bench_render_diff_one_line(b: &mut Bencher) {
    //    let renderer = TuiRenderer::default();
    //    let last_primitive = make_example_primitive();
    //    let mut last_vbuffer = renderer.make_vbuffer(&last_primitive, 100, 100);
    //    last_vbuffer.merge_primitive(&Primitive::Rectangle(3, 5, 10, 1, Cell::from_char('Z')));

    //    let primitive = make_example_primitive();

    //    b.iter(|| {
    //        let renderer = TuiRenderer::default();
    //        let mut output: Vec<u8> = vec![];
    //        renderer.render(&mut output, &primitive, &Some(last_vbuffer.clone()), false);
    //    });
    //}

    #[bench]
    fn bench_get_diff_rows(b: &mut Bencher) {
        let renderer = TuiRenderer::default();
        let last_primitive = make_example_primitive();
        let mut last_vbuffer = renderer.make_vbuffer(&last_primitive, 100, 100);
        last_vbuffer.merge_primitive(&Primitive::Rectangle(3, 5, 10, 1, Cell::from_char('Z')));

        last_vbuffer.merge_primitive(&Primitive::Rectangle(3, 6, 10, 1, Cell::from_char('Z')));
        last_vbuffer.merge_primitive(&Primitive::Rectangle(3, 8, 10, 1, Cell::from_char('Z')));

        let primitive = make_example_primitive();
        let vbuffer = renderer.make_vbuffer(&primitive, 100, 100);

        b.iter(|| {
            let _rows = renderer.get_diff_rows(&vbuffer, &last_vbuffer);
        });
    }

    #[bench]
    fn bench_primitive_partial_eq(b: &mut Bencher) {
        let primitive1 = make_example_primitive();
        let primitive2 = make_example_primitive();

        b.iter(|| {
            let _result = if primitive1 == primitive2 { 1 } else { 2 };
        });
    }
}
