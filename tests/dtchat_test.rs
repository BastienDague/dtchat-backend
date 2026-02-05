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

#[test]
fn update_with_invalid_path() {
    setup();
    let mut model = ChatModel::new();

    let obs = Arc::new(Mutex::new(Obs::default()));
    model.add_observer(obs.clone());

    model.update("this_file_does_not_exist.ion".to_string(), "dijkstra");

    let obs = obs.lock().unwrap();

    assert!(!obs.events.is_empty());

    match obs.events.last().unwrap() {
        ChatAppEvent::Info(msg) => assert!(msg.contains("Update failed")),
        _ => panic!("Expected Info event"),
    }
}

#[test]
fn pbat_is_disabled() {
    setup();
    let mut model = ChatModel::new();

    model.update("invalid_path.ion".to_string(), "dijkstra");

    assert_eq!(model.is_pbat_enabled(), false);
}

#[test]
fn get_localpeer_returns_configured_peer() {
    // get_localpeer => config coherency (uuid expected "1", name expected "Instance 1")
    setup();
    let model = ChatModel::new();

    let me = model.get_localpeer();
    assert_eq!(me.uuid, "1");
    assert_eq!(me.name, "Instance 1");
}

#[test]
fn get_rooms_contains_default_room() {
    // get_rooms => contains the default room from YAML (id 1, name "Default")
    setup();
    let model = ChatModel::new();

    let rooms = model.get_rooms();
    assert!(rooms.contains_key("1"));
    assert_eq!(rooms.get("1").unwrap().name, "Default");
}

#[test]
fn get_other_peers_for_room_returns_all_except_local() {
    // get_other_peers_for_room => returns all peers in room except local peer (2 and 3 in room 1)
    setup();
    let model = ChatModel::new();

    let others = model.get_other_peers_for_room(&"1".to_string()).expect("room 1 should exist");
    assert_eq!(others.len(), 2);

    // others: Vec<(peer_uuid, Endpoint)>
    let mut ids: Vec<String> = others.into_iter().map(|(id, _ep)| id).collect();
    ids.sort();
    assert_eq!(ids, vec!["2".to_string(), "3".to_string()]);
}

#[test]
fn get_other_peers_for_room_unknown_room_is_none() {
    // unknown room => None
    setup();
    let model = ChatModel::new();

    let res = model.get_other_peers_for_room(&"999".to_string());
    assert!(res.is_none());
}

#[test]
fn get_other_peers_contains_known_peers() {
    // get_other_peers => contains distant peers from YAML (2 and 3)
    setup();
    let model = ChatModel::new();

    let peers = model.get_other_peers();
    assert!(peers.contains_key("2"));
    assert!(peers.contains_key("3"));
}

#[test]
fn get_last_messages_returns_at_most_count() {
    // get_last_messages => always inferior to count
    setup();
    let mut model = ChatModel::new();

    let msgs = model.get_last_messages(10);
    assert!(msgs.len() <= 10);
}
