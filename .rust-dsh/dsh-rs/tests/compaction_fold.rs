//! The compaction event folds the model-visible surface, live and on replay.

use dsh_rs::session::{user_message, Session, SessionEventData, SessionHeader};

fn header() -> SessionHeader {
    SessionHeader {
        cwd: None,
        created_at: 0,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    }
}

fn say(session: &Session, id: &str) {
    session.append(SessionEventData::UserMessage {
        message: user_message(id, id),
    });
}

fn visible(session: &Session) -> Vec<String> {
    session.derive_messages().iter().map(|m| m.text()).collect()
}

#[test]
fn a_compaction_replaces_everything_before_it_and_keeps_what_follows() {
    let session = Session::new("s".into(), header(), vec![]);
    say(&session, "a");
    say(&session, "b");
    let cut = session.seq(); // "c" is the first message kept
    say(&session, "c");
    session.append(SessionEventData::Compaction {
        summary: "S1".into(),
        before_seq: cut,
    });
    say(&session, "d");

    let seen = visible(&session);
    assert_eq!(seen.len(), 3, "{seen:?}");
    assert!(seen[0].contains("S1"), "summary comes first: {seen:?}");
    assert_eq!(&seen[1..], ["c", "d"]);
}

#[test]
fn a_second_compaction_absorbs_the_first_summary() {
    let session = Session::new("s".into(), header(), vec![]);
    say(&session, "a");
    let cut = session.seq();
    say(&session, "b");
    session.append(SessionEventData::Compaction { summary: "S1".into(), before_seq: cut });
    say(&session, "c");
    let cut2 = session.seq();
    say(&session, "d");
    session.append(SessionEventData::Compaction { summary: "S2".into(), before_seq: cut2 });

    let seen = visible(&session);
    assert_eq!(seen.len(), 2, "{seen:?}");
    assert!(seen[0].contains("S2") && !seen[0].contains("S1"), "{seen:?}");
    assert_eq!(seen[1], "d");
}

#[test]
fn a_replayed_session_shows_the_model_what_the_live_one_did() {
    let live = Session::new("s".into(), header(), vec![]);
    say(&live, "a");
    let cut = live.seq();
    say(&live, "b");
    live.append(SessionEventData::Compaction { summary: "S1".into(), before_seq: cut });
    say(&live, "c");

    let replayed = Session::new("s".into(), header(), live.events());
    assert_eq!(visible(&live), visible(&replayed));
    assert_eq!(live.surface(), replayed.surface());
}

#[test]
fn compaction_and_goal_events_survive_a_json_round_trip() {
    for data in [
        SessionEventData::Compaction { summary: "x".into(), before_seq: 7 },
        SessionEventData::Goal {
            objective: "ship".into(),
            status: dsh_rs::session::GoalStatus::Active,
            note: None,
        },
    ] {
        let event = dsh_rs::session::SessionEvent::new(1, 2, data);
        let text = serde_json::to_string(&event).unwrap();
        let back: dsh_rs::session::SessionEvent = serde_json::from_str(&text).unwrap();
        assert_eq!(event, back, "{text}");
    }
}

/// Reviewer scenario B2: a reader must never see a surface from after a compaction
/// together with an event list from before it (the lookup would silently drop the new
/// summary, and the old messages are already off the surface).
///
/// Every state of this log has exactly one summary (the first compaction happens before
/// any reader starts) and at least one "keep" message. A torn read loses the summary.
#[test]
fn derive_messages_is_never_torn_by_a_concurrent_compaction() {
    use dsh_rs::session::Session;
    use dsh_rs::types::{ContentBlock, Message, SessionHeader};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let session = Arc::new(Session::new(
        "torn".into(),
        SessionHeader { cwd: None, created_at: 0, parent_session: None, seed_length: 0, agent_preset: None },
        vec![],
    ));
    let say = |s: &Session, id: &str| {
        s.append(SessionEventData::UserMessage {
            message: Message::user(id, vec![ContentBlock::text(id)]),
        })
    };
    let first = say(&session, "keep-start");
    session.append(SessionEventData::Compaction { summary: "s-start".into(), before_seq: first.seq });

    let stop = Arc::new(AtomicBool::new(false));
    let readers: Vec<_> = (0..6)
        .map(|_| {
            let (s, stop) = (session.clone(), stop.clone());
            std::thread::spawn(move || {
                let mut torn = 0usize;
                while !stop.load(Ordering::SeqCst) {
                    let msgs = s.derive_messages();
                    let summaries = msgs.iter().filter(|m| m.text().starts_with("[Summary")).count();
                    let keeps = msgs.iter().filter(|m| m.text().starts_with("keep")).count();
                    if summaries != 1 || keeps == 0 {
                        torn += 1;
                    }
                }
                torn
            })
        })
        .collect();
    for round in 0..3000 {
        let keep = say(&session, &format!("keep{round}"));
        // The compaction drops everything before `keep` and adds itself first.
        session.append(SessionEventData::Compaction { summary: format!("s{round}"), before_seq: keep.seq });
    }
    stop.store(true, Ordering::SeqCst);
    let torn: usize = readers.into_iter().map(|h| h.join().unwrap()).sum();
    assert_eq!(torn, 0, "readers saw an inconsistent history {torn} times");
}
