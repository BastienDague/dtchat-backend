use std::sync::{Arc, Mutex, Once};

use dtchat_backend::dtchat::{generate_uuid, ChatModel};
use dtchat_backend::event::{AppEventObserver, ChatAppEvent};

use uuid::Uuid;

// Run env init only once for all tests
static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        // Must match an existing peer in default.yaml
        std::env::set_var("PEER_UUID", "1");
        std::env::set_var("PEER_NAME", "Instance 1");
    });
}

// Simple observer that just stores received events
#[derive(Default)]
struct Obs {
    events: Vec<ChatAppEvent>,
}

impl AppEventObserver for Obs {
    fn on_event(&mut self, event: ChatAppEvent) {
        self.events.push(event);
    }
}

// Helper: get last Info string if the last event is Info(...), very helpful for assertions
fn last_info(events: &[ChatAppEvent]) -> Option<&str> {
    if let Some(ChatAppEvent::Info(s)) = events.last() {
        return Some(s);
    }
    None
}

#[test]
fn test_generate_uuid() {
    // Non-empty + different + valid UUID
    let a = generate_uuid();
    let b = generate_uuid();

    assert!(!a.is_empty());
    assert!(!b.is_empty());
    assert_ne!(a, b);

    assert!(Uuid::parse_str(&a).is_ok());
    assert!(Uuid::parse_str(&b).is_ok());
}

#[test]
fn test_notify_one_observer() {
    setup();
    let mut model = ChatModel::new();

    let obs = Arc::new(Mutex::new(Obs::default()));
    model.add_observer(obs.clone());

    model.notify_observers(ChatAppEvent::Info("hello".to_string()));

    let obs = obs.lock().unwrap();
    assert_eq!(obs.events.len(), 1);
    assert_eq!(last_info(&obs.events), Some("hello"));
}

#[test]
fn test_notify_two_observers() {
    setup();
    let mut model = ChatModel::new();

    let o1 = Arc::new(Mutex::new(Obs::default()));
    let o2 = Arc::new(Mutex::new(Obs::default()));
    model.add_observer(o1.clone());
    model.add_observer(o2.clone());

    model.notify_observers(ChatAppEvent::Info("broadcast".to_string()));

    assert_eq!(o1.lock().unwrap().events.len(), 1);
    assert_eq!(o2.lock().unwrap().events.len(), 1);
}

#[test]
fn test_notify_order() {
    setup();
    let mut model = ChatModel::new();

    let obs = Arc::new(Mutex::new(Obs::default()));
    model.add_observer(obs.clone());

    model.notify_observers(ChatAppEvent::Info("un".to_string()));
    model.notify_observers(ChatAppEvent::Info("deux".to_string()));
    model.notify_observers(ChatAppEvent::Info("trois".to_string()));

    let obs = obs.lock().unwrap();
    assert_eq!(obs.events.len(), 3);

    // Check order
    assert_eq!(last_info(&obs.events[0..1]), Some("un"));
    assert_eq!(last_info(&obs.events[1..2]), Some("deux"));
    assert_eq!(last_info(&obs.events[2..3]), Some("trois"));

    //instead of last_info, we could also match the event directly, but is longer to write
}
