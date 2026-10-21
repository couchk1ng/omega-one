use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use slot_b_power::{Battery, Charge, LedState, LidPolicy, Platform, Power, Woke};

struct Panel {
    step: Arc<AtomicU8>,
    clock: i64,
}

impl Platform for Panel {
    fn set_backlight(&mut self, step: u8) {
        self.step.store(step, Ordering::Relaxed);
    }

    fn battery(&self) -> Option<Battery> {
        None
    }

    fn charge(&self) -> Charge {
        Charge::Unknown
    }

    fn set_led(&mut self, _state: LedState) {}

    fn restart(&mut self) -> ! {
        panic!("the panel never powers off")
    }

    fn poweroff(&mut self) -> ! {
        panic!("the panel never powers off")
    }

    fn root(&self) -> &Path {
        Path::new("sdcard")
    }

    fn now(&self) -> i64 {
        self.clock
    }

    fn set_clock(&mut self, secs: i64) {
        self.clock = secs;
    }

    fn set_rumble(&mut self, _strength: u16) {}
}

fn power() -> (Power, Arc<AtomicU8>) {
    let step = Arc::new(AtomicU8::new(0));
    let panel = Panel {
        step: step.clone(),
        clock: 0,
    };
    (Power::new(Box::new(panel), Duration::from_secs(60)), step)
}

fn lit() -> (Power, Arc<AtomicU8>) {
    power()
}

#[test]
fn the_lid_restores_the_level_it_darkened() {
    let (mut p, step) = lit();
    p.set_backlight(7);
    p.on_close();
    assert_eq!(step.load(Ordering::Relaxed), 0, "the panel stayed lit");
    p.on_open();
    assert_eq!(step.load(Ordering::Relaxed), 7);
}

#[test]
fn closing_twice_does_not_swallow_the_level() {
    let (mut p, step) = lit();
    p.set_backlight(4);
    p.on_close();
    p.on_close();
    p.on_open();
    assert_eq!(step.load(Ordering::Relaxed), 4);
}

#[test]
fn a_level_set_while_shut_waits_for_the_open() {
    let (mut p, step) = lit();
    p.set_backlight(2);
    p.on_close();
    p.set_backlight(9);
    assert_eq!(step.load(Ordering::Relaxed), 0);
    p.on_open();
    assert_eq!(step.load(Ordering::Relaxed), 9);
}

/// A handheld that sleeps as scripted: each suspend ends as the next entry says, after as many
/// seconds as it says.
struct Sleeper {
    step: Arc<AtomicU8>,
    clock: i64,
    wakes: Vec<(Woke, i64)>,
    shut_after: bool,
    asked: Arc<Mutex<Vec<u64>>>,
}

impl Platform for Sleeper {
    fn set_backlight(&mut self, step: u8) {
        self.step.store(step, Ordering::Relaxed);
    }

    fn battery(&self) -> Option<Battery> {
        None
    }

    fn charge(&self) -> Charge {
        Charge::Unknown
    }

    fn set_led(&mut self, _state: LedState) {}

    fn restart(&mut self) -> ! {
        panic!("the sleeper never powers off")
    }

    fn poweroff(&mut self) -> ! {
        panic!("the sleeper never powers off")
    }

    fn root(&self) -> &Path {
        Path::new("sdcard")
    }

    fn now(&self) -> i64 {
        self.clock
    }

    fn set_clock(&mut self, secs: i64) {
        self.clock = secs;
    }

    fn set_rumble(&mut self, _strength: u16) {}

    fn suspend(&mut self, secs: u64) -> Woke {
        self.asked.lock().unwrap().push(secs);
        if self.wakes.is_empty() {
            return Woke::Awake;
        }
        let (woke, lasted) = self.wakes.remove(0);
        self.clock += lasted;
        woke
    }

    fn lid_shut(&self) -> bool {
        self.shut_after
    }
}

fn sleeper(
    wakes: Vec<(Woke, i64)>,
    shut_after: bool,
    timeout: u64,
) -> (Power, Arc<Mutex<Vec<u64>>>, Arc<AtomicU8>) {
    let asked = Arc::new(Mutex::new(Vec::new()));
    let step = Arc::new(AtomicU8::new(0));
    let platform = Sleeper {
        step: step.clone(),
        clock: 0,
        wakes,
        shut_after,
        asked: asked.clone(),
    };
    let power = Power::new(Box::new(platform), Duration::from_secs(timeout));
    (power, asked, step)
}

#[test]
fn a_shut_lid_sleeps_for_the_whole_timeout() {
    let (mut p, asked, _) = sleeper(vec![(Woke::Clock, 60)], true, 60);
    p.on_close();
    assert_eq!(*asked.lock().unwrap(), vec![60]);
    assert_eq!(p.timeout(), Duration::from_secs(60));
}

#[test]
fn a_wake_by_a_key_with_the_lid_still_shut_sleeps_on_for_what_is_left() {
    let (mut p, asked, _) = sleeper(vec![(Woke::Other, 20), (Woke::Clock, 40)], true, 60);
    p.on_close();
    assert_eq!(*asked.lock().unwrap(), vec![60, 40]);
}

#[test]
fn a_wake_with_the_lid_open_sleeps_no_more() {
    let (mut p, asked, _) = sleeper(vec![(Woke::Other, 5), (Woke::Clock, 55)], false, 60);
    p.on_close();
    assert_eq!(*asked.lock().unwrap(), vec![60]);
}

#[test]
fn a_wake_at_the_very_end_does_not_start_a_sleep_of_no_time() {
    let (mut p, asked, _) = sleeper(vec![(Woke::Other, 60), (Woke::Clock, 1)], true, 60);
    p.on_close();
    assert_eq!(*asked.lock().unwrap(), vec![60]);
}

#[test]
fn a_handheld_that_cannot_suspend_falls_back_to_three_minutes() {
    let (mut p, _, _) = sleeper(vec![], true, 30 * 60);
    assert_eq!(p.timeout(), Duration::from_secs(30 * 60));
    p.on_close();
    assert_eq!(p.timeout(), Duration::from_secs(180));
}

#[test]
fn a_timeout_shorter_than_three_minutes_is_kept_when_it_cannot_suspend() {
    let (mut p, _, _) = sleeper(vec![], true, 60);
    p.on_close();
    assert_eq!(p.timeout(), Duration::from_secs(60));
}

#[test]
fn a_failed_suspend_is_tried_again_at_the_next_close() {
    let (mut p, asked, _) = sleeper(vec![(Woke::Awake, 0), (Woke::Clock, 60)], true, 30 * 60);
    p.on_close();
    assert_eq!(p.timeout(), Duration::from_secs(180));
    p.on_open();
    p.on_close();
    assert_eq!(asked.lock().unwrap().len(), 2);
    assert_eq!(p.timeout(), Duration::from_secs(30 * 60));
}

#[test]
fn the_backlight_is_off_while_the_lid_is_shut_and_back_when_it_opens() {
    let (mut p, _, step) = sleeper(vec![(Woke::Clock, 60)], true, 60);
    p.set_backlight(7);
    assert_eq!(step.load(Ordering::Relaxed), 7);
    p.on_close();
    assert_eq!(step.load(Ordering::Relaxed), 0);
    p.on_open();
    assert_eq!(step.load(Ordering::Relaxed), 7);
}
