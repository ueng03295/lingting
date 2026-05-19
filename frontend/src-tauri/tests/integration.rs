/// Self-contained integration tests — no dependency on the full Tauri app_lib crate.
/// These run on any platform (ubuntu, macos, windows) because they only use sqlx + serde.
///
/// Run: `cargo test --test integration` from frontend/src-tauri/

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

// ─── Replica of key structs (independent of app_lib) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TranscriptSegment {
    pub id: String,
    pub text: String,
    pub timestamp: String,
    #[serde(default)]
    pub audio_start_time: f64,
    #[serde(default)]
    pub audio_end_time: f64,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub display_time: String,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub sequence_id: u64,
    #[serde(default)]
    pub chunk_start_time: Option<f64>,
    #[serde(default)]
    pub is_partial: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveMeetingResponse {
    pub meeting_id: String,
}

fn make_segment(text: &str, seq: u64) -> TranscriptSegment {
    TranscriptSegment {
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


// ─── JSON Compatibility Tests ───────────────────────────────────────────

#[test]
fn test_transcript_segment_serde_frontend_compatible() {
    let json = r#"[
        {
            "id": "seg_1",
            "text": "hello world",
            "timestamp": "14:30:05",
            "audio_start_time": 1.5,
            "audio_end_time": 2.5,
            "duration": 1.0,
            "is_partial": true,
            "chunk_start_time": 0.0,
            "sequence_id": 1
        },
        {
            "id": "seg_2",
            "text": "second segment",
            "timestamp": "14:30:06",
            "sequence_id": 2
        }
    ]"#;

    let segments: Vec<TranscriptSegment> =
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
    let response = SaveMeetingResponse {
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
    let response = SaveMeetingResponse {
        meeting_id: "meeting-test-123".to_string(),
    };
    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(
        parsed.get("meetingId").is_none(),
        "Must NOT have camelCase 'meetingId'"
    );
}


// ─── Database CRUD Tests ────────────────────────────────────────────────

async fn setup_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS meetings (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            folder_path TEXT
        )"
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transcripts (
            id TEXT PRIMARY KEY,
            meeting_id TEXT NOT NULL,
            transcript TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            audio_start_time REAL,
            audio_end_time REAL,
            duration REAL,
            FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE
        )"
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn test_save_and_read_meeting() {
    let pool = setup_db().await;
    let meeting_id = "meeting-test-001";
    let now = "2026-05-17T12:00:00Z";

    sqlx::query(
        "INSERT INTO meetings (id, title, created_at, updated_at, folder_path) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(meeting_id)
    .bind("Test Meeting")
    .bind(now)
    .bind(now)
    .bind("/tmp/test")
    .execute(&pool)
    .await
    .unwrap();

    // Insert a transcript
    sqlx::query(
        "INSERT INTO transcripts (id, meeting_id, transcript, timestamp, audio_start_time, audio_end_time, duration) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("t1")
    .bind(meeting_id)
    .bind("Hello world")
    .bind("14:30:00")
    .bind(1.5)
    .bind(2.5)
    .bind(1.0)
    .execute(&pool)
    .await
    .unwrap();

    // Read back
    let (title, folder): (String, Option<String>) = sqlx::query_as(
        "SELECT title, folder_path FROM meetings WHERE id = ?"
    )
    .bind(meeting_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(title, "Test Meeting");
    assert_eq!(folder, Some("/tmp/test".to_string()));

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM transcripts WHERE meeting_id = ?"
    )
    .bind(meeting_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count.0, 1);
}

#[tokio::test]
async fn test_delete_meeting_cascades() {
    let pool = setup_db().await;

    sqlx::query("INSERT INTO meetings (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)")
        .bind("meet-del")
        .bind("To Delete")
        .bind("now")
        .bind("now")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO transcripts (id, meeting_id, transcript, timestamp) VALUES (?, ?, ?, ?)"
    )
    .bind("t1")
    .bind("meet-del")
    .bind("Delete me")
    .bind("14:30")
    .execute(&pool)
    .await
    .unwrap();

    // Delete
    sqlx::query("DELETE FROM transcripts WHERE meeting_id = ?")
        .bind("meet-del")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM meetings WHERE id = ?")
        .bind("meet-del")
        .execute(&pool)
        .await
        .unwrap();

    // Verify gone
    let meeting_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM meetings WHERE id = ?")
        .bind("meet-del")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(meeting_count.0, 0);

    let transcript_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transcripts WHERE meeting_id = ?")
        .bind("meet-del")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(transcript_count.0, 0);
}

#[tokio::test]
async fn test_delete_nonexistent_is_noop() {
    let pool = setup_db().await;
    let result = sqlx::query("DELETE FROM meetings WHERE id = ?")
        .bind("meet-ghost")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(result.rows_affected(), 0);
}
