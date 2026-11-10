use crate::db::*;
use rusqlite::Connection;

#[test]
fn test_pubsub_topic_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    let topic = insert_pubsub_topic(&conn, "t1", "my_topic").unwrap();
    assert_eq!(topic.name, "my_topic");

    let topics = list_pubsub_topics(&conn).unwrap();
    assert_eq!(topics.len(), 1);

    let by_name = get_pubsub_topic_by_name(&conn, "my_topic")
        .unwrap()
        .unwrap();
    assert_eq!(by_name.id, "t1");

    delete_pubsub_topic(&conn, "my_topic").unwrap();
    assert!(list_pubsub_topics(&conn).unwrap().is_empty());
}

// --- PubSub Messages ---

#[test]
fn test_pubsub_message_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_pubsub_topic(&conn, "topic1", "chat").unwrap();
    insert_pubsub_message(&conn, "m1", "topic1", "hello world").unwrap();

    let msgs = list_pubsub_messages(&conn, "topic1").unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].body, "hello world");
}

// --- Cron Job CRUD ---

#[test]
fn test_list_pubsub_messages_empty() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(
        list_pubsub_messages(&conn, "nonexistent")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn test_get_pubsub_topic_by_name_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(
        get_pubsub_topic_by_name(&conn, "nonexistent")
            .unwrap()
            .is_none()
    );
}
