use rs_archiver::storage::AttachmentStatus::{
    FailedPermanent, FailedRetryable, SkippedTooSmall, Uploaded,
};
use rs_archiver::storage::{AttachmentKey, AttachmentState, AttachmentStatus, Entry, Store};
use std::path::PathBuf;
use std::{collections::HashMap, sync::Mutex};

fn create_entry(uidl: &str, attch_name: &str, status: AttachmentStatus, attemps: u8) -> Entry {
    Entry {
        state: AttachmentState {
            status: status,
            attempts: attemps as u32,
        },
        key: AttachmentKey {
            uidl: uidl.into(),
            attachment: attch_name.into(),
        },
    }
}

#[tokio::test]
async fn test_create_or_upload() {
    struct TestCleanup {
        path: std::path::PathBuf,
    }

    impl Drop for TestCleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }
    let path = PathBuf::from("test.json");
    assert_eq!(!path.exists(), true);

    let temp = TestCleanup { path: path.clone() };
    if let Ok(s) = Store::load_or_create(temp.path.clone()).await {
        assert_eq!(s.attachments.lock().unwrap().is_empty(), true);
        assert_eq!(temp.path.exists(), false);
        let mut attchs = s.attachments.lock().unwrap();
        let k = AttachmentKey {
            uidl: "A".into(),
            attachment: "A".into(),
        };
        let v = AttachmentState {
            status: Uploaded,
            attempts: 1,
        };
        attchs.insert(k.clone(), v);
        drop(attchs);
        if let Ok(()) = s.record_outcome(k, Uploaded).await {
            assert_eq!(s.attachments.lock().unwrap().is_empty(), false);
            assert_eq!(temp.path.exists(), true);
        }
    }
    drop(path);
}
#[test]
fn test_is_uploadable() {
    let entry_vec = vec![
        create_entry("A", "A.pdf", Uploaded, 1),
        create_entry("B", "B.pdf", SkippedTooSmall, 1),
        create_entry("C", "C.pdf", FailedRetryable, 1),
        create_entry("D", "D.pdf", FailedRetryable, 9),
        create_entry("E", "E.pdf", FailedPermanent, 1),
    ];
    let h_map: HashMap<AttachmentKey, AttachmentState> =
        entry_vec.into_iter().map(|e| (e.key, e.state)).collect();

    let path = PathBuf::from("test.json");
    let store = Store {
        attachments: Mutex::new(h_map),
        path,
        max_attch_attempts: 2,
    };

    assert_eq!(
        store.is_uploadable(&AttachmentKey {
            uidl: "nwe".into(),
            attachment: "new-file.pdf".into()
        }),
        true
    );
    assert_eq!(
        store.is_uploadable(&AttachmentKey {
            uidl: "A".into(),
            attachment: "A.pdf".into()
        }),
        false
    );
    assert_eq!(
        store.is_uploadable(&AttachmentKey {
            uidl: "B".into(),
            attachment: "B.pdf".into()
        }),
        false
    );
    assert_eq!(
        store.is_uploadable(&AttachmentKey {
            uidl: "C".into(),
            attachment: "C.pdf".into()
        }),
        true
    );
    assert_eq!(
        store.is_uploadable(&AttachmentKey {
            uidl: "D".into(),
            attachment: "D.pdf".into()
        }),
        false
    );
    assert_eq!(
        store.is_uploadable(&AttachmentKey {
            uidl: "E".into(),
            attachment: "E.pdf".into()
        }),
        false
    );
}
