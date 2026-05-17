/// Integration tests for LingListen core API — save, read, delete meetings.
///
/// Run: `cargo test -p linglisten --test integration`

use sqlx::SqlitePool;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

fn make_segment(text: &str, seq: u64) -> app_lib::api::TranscriptSegment {
    app_lib::api::TranscriptSegment {
        id: format!("seg_{}", seq),
        text: text.to_string(),
        timestamp: "14:30:00".to_string(),
        audio_start_time: seq as f64,
        audio_end_time: seq as f64 + 1.0,
        duration: 1.0,
        display_time: format!("[00:{:02}]", seq),
        confidence: 0.95,
        sequence_id: seq,
        chunk_start_time: None,
        is_partial: None,
    }
}

// ─── JSON Compatibility Tests ───────────────────────────────────

#[test]
fn test_transcript_segment_serde_frontend_compatible() {
    // Simulates EXACTLY what the frontend sends to save_meeting
    let json = r#"[
        {
            "id": "seg_1",
            "text": "hello world",
            "timestamp": "14:30:05",
            "audioStartTime": 1.5,
            "audioEndTime": 2.5,
            "duration": 1.0,
            "isPartial": true,
            "chunkStartTime": 0.0,
            "sequenceId": 1
        },
        {
            "id": "seg_2",
            "text": "second segment",
            "timestamp": "14:30:06",
            "sequenceId": 2
        }
    ]"#;

    let segments: Vec<app_lib::api::TranscriptSegment> =
        serde_json::from_str(json).expect("Should deserialize frontend Transcript JSON");

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].text, "hello world");
    assert_eq!(segments[0].audio_start_time, 1.5);
    assert_eq!(segments[0].chunk_start_time, Some(0.0));
    assert_eq!(segments[0].is_partial, Some(true));
    // Optional fields default to 0
    assert_eq!(segments[1].audio_start_time, 0.0);
    assert_eq!(segments[1].duration, 0.0);
}

#[test]
fn test_save_meeting_response_snake_case() {
    let response = app_lib::api::SaveMeetingResponse {
        meeting_id: "meeting-test-123".to_string(),
    };
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(
        parsed.get("meeting_id").is_some(),
        "Must have snake_case 'meeting_id', got: {}", json
    );
}

#[test]
fn test_save_meeting_response_no_camel_case() {
    let response = app_lib::api::SaveMeetingResponse {
        meeting_id: "meeting-test-123".to_string(),
    };
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.get("meetingId").is_none(), "Must NOT have camelCase 'meetingId'");
}

// ─── Database CRUD Tests ────────────────────────────────────────

#[tokio::test]
async fn test_save_transcript_creates_meeting() {
    use app_lib::database::repositories::transcript::TranscriptsRepository;

    let pool = setup_test_db().await;
    let segments = vec![make_segment("Hello, this is a test.", 1)];

    let meeting_id = TranscriptsRepository::save_transcript(
        &pool, "Test Meeting", &segments, Some("/tmp/test".to_string()),
    )
    .await
    .expect("save_transcript should succeed");

    assert!(meeting_id.starts_with("meeting-"));
}

#[tokio::test]
async fn test_get_meeting_after_save() {
    use app_lib::database::repositories::transcript::TranscriptsRepository;
    use app_lib::database::repositories::meeting::MeetingsRepository;

    let pool = setup_test_db().await;
    let segments = vec![make_segment("Readback.", 1)];

    let meeting_id = TranscriptsRepository::save_transcript(
        &pool, "Readback Test", &segments, Some("/tmp/rb".to_string()),
    )
    .await
    .unwrap();

    let meeting = MeetingsRepository::get_meeting(&pool, &meeting_id)
        .await
        .unwrap()
        .expect("Meeting should exist");

    assert_eq!(meeting.title, "Readback Test");
    assert_eq!(meeting.transcripts.len(), 1);
    assert_eq!(meeting.transcripts[0].text, "Readback.");
    assert_eq!(meeting.folder_path, Some("/tmp/rb".to_string()));
}

#[tokio::test]
async fn test_delete_meeting_returns_true() {
    use app_lib::database::repositories::transcript::TranscriptsRepository;
    use app_lib::database::repositories::meeting::MeetingsRepository;

    let pool = setup_test_db().await;
    let segments = vec![make_segment("Delete me.", 1)];

    let meeting_id = TranscriptsRepository::save_transcript(
        &pool, "To Delete", &segments, None,
    )
    .await
    .unwrap();

    let deleted = MeetingsRepository::delete_meeting(&pool, &meeting_id)
        .await
        .expect("delete_meeting should not error");
    assert!(deleted, "Should return true for existing meeting");
}

#[tokio::test]
async fn test_delete_nonexistent_returns_false() {
    use app_lib::database::repositories::meeting::MeetingsRepository;

    let pool = setup_test_db().await;
    let deleted = MeetingsRepository::delete_meeting(&pool, "meeting-ghost").await.unwrap();
    assert!(!deleted, "Should return false for nonexistent meeting");
}
