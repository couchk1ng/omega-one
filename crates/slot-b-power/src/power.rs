use std::time::Duration;

use crate::{Battery, Charge, LedState, LidPolicy, Platform, Woke};

/// How long a handheld that cannot suspend stays awake with the lid shut, at most.
const AWAKE_TIMEOUT: Duration = Duration::from_secs(180);

pub struct Power {
    platform: Box<dyn Platform>,
    level: u8,
    closed: bool,
    timeout: Duration,
    suspend_failed: bool,
}

impl Power {
    pub fn new(platform: Box<dyn Platform>, timeout: Duration) -> Self {
        Power {
            platform,
            level: 0,
            closed: false,
            timeout,
            suspend_failed: false,
        }
    }

    /// With the lid shut, sleeps the handheld for what is left of the timeout, and again each
    /// time something but the lid wakes it. A handheld that cannot suspend stays awake, on the
    /// shorter timeout.
    fn sleep_while_shut(&mut self) {
        self.suspend_failed = false;
        let began = self.platform.now();
        loop {
            let waited = u64::try_from(self.platform.now() - began).unwrap_or(0);
            let left = self.timeout.as_secs().saturating_sub(waited);
            if left == 0 {
                return;
            }
            match self.platform.suspend(left) {
                Woke::Awake => {
                    self.suspend_failed = true;
                    return;
                }
                Woke::Clock => return,
                Woke::Other => {
                    if !self.platform.lid_shut() {
                        return;
                    }
                }
            }
        }
    }

    pub fn set_backlight(&mut self, step: u8) {
        self.level = step;
        if !self.closed {
            self.platform.set_backlight(step);
        }
    }

    pub fn battery(&self) -> Option<Battery> {
        self.platform.battery()
    }

    pub fn charge(&self) -> Charge {
        self.platform.charge()
    }

    pub fn headphones(&self) -> bool {
        self.platform.headphones()
    }

    pub fn set_led(&mut self, state: LedState) {
        self.platform.set_led(state)
    }

    pub fn set_rumble(&mut self, strength: u16) {
        self.platform.set_rumble(strength);
    }

    pub fn relink_adb(&mut self) -> bool {
        self.platform.relink_adb()
    }

    pub fn poweroff(&mut self) -> ! {
        self.platform.poweroff()
    }

    pub fn restart(&mut self) -> ! {
        self.platform.restart()
    }

    pub fn now(&self) -> i64 {
        self.platform.now()
    }

    pub fn set_clock(&mut self, secs: i64) {
        self.platform.set_clock(secs);
    }
}

impl LidPolicy for Power {
    fn on_close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        self.platform.set_backlight(0);
        self.sleep_while_shut();
    }

    fn on_open(&mut self) {
        self.closed = false;
        self.platform.set_backlight(self.level);
    }

    fn timeout(&self) -> Duration {
        if self.suspend_failed {
            self.timeout.min(AWAKE_TIMEOUT)
        } else {
            self.timeout
        }
    }
}
