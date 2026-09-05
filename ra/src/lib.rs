#![cfg_attr(not(test), no_std)]
use os_terminal::{DrawTarget, Rgb, Terminal};
use spin::Mutex;
#[doc = "The `fs` module contains the implementation of the file system."]
pub mod fs;
#[doc = "The `tree` module contains the implementation of the Trie data structure."]
pub mod tree;

/// Represents the framebuffer data for the screen.
///
/// # Fields
/// - `ptr`: A pointer to the framebuffer memory.
/// - `width`: The width of the screen in pixels.
/// - `height`: The height of the screen in pixels.
/// - `pitch`: The number of bytes in a single row of the framebuffer.
pub struct ScreenData {
    pub ptr: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
}
// SAFETY: The framebuffer pointer is managed exclusively by the kernel and is safe to send between threads.
unsafe impl Send for ScreenData {}
// SAFETY: Concurrent access to the framebuffer is strictly protected by the global Mutex (TERMINAL).
unsafe impl Sync for ScreenData {}

impl DrawTarget for ScreenData {
    fn size(&self) -> (usize, usize) {
        (
            usize::try_from(self.width).unwrap_or(0),
            usize::try_from(self.height).unwrap_or(0),
        )
    }

    #[inline]
    fn draw_pixel(&mut self, x: usize, y: usize, color: Rgb) {
        let screen_x = u64::try_from(x).unwrap_or(0);
        let screen_y = u64::try_from(y).unwrap_or(0);
        let offset = (screen_y * self.pitch) + (screen_x * 4);
        let offset_usize = usize::try_from(offset).unwrap_or(0);
        #[allow(clippy::cast_ptr_alignment)]
        // SAFETY: L'offset est calculé mathématiquement pour correspondre aux limites du Framebuffer vidéo.
        unsafe {
            let pixel_ptr = self.ptr.add(offset_usize).cast::<u32>();
            let raw_color =
                (u32::from(color.0) << 16) | (u32::from(color.1) << 8) | u32::from(color.2);
            *pixel_ptr = raw_color;
        }
    }
}

/// A global mutex-protected terminal instance that can be safely accessed across threads.
pub static TERMINAL: Mutex<Option<Terminal<ScreenData>>> = Mutex::new(None);
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        if let Some(term) = $crate::TERMINAL.lock().as_mut() {
            let _ = core::fmt::write(term, format_args!($($arg)*));
        }
    }};
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => { $crate::print!("{}\n", format_args!($($arg)*)) };
}
/// The `RaHealth` enum represents the health status of the Ra (Application Kernel).
///
/// # Variants
/// * `Ok` - The system is healthy and all plans are running normally.
/// * `PlanFailed(usize)` - A specific plan has failed, returning its ID.
/// * `Compromised` - The application kernel has detected a compromise or corruption.
pub enum RaHealth {
    /// The system is healthy and all plans are running normally.
    Ok,
    /// A specific plan has failed, returning its ID.
    PlanFailed(usize),
    /// The application kernel has detected a compromise or corruption.
    Compromised,
}

/// The `PlanState` enum represents the state of a plan (process or application) in the Ra (Application Kernel).
/// # Variants
/// * `Empty` - The plan slot is free in RAM.
/// * `Running` - The plan is currently running.
/// * `Failed` - The plan has crashed (e.g., Segfault).
#[derive(Copy, Clone)]
pub enum PlanState {
    /// The plan slot is free in RAM.
    Empty,
    /// The plan is currently running.
    Running,
    /// The plan has crashed (e.g., Segfault).
    Failed,
}

/// The `Plan` struct represents a plan (process or application) managed by the Ra (Application Kernel).
/// # Fields
/// * `id` - The unique identifier of the plan.
/// * `state` - The current state of the plan, represented by the `PlanState` enum.
#[derive(Copy, Clone)]
pub struct Plan {
    pub id: usize,
    pub state: PlanState,
}

/// The Ra (Application Kernel) manages the lifecycle of plans (processes or applications).
pub struct Ra {
    pub plans: [Plan; 64],
}

impl Default for Ra {
    fn default() -> Self {
        Self::new()
    }
}

impl Ra {
    /// Create a new instance of Ra with an empty plan list.
    /// # Returns
    /// * `Ra` - A new instance of Ra with an empty plan list
    #[must_use]
    pub const fn new() -> Self {
        const EMPTY_PLAN: Plan = Plan {
            id: 0,
            state: PlanState::Empty,
        };

        Self {
            plans: [EMPTY_PLAN; 64],
        }
    }

    /// The application management cycle. Called in a loop by `re`.
    pub fn tick(&mut self) -> RaHealth {
        for plan in &mut self.plans {
            match plan.state {
                PlanState::Failed => {
                    return RaHealth::PlanFailed(plan.id);
                }
                PlanState::Running | PlanState::Empty => {}
            }
        }
        RaHealth::Ok
    }
}
