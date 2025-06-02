use crate::session::Session;
use crate::telnet::Telnet;
use crate::timestamp::Timestamp;
use crate::types::EventType;
use async_trait::async_trait;
use log::info;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait Event: Send + Sync {
    fn event_type(&self) -> EventType;
    fn time(&self) -> Timestamp;
    fn set_abs_time(&mut self, when: i64);
    fn set_rel_time(&mut self, when: i64);
    async fn execute(&mut self) -> bool; // Returns true to reschedule
}

#[derive(Debug, Clone)]
pub struct ShutdownEvent {
    time: Timestamp,
    final_warning: bool,
    by: String,
}

impl ShutdownEvent {
    pub const FINAL_WARNING_TIME: i64 = 3;

    pub async fn new(by: String, when: i64) -> Self {
        let mut event = Self {
            time: Timestamp::new(),
            final_warning: false,
            by,
        };
        event.set_rel_time(when);
        event.shutdown_warning(when).await;
        event
    }

    pub fn immediate(by: String) -> Self {
        info!("Immediate shutdown requested by {}.", by);
        let mut event = Self {
            time: Timestamp::new(),
            final_warning: false,
            by,
        };
        event.final_warning().await;
        event
    }

    pub async fn shutdown_warning(&self, when: i64) {
        info!("Shutdown requested by {} in {} seconds.", self.by, when);
        Session::announce(&format!(
            "\x07>>> This server will shutdown in {} seconds... <<<\n\x07",
            when
        ))
        .await;
    }

    pub async fn final_warning(&mut self) {
        self.final_warning = true;
        self.set_rel_time(Self::FINAL_WARNING_TIME);
        info!("Final shutdown warning.");
        Session::announce("\x07>>> Server shutting down NOW!  Goodbye. <<<\n\x07").await;
    }

    pub async fn shutdown_server(&self) {
        info!("Server down.");
        // In a real implementation, this would trigger graceful shutdown
        std::process::exit(0);
    }
}

#[async_trait]
impl Event for ShutdownEvent {
    fn event_type(&self) -> EventType {
        EventType::ShutdownEvent
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    fn set_abs_time(&mut self, when: i64) {
        self.time = Timestamp::from_unix(when);
    }

    fn set_rel_time(&mut self, when: i64) {
        self.time = Timestamp::new() + when;
    }

    async fn execute(&mut self) -> bool {
        if self.final_warning {
            self.shutdown_server().await;
            false
        } else {
            self.final_warning().await;
            true
        }
    }
}

#[derive(Debug, Clone)]
pub struct RestartEvent {
    time: Timestamp,
    final_warning: bool,
    by: String,
}

impl RestartEvent {
    pub const FINAL_WARNING_TIME: i64 = 3;

    pub fn new(by: String, when: i64) -> Self {
        let mut event = Self {
            time: Timestamp::new(),
            final_warning: false,
            by,
        };
        event.set_rel_time(when);
        event.restart_warning(when).await;
        event
    }

    pub fn immediate(by: String) -> Self {
        info!("Immediate restart requested by {}.", by);
        let mut event = Self {
            time: Timestamp::new(),
            final_warning: false,
            by,
        };
        event.final_warning().await;
        event
    }

    pub async fn restart_warning(&self, when: i64) {
        info!("Restart requested by {} in {} seconds.", self.by, when);
        Session::announce(&format!(
            "\x07>>> This server will restart in {} seconds... <<<\n\x07",
            when
        ))
        .await;
    }

    pub async fn final_warning(&mut self) {
        self.final_warning = true;
        self.set_rel_time(Self::FINAL_WARNING_TIME);
        info!("Final restart warning.");
        Session::announce("\x07>>> Server restarting NOW!  Goodbye. <<<\n\x07").await;
    }

    pub async fn restart_server(&self) {
        info!("Restarting server.");
        // In a real implementation, this would exec the server binary
        std::process::exit(0);
    }
}

#[async_trait]
impl Event for RestartEvent {
    fn event_type(&self) -> EventType {
        EventType::RestartEvent
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    fn set_abs_time(&mut self, when: i64) {
        self.time = Timestamp::from_unix(when);
    }

    fn set_rel_time(&mut self, when: i64) {
        self.time = Timestamp::new() + when;
    }

    async fn execute(&mut self) -> bool {
        if self.final_warning {
            self.restart_server().await;
            false
        } else {
            self.final_warning().await;
            true
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoginTimeoutEvent {
    time: Timestamp,
    telnet: Arc<RwLock<Telnet>>,
}

impl LoginTimeoutEvent {
    pub fn new(telnet: Arc<RwLock<Telnet>>, when: i64) -> Self {
        let mut event = Self {
            time: Timestamp::new(),
            telnet,
        };
        event.set_rel_time(when);
        event
    }
}

#[async_trait]
impl Event for LoginTimeoutEvent {
    fn event_type(&self) -> EventType {
        EventType::LoginTimeoutEvent
    }

    fn time(&self) -> Timestamp {
        self.time
    }

    fn set_abs_time(&mut self, when: i64) {
        self.time = Timestamp::from_unix(when);
    }

    fn set_rel_time(&mut self, when: i64) {
        self.time = Timestamp::new() + when;
    }

    async fn execute(&mut self) -> bool {
        let mut telnet = self.telnet.write().await;
        telnet.output("\nLogin timed out!\n").await;
        telnet.close(true).await;
        false
    }
}

// Event wrapper for heap ordering
#[derive(Debug, Clone)]
struct EventWrapper {
    event: Box<dyn Event>,
}

impl PartialEq for EventWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.event.time() == other.event.time()
    }
}

impl Eq for EventWrapper {}

impl PartialOrd for EventWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reverse order for min-heap behavior
        other.event.time().partial_cmp(&self.event.time())
    }
}

impl Ord for EventWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap behavior
        other.event.time().cmp(&self.event.time())
    }
}

#[derive(Debug, Clone)]
pub struct EventQueue {
    queue: Arc<RwLock<BinaryHeap<EventWrapper>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
        }
    }

    pub async fn enqueue(&self, event: Box<dyn Event>) {
        let mut queue = self.queue.write().await;
        queue.push(EventWrapper { event });
    }

    pub async fn dequeue(&self, event_type: EventType) {
        let mut queue = self.queue.write().await;
        let events: Vec<_> = queue.drain().collect();
        for wrapper in events {
            if wrapper.event.event_type() != event_type {
                queue.push(wrapper);
            }
        }
    }

    pub async fn execute(&self) -> Option<std::time::Duration> {
        loop {
            let now = Timestamp::new();
            let mut queue = self.queue.write().await;

            if let Some(mut wrapper) = queue.peek_mut() {
                if wrapper.event.time() <= now {
                    let mut event = queue.pop().unwrap().event;
                    drop(queue); // Release lock before executing

                    if event.execute().await {
                        self.enqueue(event).await;
                    }
                } else {
                    let wait_time = wrapper.event.time() - now;
                    return Some(std::time::Duration::from_secs(wait_time as u64));
                }
            } else {
                return None;
            }
        }
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}
