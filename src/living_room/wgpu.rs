use std::time::Instant;

use pollster::block_on;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    frame::{FRAME_HEIGHT, FRAME_WIDTH},
    input::ButtonState,
    living_room::{LivingRoom, PRESENT_INTERVAL, WAKE_INTERVAL},
    tv::wgpu::WgpuTV,
};

impl ApplicationHandler for LivingRoom<WgpuTV> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.tv.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("lynes")
            .with_inner_size(PhysicalSize::new(
                FRAME_WIDTH as u32 * 3,
                FRAME_HEIGHT as u32 * 3,
            ));

        let window = match event_loop.create_window(attributes) {
            Ok(window) => window,
            Err(error) => {
                eprintln!("Failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };

        match block_on(WgpuTV::new(window)) {
            Ok(tv) => {
                tv.window().request_redraw();
                self.set_tv(tv);
                self.last_tick = Some(Instant::now());
                self.cycle_fraction = 0.0;
                self.next_present_time = None;
                self.frame_pending = true;
            }
            Err(error) => {
                eprintln!("Failed to initialize display: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(tv) = self.tv.as_mut() else {
            return;
        };

        if tv.window().id() != window_id {
            return;
        }

        use WindowEvent::*;
        match event {
            CloseRequested => event_loop.exit(),
            Resized(size) => {
                tv.resize(size);

                if size.width > 0 && size.height > 0 {
                    tv.window().request_redraw();
                }
            }
            RedrawRequested => {
                self.frame_pending = true;

                let now = Instant::now();

                if let Some(deadline) = self.next_present_time {
                    if now < deadline {
                        return;
                    }
                }

                let result = self.present();
                if let Err(error) = result {
                    eprintln!("Failed to present frame: {error}");
                    event_loop.exit();
                }

                self.next_present_time = Some(now + PRESENT_INTERVAL);
                self.frame_pending = false
            }
            KeyboardInput { event, .. } => {
                let PhysicalKey::Code(key) = event.physical_key else {
                    return;
                };
                let pressed = event.state == ElementState::Pressed;

                use KeyCode::*;
                self.update_buttons(|buttons| match key {
                    KeyZ => buttons.set_a(pressed),
                    KeyX => buttons.set_b(pressed),
                    Enter => buttons.set_start(pressed),
                    ShiftRight => buttons.set_select(pressed),
                    KeyW => buttons.set_up(pressed),
                    KeyS => buttons.set_down(pressed),
                    KeyA => buttons.set_left(pressed),
                    KeyD => buttons.set_right(pressed),
                    _ => {}
                });
            }
            Focused(false) => self.update_buttons(ButtonState::clear),
            _ => {}
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.tv = None;
        self.last_tick = None;
        self.cycle_fraction = 0.0;
        self.next_present_time = None;
        self.frame_pending = true;
        self.update_buttons(ButtonState::clear);

        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }

        if self.tv.is_none() {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }

        self.advance_to(Instant::now());

        let now = Instant::now();
        let mut wake_at = now + WAKE_INTERVAL;

        if self.frame_pending {
            let deadline = self.next_present_time.unwrap_or(now);

            if now >= deadline {
                if let Some(tv) = &self.tv {
                    tv.window().request_redraw();
                }
            } else {
                wake_at = wake_at.min(deadline);
            }
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(wake_at));
    }
}
