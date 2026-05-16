-- Migration: Add OpenAI-Compatible transcription provider fields
-- Adds support for custom OpenAI-compatible transcription server endpoints

ALTER TABLE transcript_settings ADD COLUMN openaiCompatibleEndpoint TEXT;
ALTER TABLE transcript_settings ADD COLUMN openaiCompatibleApiKey TEXT;