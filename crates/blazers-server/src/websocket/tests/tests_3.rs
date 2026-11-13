use super::*;
use axum::extract::ws::Message;
use tokio::sync::mpsc;

async fn client(
    hub: &std::sync::Arc<RwLock<WSHub>>,
    id: &str,
    subs: Vec<(&str, Option<&str>)>,
) -> mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = mpsc::unbounded_channel();
    let map: std::collections::HashMap<String, Option<String>> = subs
        .iter()
        .map(|(c, f)| (c.to_string(), f.map(str::to_string)))
        .collect();
    hub.write().await.clients.insert(
        id.to_string(),
        WSConsumer {
            sender: tx,
            subs: Mutex::new(map),
        },
    );
    rx
}

#[tokio::test]
async fn broadcast_record_change_honors_subscription_filter() {
    let hub = new_ws_hub();
    let mut rx_match = client(&hub, "match", vec![("posts", Some("views > 50"))]).await;
    let mut rx_skip = client(&hub, "skip", vec![("posts", Some("views < 50"))]).await;

    let rec = serde_json::json!({"id": "r1", "data": {"views": 100}});
    broadcast_record_change(&hub, "posts", "create", &rec, None).await;
    assert!(rx_match.recv().await.is_some());
    assert!(rx_skip.try_recv().is_err());

    let rec2 = serde_json::json!({"id": "r2", "data": {"views": 10}});
    broadcast_record_change(&hub, "posts", "create", &rec2, None).await;
    assert!(rx_skip.recv().await.is_some());
    assert!(rx_match.try_recv().is_err());
}

#[tokio::test]
async fn presence_join_state_leave() {
    let hub = new_ws_hub();
    let mut rx1 = client(&hub, "c1", vec![]).await;
    let mut rx2 = client(&hub, "c2", vec![]).await;

    let frame =
        |action: &str, extra: &str| format!(r#"{{"action":"{action}","channel":"room"{extra}}}"#);
    let send = |id: &str, text: String| {
        let hub = hub.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            crate::websocket::frames::handle_client_frame(&id, &text, &hub);
        })
    };

    send("c1", frame("presence_join", "")).await.unwrap();
    // c1 receives its own join event
    let msg = rx1.recv().await.unwrap();
    let Message::Text(t) = msg else {
        panic!("text")
    };
    let v: Value = serde_json::from_str(&t).unwrap();
    assert_eq!(v["type"], "presence");
    assert_eq!(v["action"], "join");

    send("c2", frame("presence", r#","state":{"cursor":1}"#))
        .await
        .unwrap();
    // both members get the state event
    for rx in [&mut rx1, &mut rx2] {
        let msg = rx.recv().await.unwrap();
        let Message::Text(t) = msg else {
            panic!("text")
        };
        let v: Value = serde_json::from_str(&t).unwrap();
        assert_eq!(v["action"], "state");
        assert_eq!(v["state"]["cursor"], 1);
    }

    // disconnect cleanup broadcasts leave to remaining members
    crate::websocket::frames::presence_leave_all(&hub, "c1").await;
    let msg = rx2.recv().await.unwrap();
    let Message::Text(t) = msg else {
        panic!("text")
    };
    let v: Value = serde_json::from_str(&t).unwrap();
    assert_eq!(v["action"], "leave");
    assert_eq!(v["client"], "c1");
    assert!(rx1.try_recv().is_err());
}
