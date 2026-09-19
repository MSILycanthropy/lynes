use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, poll, read};

use crate::{
    input::ButtonState,
    living_room::{LivingRoom, PRESENT_INTERVAL, WAKE_INTERVAL},
    tv::ratatui::{RatatuiResult, RatatuiTV},
};

const KEY_PULSE_DURATION: Duration = Duration::from_millis(150);

impl LivingRoom<RatatuiTV> {
    pub fn run(mut self) -> RatatuiResult<()> {
        let tv = RatatuiTV::new()?;
        let reports_key_releases = tv.reports_key_releases();
        self.set_tv(tv);
        let mut release_deadlines = HashMap::<KeyCode, Instant>::new();

        self.last_tick = Some(Instant::now());
        self.cycle_fraction = 0.0;
        self.next_present_time = None;
        self.frame_pending = true;

        loop {
            let tick_time = Instant::now();

            self.advance_to(tick_time);

            release_deadlines.retain(|key, deadline| {
                if tick_time >= *deadline {
                    self.update_terminal_key(*key, false);
                    false
                } else {
                    true
                }
            });

            let now = Instant::now();
            let deadline = self.next_present_time.unwrap_or(now);

            if self.frame_pending && now >= deadline {
                self.present()?;
                self.next_present_time = Some(now + PRESENT_INTERVAL);
                self.frame_pending = false;
            }

            let mut wake_at = tick_time + WAKE_INTERVAL;

            if self.frame_pending {
                if let Some(deadline) = self.next_present_time {
                    wake_at = wake_at.min(deadline);
                }
            }

            let timeout = wake_at.saturating_duration_since(Instant::now());

            if !poll(timeout)? {
                continue;
            }

            use KeyCode::*;
            match read()? {
                Event::Key(key) => {
                    let code = match key.code {
                        Char(character) => Char(character.to_ascii_lowercase()),
                        code => code,
                    };
                    let pressed = key.kind != KeyEventKind::Release;

                    if pressed
                        && (code == Esc
                            || (code == Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)))
                    {
                        return Ok(());
                    }

                    if self.update_terminal_key(code, pressed) && !reports_key_releases {
                        if pressed {
                            release_deadlines.insert(code, Instant::now() + KEY_PULSE_DURATION);
                        } else {
                            release_deadlines.remove(&code);
                        }
                    }
                }
                Event::FocusLost => {
                    self.update_buttons(ButtonState::clear);
                    release_deadlines.clear();
                }
                Event::Resize(_, _) => {
                    self.frame_pending = true;
                }
                _ => {}
            }
        }
    }

    fn update_terminal_key(&mut self, key: KeyCode, pressed: bool) -> bool {
        use KeyCode::*;

        let update: fn(&mut ButtonState, bool) = match key {
            Char('w') => ButtonState::set_up,
            Char('a') => ButtonState::set_left,
            Char('s') => ButtonState::set_down,
            Char('d') => ButtonState::set_right,
            Char('z') => ButtonState::set_a,
            Char('x') => ButtonState::set_b,
            Enter => ButtonState::set_start,
            Tab => ButtonState::set_select,
            _ => return false,
        };

        self.update_buttons(|buttons| update(buttons, pressed));
        true
    }
}
