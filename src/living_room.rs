use std::time::{Duration, Instant};

use crate::{NES, input::ButtonState, tv::TV};

pub mod ratatui;

mod wgpu;

const WAKE_INTERVAL: Duration = Duration::from_millis(4);
const MAX_FRAMERATE: u64 = 60;
const PRESENT_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / MAX_FRAMERATE);
pub const CPU_HZ: f64 = 1_789_773.0;

/// This is effectivly an Application. A given frontend will do it's thing against this
pub struct LivingRoom<T: TV> {
    nes: NES,
    tv: Option<T>,
    cycles_ahead: usize,
    last_tick: Option<Instant>,
    cycle_fraction: f64,

    next_present_time: Option<Instant>,
    frame_pending: bool,
}

impl<T: TV> LivingRoom<T> {
    pub fn new(nes: NES) -> Self {
        Self {
            nes,
            tv: None,
            cycles_ahead: 0,
            last_tick: None,
            cycle_fraction: 0.0,

            next_present_time: None,
            frame_pending: false,
        }
    }

    pub fn set_tv(&mut self, tv: T) {
        self.tv = Some(tv);
    }

    pub fn update_buttons(&mut self, update: impl FnOnce(&mut ButtonState)) {
        self.nes.update_buttons(update);
    }

    /// Advances by the requested CPU cycle budget.
    /// Returns whether a frame completed during this batch.
    pub fn advance(&mut self, cpu_cycles: usize) -> bool {
        if self.cycles_ahead >= cpu_cycles {
            self.cycles_ahead -= cpu_cycles;

            return false;
        }

        let budget = cpu_cycles - self.cycles_ahead;
        let mut elapsed = 0;
        let mut frame_ready = false;

        while elapsed < budget {
            let result = self.nes.step();

            elapsed += result.cpu_cycles;
            frame_ready |= result.frame_ready;
        }

        self.cycles_ahead = elapsed - budget;
        frame_ready
    }

    fn advance_to(&mut self, now: Instant) {
        let Some(previous) = self.last_tick.replace(now) else {
            return;
        };

        let elapsed = now.duration_since(previous);
        let cycles = elapsed.as_secs_f64() * CPU_HZ + self.cycle_fraction;
        let budget = cycles.floor() as usize;

        self.cycle_fraction = cycles - budget as f64;

        if self.advance(budget) {
            self.frame_pending = true;
        }
    }

    pub fn present(&mut self) -> Result<(), T::Error> {
        if let Some(tv) = self.tv.as_mut() {
            tv.present(self.nes.frame())?;
        }

        Ok(())
    }
}
