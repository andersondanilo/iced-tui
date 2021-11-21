use core::marker::PhantomData;
use crossterm::{cursor, execute, queue, terminal};
use iced_native::layout::Node;
use iced_native::Element;
use iced_native::Layout;
use iced_native::Point;
use iced_native::Rectangle;
use iced_native::Size;
use iced_native::Vector;
use iced_native::{column, container, row, text, Length, Renderer};
use std::io::Write;
use std::iter::zip;

pub struct TuiRenderer {}

pub enum Primitive {
    Cell(u16, u16, Cell),
    Group(Vec<Primitive>),
}

#[derive(Clone, Copy)]
pub struct TuiFont {}

impl Default for TuiFont {
    fn default() -> Self {
        Self {}
    }
}

impl Default for TuiRenderer {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct Cell {
    content: Option<char>,
    style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            content: Some(' '),
            style: Style::default(),
        }
    }
}

#[derive(Debug)]
pub struct Style {
    fg_color: Option<iced_native::Color>,
    bg_color: Option<iced_native::Color>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
        }
    }
}

#[derive(Debug)]
struct VirtualBuffer {
    width: u16,
    height: u16,
    rows: Vec<Vec<Cell>>,
}

impl VirtualBuffer {
    fn from_size(width: u16, height: u16) -> Self {
        let mut rows: Vec<Vec<Cell>> = Vec::with_capacity(height.into());
        for y in 0..height {
            let mut row = Vec::with_capacity(width.into());
            for x in 0..width {
                row.push(Cell::default());
            }
            rows.push(row);
        }

        Self {
            rows,
            width,
            height,
        }
    }

    fn merge_primitive(&mut self, primitive: Primitive) {
        match primitive {
            Primitive::Group(primitives) => {
                for primitive in primitives {
                    self.merge_primitive(primitive);
                }
            }
            Primitive::Cell(x, y, cell) => {
                if x < self.width && y < self.height {
                    self.rows[y as usize][x as usize] = cell;
                }
            }
        };
    }
}

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

    pub fn render(&mut self, stdout: &mut std::io::Stdout, primitive: Primitive) {
        let (width, height) = terminal::size().unwrap();

        eprintln!("rendering terminal height = {}", height);

        let mut vbuffer = VirtualBuffer::from_size(width, height);
        vbuffer.merge_primitive(primitive);

        queue!(
            stdout,
            crossterm::style::ResetColor,
            cursor::Hide,
            //terminal::Clear(terminal::ClearType::All),
            // cursor::MoveTo(0, 0),
        )
        .unwrap();

        let height_usize = vbuffer.height as usize;

        for (i, row) in vbuffer.rows.iter().enumerate() {
            let mut row_content = row
                .iter()
                .map(|cell| match cell.content {
                    Some(c) => c,
                    None => ' ',
                })
                .collect::<String>();
            // eprintln!("rendering row {}: {}", i, row_content);

            queue!(
                stdout,
                cursor::MoveTo(0, i as u16),
                terminal::Clear(terminal::ClearType::CurrentLine),
                crossterm::style::Print(row_content)
            );
        }

        queue!(stdout, cursor::MoveTo(0, 0));

        stdout.flush().unwrap();
    }
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

impl container::Renderer for TuiRenderer {
    type Style = Style;

    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        cursor_position: Point,
        viewport: &Rectangle,
        _style: &Self::Style,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
    ) -> <Self as iced_native::Renderer>::Output {
        let content = content.draw(self, defaults, content_layout, cursor_position, viewport);

        Primitive::Group(vec![content])
    }
}

impl column::Renderer for TuiRenderer {
    fn draw<Message>(
        &mut self,
        defaults: &<Self as iced_native::Renderer>::Defaults,
        contents: &[iced_native::Element<'_, Message, Self>],
        layout: iced_native::Layout<'_>,
        cursor_position: iced_native::Point,
        viewport: &iced_native::Rectangle,
    ) -> <Self as iced_native::Renderer>::Output {
        Primitive::Group(
            zip(layout.children(), contents)
                .map(|(layout, content)| {
                    content.draw(self, defaults, layout, cursor_position, viewport)
                })
                .collect::<Vec<Self::Output>>(),
        )
    }
}

impl row::Renderer for TuiRenderer {
    fn draw<Message>(
        &mut self,
        defaults: &<Self as iced_native::Renderer>::Defaults,
        contents: &[iced_native::Element<'_, Message, Self>],
        layout: iced_native::Layout<'_>,
        cursor_position: iced_native::Point,
        viewport: &iced_native::Rectangle,
    ) -> <Self as iced_native::Renderer>::Output {
        Primitive::Group(
            zip(layout.children(), contents)
                .map(|(layout, content)| {
                    content.draw(self, defaults, layout, cursor_position, viewport)
                })
                .collect::<Vec<Self::Output>>(),
        )
    }
}

impl text::Renderer for TuiRenderer {
    type Font = TuiFont;

    fn default_size(&self) -> u16 {
        1
    }

    fn measure(
        &self,
        content: &str,
        _size: u16,
        _font: <Self as iced_native::text::Renderer>::Font,
        bounds: iced_native::Size,
    ) -> (f32, f32) {
        let (_, width, height) = auto_wrap_text(content, bounds);
        (width as f32, height as f32)
    }

    fn draw(
        &mut self,
        _defaults: &<Self as iced_native::Renderer>::Defaults,
        bounds: iced_native::Rectangle,
        content: &str,
        _size: u16,
        _font: <Self as iced_native::text::Renderer>::Font,
        _color: std::option::Option<iced_native::Color>,
        _horizontal_alignment: iced_native::HorizontalAlignment,
        _vertical_alignment: iced_native::VerticalAlignment,
    ) -> <Self as iced_native::Renderer>::Output {
        let (primitive_cells, _width, _height) = auto_wrap_text(content, bounds.size());
        Primitive::Group(primitive_cells)
    }
}

fn auto_wrap_text<'a>(content: &str, bounds: iced_native::Size) -> (Vec<Primitive>, u16, u16) {
    let mut primitive_cells: Vec<Primitive> = Vec::with_capacity(content.len());
    let bounds_width_i = bounds.width as u16;
    let bounds_height_i = bounds.height as u16;
    let mut current_x: u16 = 0;
    let mut current_y: u16 = 0;
    let mut max_x: u16 = 0;

    // TODO: Handle non-printable chars and tab/space

    for c in content.chars() {
        let height = current_y + 1;
        if c == '\n' {
            if height < bounds_height_i {
                current_x = 0;
                current_y += 1;
            } else {
                break;
            }
        } else {
            if current_y == bounds_width_i {
                if height < bounds_height_i {
                    current_y += 1;
                    current_x = 0;

                    primitive_cells.push(Primitive::Cell(
                        current_x,
                        current_y,
                        Cell {
                            content: Some(c),
                            style: Style::default(),
                        },
                    ));
                } else {
                    break;
                }
            } else {
                primitive_cells.push(Primitive::Cell(
                    current_x,
                    current_y,
                    Cell {
                        content: Some(c),
                        style: Style::default(),
                    },
                ));

                current_x += 1;
            }
        }

        if current_x > max_x {
            max_x = current_x;
        }
    }

    let width = max_x + 1;
    let height = current_y + 1;

    (primitive_cells, width, height)
}
