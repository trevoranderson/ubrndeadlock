use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{env, thread};

// You must call this once
uniffi::setup_scaffolding!();

#[derive(uniffi::Object)]
pub struct ExampleFfi {
    core: Arc<Mutex<ExampleDriver>>,
}

#[uniffi::export]
impl ExampleFfi {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            core: Arc::new(Mutex::new(ExampleDriver::new())),
        })
    }
    pub fn replace_event_handler(&self, event_handler: Arc<dyn RustEventHandler>) {
        self.core
            .lock()
            .unwrap()
            .replace_event_handler(event_handler);
    }
    pub fn start_idempotent(&self) {
        let core = self.core.clone();
        self.core.lock().unwrap().start_idempotent(core);
    }
}

#[uniffi::export(with_foreign)]
pub trait RustEventHandler: Send + Sync + Debug {
    fn on_update(&self, update: i32);
}

#[derive(Debug)]
struct DummyEventHandler {}

type StateUpdate = i32;

impl RustEventHandler for DummyEventHandler {
    fn on_update(&self, _update: StateUpdate) {}
}

pub struct ExampleDriver {
    pub state: StateUpdate,
    pub event_handler: Arc<dyn RustEventHandler>,
    pub thread: Option<JoinHandle<()>>,
}

fn create_thread_controls(driver: Arc<Mutex<ExampleDriver>>) -> JoinHandle<()> {
    let handle = thread::Builder::new()
        .name("rust_thread_controls".to_string())
        .spawn(move || loop {
            // Safe version. Never see deadlock, despite blocking UI thread with sleep
            // {
            //     let (event_handler, state) = {
            //         let mut driver = driver.lock().unwrap();
            //         driver.state += 1;
            //         thread::sleep(Duration::from_millis(100));
            //         (driver.event_handler.clone(), driver.state)
            //     };
            //     event_handler.on_update(state);
            // }
            // Unsafe version. Always deadlocks roughly instantly on my machine
            // The useState change from event handler causes the App to rebuild which
            // causes the event handler to be replaced which requires the driver lock.
            {
                let mut driver = driver.lock().unwrap();
                driver.state += 1;
                thread::sleep(Duration::from_millis(100));
                driver.event_handler.on_update(driver.state);
            }
        })
        .expect("Failed to spawn thread");
    handle
}

impl ExampleDriver {
    pub fn new() -> Self {
        unsafe {
            env::set_var("RUST_BACKTRACE", "1");
        }
        Self {
            state: 0,
            event_handler: Arc::new(DummyEventHandler {}),
            thread: None,
        }
    }
    pub fn replace_event_handler(&mut self, event_handler: Arc<dyn RustEventHandler>) {
        self.event_handler = event_handler;
    }
    pub fn start_idempotent(&mut self, core: Arc<Mutex<Self>>) {
        if self.thread.is_none() {
            let handle = create_thread_controls(core);
            self.thread = Some(handle);
        }
    }
}
