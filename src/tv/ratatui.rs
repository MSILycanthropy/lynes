use std::io;

use crossterm::{
    event::{
        DisableFocusChange, EnableFocusChange, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::supports_keyboard_enhancement,
};
use image::{DynamicImage, RgbImage};
use ratatui::{
    DefaultTerminal,
    layout::{Rect, Size},
    style::{Color, Style},
    widgets::{Paragraph, Wrap},
};
use ratatui_image::{
    Image,
    errors::Errors as ImageProtocolError,
    protocol::{Protocol, halfblocks::Halfblocks},
};
use thiserror::Error;

use crate::{
    frame::{FRAME_HEIGHT, FRAME_WIDTH, Frame},
    tv::TV,
};

pub type RatatuiResult<T> = Result<T, RatatuiError>;

#[derive(Debug, Error)]
pub enum RatatuiError {
    #[error("terminal I/O failed: {0}")]
    TerminalIo(#[from] io::Error),

    #[error("terminal image protocol failed: {0}")]
    ImageProtocol(#[from] ImageProtocolError),
}

pub struct RatatuiTV {
    terminal: DefaultTerminal,
    keyboard_enhancement: bool,
}

impl RatatuiTV {
    pub fn new() -> RatatuiResult<Self> {
        let terminal = match ratatui::try_init() {
            Ok(terminal) => terminal,
            Err(error) => {
                ratatui::restore();
                return Err(error.into());
            }
        };

        let mut tv = Self {
            terminal,
            keyboard_enhancement: false,
        };

        tv.keyboard_enhancement = supports_keyboard_enhancement()?;
        if tv.keyboard_enhancement {
            execute!(
                tv.terminal.backend_mut(),
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
                ),
            )?;
        }
        execute!(tv.terminal.backend_mut(), EnableFocusChange)?;

        Ok(tv)
    }

    pub(crate) fn reports_key_releases(&self) -> bool {
        cfg!(windows) || self.keyboard_enhancement
    }
}

impl Drop for RatatuiTV {
    fn drop(&mut self) {
        if self.keyboard_enhancement {
            let _ = execute!(self.terminal.backend_mut(), PopKeyboardEnhancementFlags);
        }
        let _ = execute!(self.terminal.backend_mut(), DisableFocusChange);
        ratatui::restore();
    }
}

impl TV for RatatuiTV {
    type Error = RatatuiError;

    fn present(&mut self, frame: &Frame) -> RatatuiResult<()> {
        let available = self.terminal.size()?;

        if available.width == 0 || available.height == 0 {
            return Ok(());
        }

        let columns = FRAME_WIDTH as u16;
        let rows = (FRAME_HEIGHT / 2) as u16;
        if available.width < columns || available.height < rows {
            self.terminal.draw(|terminal_frame| {
                let area = terminal_frame.area();
                terminal_frame
                    .buffer_mut()
                    .set_style(area, Style::default().bg(Color::Black));

                let message = Paragraph::new(format!(
                    "Terminal too small\n\nLower your font size or enlarge the window.\n\nNeed: {columns} columns x {rows} rows\nCurrent: {} columns x {} rows",
                    area.width, area.height,
                ))
                .centered()
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true });
                let height = area.height.min(7);
                let message_area = Rect::new(
                    area.x,
                    area.y + area.height.saturating_sub(height) / 2,
                    area.width,
                    height,
                );
                terminal_frame.render_widget(message, message_area);
            })?;
            return Ok(());
        }

        let mut pixels = RgbImage::new(FRAME_WIDTH as u32, FRAME_HEIGHT as u32);
        pixels.as_mut().copy_from_slice(frame.data());

        let image = DynamicImage::ImageRgb8(pixels);
        let size = Size::new(columns, rows);
        let protocol = Protocol::Halfblocks(Halfblocks::new(image, size)?);

        self.terminal.draw(|terminal_frame| {
            let area = terminal_frame.area();
            terminal_frame
                .buffer_mut()
                .set_style(area, Style::default().bg(Color::Black));
            let size = protocol.size();

            let x = area.x + area.width.saturating_sub(size.width) / 2;
            let y = area.y + area.height.saturating_sub(size.height) / 2;

            let image_area = Rect::new(
                x,
                y,
                size.width.min(area.width),
                size.height.min(area.height),
            );

            terminal_frame.render_widget(Image::new(&protocol), image_area);
        })?;

        Ok(())
    }
}
