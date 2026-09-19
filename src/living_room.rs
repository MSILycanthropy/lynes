use std::time::Instant;

use crate::{NES, input::ButtonState, tv::TV};

#[cfg(feature = "wgpu")]
mod wgpu;

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

    pub fn present(&mut self) -> Result<(), T::Error> {
        if let Some(tv) = self.tv.as_mut() {
            tv.present(self.nes.frame())?;
        }

        Ok(())
    }
}
