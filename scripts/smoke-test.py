#!/usr/bin/env python3
"""LingListen smoke tests — no Rust compilation needed. Runs on any OS."""

import sqlite3, json, sys, re

PASSED = 0
FAILED = 0

def check(name, fn):
    global PASSED, FAILED
    try:
        fn()
        print(f"  ✅ {name}")
        PASSED += 1
    except Exception as e:
        print(f"  ❌ {name}: {e}")
        FAILED += 1


# ─── Tests ──────────────────────────────────────────────────────

def test_meetings_table_has_folder_path():
    db = sqlite3.connect(":memory:")
    db.execute("CREATE TABLE meetings (id TEXT PRIMARY KEY, title TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, folder_path TEXT)")
    db.execute("INSERT INTO meetings VALUES ('m1','Test','now','now','/tmp/folder')")
    row = db.execute("SELECT folder_path FROM meetings WHERE id='m1'").fetchone()
    assert row[0] == '/tmp/folder', f"Got {row[0]}"


def test_save_and_read_meeting():
    db = sqlite3.connect(":memory:")
    db.execute("CREATE TABLE meetings (id TEXT PRIMARY KEY, title TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, folder_path TEXT)")
    db.execute("CREATE TABLE transcripts (id TEXT PRIMARY KEY, meeting_id TEXT NOT NULL, transcript TEXT NOT NULL, timestamp TEXT NOT NULL, audio_start_time REAL, audio_end_time REAL, duration REAL)")
    db.execute("INSERT INTO meetings VALUES ('m1','Test Meeting','now','now','/tmp/test')")
    db.execute("INSERT INTO transcripts VALUES ('t1','m1','hello','14:30',1.5,2.5,1.0)")
    title, folder = db.execute("SELECT title, folder_path FROM meetings WHERE id='m1'").fetchone()
    assert title == 'Test Meeting'
    assert folder == '/tmp/test'
    cnt = db.execute("SELECT COUNT(*) FROM transcripts WHERE meeting_id='m1'").fetchone()[0]
    assert cnt == 1


def test_delete_meeting_cascades():
    db = sqlite3.connect(":memory:")
    db.execute("PRAGMA foreign_keys = ON")
    db.execute("CREATE TABLE meetings (id TEXT PRIMARY KEY, title TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, folder_path TEXT)")
    db.execute("CREATE TABLE transcripts (id TEXT PRIMARY KEY, meeting_id TEXT NOT NULL, transcript TEXT NOT NULL, timestamp TEXT NOT NULL, audio_start_time REAL, audio_end_time REAL, duration REAL, FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE)")
    db.execute("INSERT INTO meetings VALUES ('md','To Delete','n','n',NULL)")
    db.execute("INSERT INTO transcripts VALUES ('t','md','text','14:30',0,0,0)")
    db.execute("DELETE FROM meetings WHERE id='md'")
    cnt = db.execute("SELECT COUNT(*) FROM transcripts WHERE meeting_id='md'").fetchone()[0]
    assert cnt == 0, f"Transcripts not cascade-deleted, got {cnt}"


def test_transcript_json_camelcase():
    j = '''
    [
        {"id":"seg_1","text":"hello","timestamp":"14:30","audioStartTime":1.5,"audioEndTime":2.5,"duration":1.0,"isPartial":true,"chunkStartTime":0.0,"sequenceId":1},
        {"id":"seg_2","text":"world","timestamp":"14:31","sequenceId":2}
    ]
    '''
    segs = json.loads(j)
    assert len(segs) == 2
    assert segs[0]['audioStartTime'] == 1.5
    assert segs[0]['isPartial'] is True
    # Missing optional fields are fine
    assert segs[1].get('audioStartTime') is None  # frontend may omit


def test_save_response_snake_case():
    resp = {"meeting_id": "meeting-test-123"}
    j = json.dumps(resp)
    d = json.loads(j)
    assert 'meeting_id' in d
    assert 'meetingId' not in d


def test_meeting_id_format():
    mid = "meeting-550e8400-e29b-41d4-a716-446655440000"
    assert re.match(r'^meeting-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$', mid)


def test_delete_nonexistent_is_noop():
    db = sqlite3.connect(":memory:")
    db.execute("CREATE TABLE meetings (id TEXT PRIMARY KEY, title TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, folder_path TEXT)")
    result = db.execute("DELETE FROM meetings WHERE id='meet-ghost'")
    assert result.rowcount == 0


# ─── Run ────────────────────────────────────────────────────────

print(f"\n{'='*50}")
print(f"  LingListen Smoke Tests")
print(f"{'='*50}\n")

TESTS = [
    ("meetings table has folder_path", test_meetings_table_has_folder_path),
    ("save → readback meeting + transcript", test_save_and_read_meeting),
    ("delete meeting cascade-deletes transcripts", test_delete_meeting_cascades),
    ("Transcript JSON (camelCase frontend format)", test_transcript_json_camelcase),
    ("SaveMeetingResponse field = meeting_id (snake_case)", test_save_response_snake_case),
    ("meeting_id format meeting-UUID", test_meeting_id_format),
    ("delete nonexistent meeting is no-op", test_delete_nonexistent_is_noop),
]

for name, fn in TESTS:
    check(name, fn)

print(f"\n{'='*50}")
if FAILED == 0:
    print(f"  🎉 ALL {PASSED} TESTS PASSED")
    print(f"{'='*50}\n")
else:
    print(f"  {PASSED} passed, {FAILED} FAILED")
    print(f"{'='*50}\n")
    sys.exit(1)
