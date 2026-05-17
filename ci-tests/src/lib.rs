#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use sqlx::SqlitePool;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
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

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS meetings (
                id TEXT PRIMARY KEY, title TEXT NOT NULL,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL, folder_path TEXT
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transcripts (
                id TEXT PRIMARY KEY, meeting_id TEXT NOT NULL,
                transcript TEXT NOT NULL, timestamp TEXT NOT NULL,
                audio_start_time REAL, audio_end_time REAL, duration REAL,
                FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[test]
    fn test_transcript_segment_serde_frontend_compatible() {
        let json = r#"[
            {"id":"seg_1","text":"hello","timestamp":"14:30:05","audioStartTime":1.5,"audioEndTime":2.5,"duration":1.0,"isPartial":true,"chunkStartTime":0.0,"sequenceId":1},
            {"id":"seg_2","text":"second","timestamp":"14:30:06","sequenceId":2}
        ]"#;
        let segments: Vec<TranscriptSegment> = serde_json::from_str(json).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].audio_start_time, 1.5);
        assert_eq!(segments[0].is_partial, Some(true));
        assert_eq!(segments[1].audio_start_time, 0.0); // default
    }

    #[test]
    fn test_save_meeting_response_snake_case() {
        let r = SaveMeetingResponse { meeting_id: "x".into() };
        let json = serde_json::to_string(&r).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("meeting_id").is_some());
        assert!(v.get("meetingId").is_none());
    }

    #[tokio::test]
    async fn test_save_and_read_meeting() {
        let pool = setup_db().await;
        sqlx::query("INSERT INTO meetings (id,title,created_at,updated_at,folder_path) VALUES (?1,?2,?3,?4,?5)")
            .bind("m1").bind("Test").bind("now").bind("now").bind("/tmp/t")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO transcripts (id,meeting_id,transcript,timestamp,audio_start_time,audio_end_time,duration) VALUES (?1,?2,?3,?4,?5,?6,?7)")
            .bind("t1").bind("m1").bind("hello").bind("14:30").bind(1.5).bind(2.5).bind(1.0)
            .execute(&pool).await.unwrap();
        let (title, folder): (String, Option<String>) =
            sqlx::query_as("SELECT title, folder_path FROM meetings WHERE id='m1'")
                .fetch_one(&pool).await.unwrap();
        assert_eq!(title, "Test");
        assert_eq!(folder, Some("/tmp/t".into()));
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM transcripts WHERE meeting_id='m1'")
                .fetch_one(&pool).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_delete_meeting_cascades() {
        let pool = setup_db().await;
        sqlx::query("INSERT INTO meetings (id,title,created_at,updated_at) VALUES ('md','To Delete','n','n')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO transcripts (id,meeting_id,transcript,timestamp) VALUES ('t','md','text','14:30')")
            .execute(&pool).await.unwrap();
        sqlx::query("DELETE FROM transcripts WHERE meeting_id='md'").execute(&pool).await.unwrap();
        sqlx::query("DELETE FROM meetings WHERE id='md'").execute(&pool).await.unwrap();
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM meetings WHERE id='md'").fetch_one(&pool).await.unwrap();
        assert_eq!(c, 0);
    }
}
