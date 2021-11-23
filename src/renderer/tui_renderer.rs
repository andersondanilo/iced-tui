use super::primitives::{Primitive, Style, VirtualBuffer};
use crossterm::{cursor, execute, queue, terminal};
use iced_native::Renderer;
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

        for (i, row) in vbuffer.rows.iter().enumerate() {
            let row_content = row
                .iter()
                .map(|cell| match cell.content {
                    Some(c) => c,
                    None => ' ',
                })
                .collect::<String>();

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

impl Default for TuiRenderer {
    fn default() -> Self {
        Self {}
    }
}
