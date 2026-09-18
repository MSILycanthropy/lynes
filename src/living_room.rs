use crate::{NES, input::ButtonState, tv::TV};

/// This is effectivly an Application. A given frontend will do it's thing against this
/// Let's us easily wrap the NES and TV into one thing while managing _cycles_
/// The actual frontend manages time.
struct LivingRoom<T: TV> {
    nes: NES,
    tv: Option<T>,
    cycles_ahead: usize,
}

impl<T: TV> LivingRoom<T> {
    pub fn new(nes: NES) -> Self {
        Self {
            nes,
            tv: None,
            cycles_ahead: 0,
        }
    }

    pub fn set_tv(&mut self, tv: T) {
        self.tv = Some(tv);
    }

    pub fn set_buttons(&mut self, buttons: ButtonState) {
        self.nes.set_buttons(buttons);
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
