// Analytics stub — all tracking disabled for LingListen fork
// No data is collected or sent anywhere.

export interface AnalyticsProperties {
  [key: string]: string;
}

class AnalyticsStub {
  async init(): Promise<void> { /* no-op */ }
  async track(_event: string, _properties?: AnalyticsProperties): Promise<void> { /* no-op */ }
  async identify(_userId: string, _traits?: Record<string, string>): Promise<void> { /* no-op */ }
  async disable(): Promise<void> { /* no-op */ }
  async enable(): Promise<void> { /* no-op */ }
  async isAnalyticsEnabled(): Promise<boolean> { return false; }
  async trackSettingsChanged(_category: string, _value: string): Promise<void> { /* no-op */ }
  async trackFeatureUsed(_feature: string): Promise<void> { /* no-op */ }
  async trackMeetingStarted(): Promise<void> { /* no-op */ }
  async trackRecordingStarted(): Promise<void> { /* no-op */ }
  async trackRecordingStopped(): Promise<void> { /* no-op */ }
  async trackMeetingDeleted(_meetingId?: string): Promise<void> { /* no-op */ }
  async trackDailyActiveUser(): Promise<void> { /* no-op */ }
  async trackUserFirstLaunch(): Promise<void> { /* no-op */ }
  async trackSummaryGenerationStarted(_provider?: string, _model?: string, _transcriptLength?: number, _timeSince?: number): Promise<void> { /* no-op */ }
  async trackSummaryGenerationCompleted(_provider?: string, _model?: string, _isCached?: boolean, _duration?: number, _error?: string): Promise<void> { /* no-op */ }
  async trackSummaryGenerationError(_error: string): Promise<void> { /* no-op */ }
  async trackTranscriptionStarted(): Promise<void> { /* no-op */ }
  async trackTranscriptionCompleted(): Promise<void> { /* no-op */ }
  async trackTranscriptionError(_error: string): Promise<void> { /* no-op */ }
  async trackImportStarted(): Promise<void> { /* no-op */ }
  async trackImportCompleted(): Promise<void> { /* no-op */ }
  async trackModelError(_error: string): Promise<void> { /* no-op */ }
  async trackLanguageSelected(_language: string): Promise<void> { /* no-op */ }
  async trackModelDownloadStarted(_model: string): Promise<void> { /* no-op */ }
  async trackModelDownloadCompleted(_model: string): Promise<void> { /* no-op */ }
  async trackModelDownloadError(_model: string, _error: string): Promise<void> { /* no-op */ }
  async trackApiKeyValidation(_provider: string, _success: boolean): Promise<void> { /* no-op */ }
  async startSession(): Promise<void> { /* no-op */ }
  async endSession(): Promise<void> { /* no-op */ }
  async isSessionActive(): Promise<boolean> { return false; }
  async trackPageView(_page: string): Promise<void> { /* no-op */ }
  async trackImportError(_error: string): Promise<void> { /* no-op */ }
  async trackExportCompleted(_format: string): Promise<void> { /* no-op */ }
  async trackKeyboardShortcut(_shortcut: string): Promise<void> { /* no-op */ }
  async trackBackendConnection(_success: boolean, _provider?: string, _latencyMs?: number): Promise<void> { /* no-op */ }
  async trackButtonClick(_button: string, _context?: string): Promise<void> { /* no-op */ }
  async trackCopy(_type: string, _properties?: Record<string, string>): Promise<void> { /* no-op */ }
  async trackCustomPromptUsed(_length?: number): Promise<void> { /* no-op */ }
  async trackError(_error: string, _detail?: string): Promise<void> { /* no-op */ }
  async trackMeetingCompleted(_meetingId?: string, _properties?: Record<string, string | number>): Promise<void> { /* no-op */ }
  async trackModelChanged(_oldModel: string, _newModel?: string, _oldProvider?: string, _newProvider?: string): Promise<void> { /* no-op */ }
  async trackTranscriptionSuccess(): Promise<void> { /* no-op */ }
  async getMeetingsCountToday(): Promise<number> { return 0; }
  async updateMeetingCount(): Promise<void> { /* no-op */ }
  async calculateDaysSince(_key: string): Promise<number> { return 0; }
}

const Analytics = new AnalyticsStub();
export default Analytics;