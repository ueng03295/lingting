import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Input } from './ui/input';
import { Button } from './ui/button';
import { Label } from './ui/label';
import { Eye, EyeOff, Loader2, CheckCircle2, XCircle, Server, MemoryStick, Download } from 'lucide-react';
import { configService } from '@/services/configService';
import { useConfig } from '@/contexts/ConfigContext';
import { LanguageSelection } from './LanguageSelection';


export interface TranscriptModelProps {
    provider: 'openaiCompatible';
    model: string;
    apiKey?: string | null;
    openaiCompatibleEndpoint?: string | null;
    openaiCompatibleApiKey?: string | null;
}

export interface TranscriptSettingsProps {
    transcriptModelConfig: TranscriptModelProps;
    setTranscriptModelConfig: (config: TranscriptModelProps) => void;
    onModelSelect?: () => void;
}

export function TranscriptSettings({ transcriptModelConfig, setTranscriptModelConfig, onModelSelect }: TranscriptSettingsProps) {
    const { selectedLanguage } = useConfig();

    // OpenAI-Compatible (qwen3-asr) state
    const [openaiCompatibleEndpoint, setOpenaiCompatibleEndpoint] = useState<string>(
        transcriptModelConfig.openaiCompatibleEndpoint || 'http://localhost:8765'
    );
    const [openaiCompatibleApiKey, setOpenaiCompatibleApiKey] = useState<string>(
        transcriptModelConfig.openaiCompatibleApiKey || ''
    );
    const [openaiCompatibleModel, setOpenaiCompatibleModel] = useState<string>(
        transcriptModelConfig.model || 'qwen3-asr-1.7b'
    );
    const [showApiKey, setShowApiKey] = useState<boolean>(false);
    const [availableModels, setAvailableModels] = useState<string[]>([]);
    const [isTestingConnection, setIsTestingConnection] = useState<boolean>(false);
    const [connectionStatus, setConnectionStatus] = useState<'idle' | 'success' | 'error'>('idle');
    const [connectionMessage, setConnectionMessage] = useState<string>('');

    // ASR model management state
    const [asrStatus, setAsrStatus] = useState<{ loaded: boolean; model_id: string | null; language: string; available_models: any[] } | null>(null);
    const [isLoadingModel, setIsLoadingModel] = useState(false);
    const [isUnloadingModel, setIsUnloadingModel] = useState(false);
    const [asrLanguage, setAsrLanguage] = useState<string>('zh');

    // Auto-save transcript config to backend when it changes
    const prevConfigRef = useRef<string>('');
    useEffect(() => {
        const configKey = JSON.stringify({
            provider: transcriptModelConfig.provider,
            model: transcriptModelConfig.model,
            openaiCompatibleEndpoint: transcriptModelConfig.openaiCompatibleEndpoint,
            openaiCompatibleApiKey: transcriptModelConfig.openaiCompatibleApiKey,
        });
        if (prevConfigRef.current && prevConfigRef.current !== configKey) {
            const save = async () => {
                try {
                    await invoke('api_save_transcript_config', {
                        provider: transcriptModelConfig.provider,
                        model: transcriptModelConfig.model,
                        apiKey: transcriptModelConfig.apiKey ?? null,
                        openaiCompatibleEndpoint: transcriptModelConfig.openaiCompatibleEndpoint ?? null,
                        openaiCompatibleApiKey: transcriptModelConfig.openaiCompatibleApiKey ?? null,
                    });
                } catch (error) {
                    console.error('Failed to auto-save transcript config:', error);
                }
            };
            save();
        }
        prevConfigRef.current = configKey;
    }, [transcriptModelConfig.provider, transcriptModelConfig.model, transcriptModelConfig.openaiCompatibleEndpoint, transcriptModelConfig.openaiCompatibleApiKey]);

    // Sync OpenAI-Compatible state from config
    useEffect(() => {
        if (transcriptModelConfig.openaiCompatibleEndpoint) {
            setOpenaiCompatibleEndpoint(transcriptModelConfig.openaiCompatibleEndpoint);
        }
        if (transcriptModelConfig.openaiCompatibleApiKey) {
            setOpenaiCompatibleApiKey(transcriptModelConfig.openaiCompatibleApiKey);
        }
        if (transcriptModelConfig.model) {
            setOpenaiCompatibleModel(transcriptModelConfig.model);
        }
    }, [transcriptModelConfig.openaiCompatibleEndpoint, transcriptModelConfig.openaiCompatibleApiKey, transcriptModelConfig.model]);

    const handleTestConnection = async () => {
        setIsTestingConnection(true);
        setConnectionStatus('idle');
        setConnectionMessage('');
        try {
            const result = await configService.testOpenAICompatibleConnection(
                openaiCompatibleEndpoint,
                openaiCompatibleApiKey || null
            );
            setConnectionStatus('success');
            setConnectionMessage(result.message || 'Connection successful');
            if (result.models && result.models.length > 0) {
                setAvailableModels(result.models);
            }
            fetchAsrStatus();
        } catch (err: any) {
            setConnectionStatus('error');
            setConnectionMessage(err?.toString() || 'Connection failed');
        } finally {
            setIsTestingConnection(false);
        }
    };

    // Fetch ASR server status
    const fetchAsrStatus = useCallback(async () => {
        const endpoint = openaiCompatibleEndpoint || transcriptModelConfig.openaiCompatibleEndpoint || '';
        if (!endpoint) return;
        try {
            const url = `${endpoint.replace(/\/+$/, '')}/v1/asr/status`;
            const response = await fetch(url, { signal: AbortSignal.timeout(5000) });
            if (response.ok) {
                const data = await response.json();
                setAsrStatus(data);
                setAsrLanguage(data.language || 'zh');
            }
        } catch {
            setAsrStatus(null);
        }
    }, [openaiCompatibleEndpoint, transcriptModelConfig.openaiCompatibleEndpoint]);

    // Load ASR model into memory
    const handleLoadModel = async () => {
        setIsLoadingModel(true);
        const endpoint = openaiCompatibleEndpoint || transcriptModelConfig.openaiCompatibleEndpoint || '';
        try {
            const url = `${endpoint.replace(/\/+$/, '')}/v1/asr/load`;
            await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ model: openaiCompatibleModel || 'qwen3-asr-1.7b' }),
            });
            await fetchAsrStatus();
        } catch { /* ignore */ } finally {
            setIsLoadingModel(false);
        }
    };

    // Unload ASR model from memory
    const handleUnloadModel = async () => {
        setIsUnloadingModel(true);
        const endpoint = openaiCompatibleEndpoint || transcriptModelConfig.openaiCompatibleEndpoint || '';
        try {
            const url = `${endpoint.replace(/\/+$/, '')}/v1/asr/unload`;
            await fetch(url, { method: 'POST' });
            await fetchAsrStatus();
        } catch { /* ignore */ } finally {
            setIsUnloadingModel(false);
        }
    };

    // Sync global language preference to ASR server
    useEffect(() => {
        if (selectedLanguage) {
            const endpoint = openaiCompatibleEndpoint || transcriptModelConfig.openaiCompatibleEndpoint || '';
            if (!endpoint) return;
            const url = `${endpoint.replace(/\/+$/, '')}/v1/asr/language`;
            fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ language: selectedLanguage }),
            }).then(() => setAsrLanguage(selectedLanguage)).catch(() => {});
        }
    }, [selectedLanguage, openaiCompatibleEndpoint, transcriptModelConfig.openaiCompatibleEndpoint]);

    // Auto-fetch ASR status on mount
    useEffect(() => {
        fetchAsrStatus();
    }, [fetchAsrStatus]);

    return (
        <div>
            <div>
                <div className="space-y-4 pb-6">
                    <div className="space-y-3 p-3 bg-gray-50 rounded-lg border border-gray-200">
                        <div className="flex items-center gap-2 mb-1">
                            <Server className="h-4 w-4 text-gray-600" />
                            <Label className="text-sm font-medium text-gray-900">qwen3-asr 转写引擎</Label>
                        </div>

                        <div>
                            <Label className="block text-sm font-medium text-gray-700 mb-1">
                                Server URL
                            </Label>
                            <Input
                                type="text"
                                className="focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
                                value={openaiCompatibleEndpoint}
                                onChange={(e) => {
                                    setOpenaiCompatibleEndpoint(e.target.value);
                                    setConnectionStatus('idle');
                                    setTranscriptModelConfig({
                                        ...transcriptModelConfig,
                                        provider: 'openaiCompatible',
                                        openaiCompatibleEndpoint: e.target.value || null,
                                    });
                                }}
                                placeholder="http://localhost:8765"
                            />
                        </div>

                        <div>
                            <Label className="block text-sm font-medium text-gray-700 mb-1">
                                API Key（可选）
                            </Label>
                            <div className="relative">
                                <Input
                                    type={showApiKey ? "text" : "password"}
                                    className="pr-10 focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
                                    value={openaiCompatibleApiKey}
                                    onChange={(e) => {
                                        setOpenaiCompatibleApiKey(e.target.value);
                                        setTranscriptModelConfig({
                                            ...transcriptModelConfig,
                                            provider: 'openaiCompatible',
                                            openaiCompatibleApiKey: e.target.value || null,
                                        });
                                    }}
                                    placeholder="Optional API key"
                                />
                                <div className="absolute inset-y-0 right-0 pr-1 flex items-center">
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="icon"
                                        onClick={() => setShowApiKey(!showApiKey)}
                                    >
                                        {showApiKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                                    </Button>
                                </div>
                            </div>
                        </div>

                        <div>
                            <Label className="block text-sm font-medium text-gray-700 mb-1">
                                Model Name
                            </Label>
                            {availableModels.length > 0 ? (
                                <Select
                                    value={openaiCompatibleModel}
                                    onValueChange={(value) => {
                                        setOpenaiCompatibleModel(value);
                                        setTranscriptModelConfig({
                                            ...transcriptModelConfig,
                                            provider: 'openaiCompatible',
                                            model: value,
                                        });
                                    }}
                                >
                                    <SelectTrigger className="focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
                                        <SelectValue placeholder="Select model" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {availableModels.map((model) => (
                                            <SelectItem key={model} value={model}>{model}</SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            ) : (
                                <Input
                                    type="text"
                                    className="focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
                                    value={openaiCompatibleModel}
                                    onChange={(e) => {
                                        setOpenaiCompatibleModel(e.target.value);
                                        setTranscriptModelConfig({
                                            ...transcriptModelConfig,
                                            provider: 'openaiCompatible',
                                            model: e.target.value,
                                        });
                                    }}
                                    placeholder="qwen3-asr-1.7b"
                                />
                            )}
                        </div>

                        <div className="flex items-center gap-2">
                            <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                onClick={handleTestConnection}
                                disabled={isTestingConnection || !openaiCompatibleEndpoint}
                            >
                                {isTestingConnection ? (
                                    <>
                                        <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                                        Testing...
                                    </>
                                ) : (
                                    'Test Connection'
                                )}
                            </Button>
                            {connectionStatus === 'success' && (
                                <div className="flex items-center gap-1 text-green-600 text-sm">
                                    <CheckCircle2 className="h-4 w-4" />
                                    <span>{connectionMessage}</span>
                                </div>
                            )}
                            {connectionStatus === 'error' && (
                                <div className="flex items-center gap-1 text-red-600 text-sm">
                                    <XCircle className="h-4 w-4" />
                                    <span>{connectionMessage}</span>
                                </div>
                            )}
                        </div>

                        {/* ASR Model Management */}
                        {asrStatus && (
                            <div className="mt-3 p-3 bg-white rounded-lg border border-gray-200">
                                <div className="flex items-center gap-2 mb-2">
                                    <Server className="h-4 w-4 text-gray-600" />
                                    <span className="text-sm font-medium text-gray-900">ASR 服务器状态</span>
                                </div>
                                
                                <div className="flex items-center gap-2 mb-3 text-sm">
                                    <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${asrStatus.loaded ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'}`}>
                                        <span className={`w-1.5 h-1.5 rounded-full ${asrStatus.loaded ? 'bg-green-500' : 'bg-gray-400'}`} />
                                        {asrStatus.loaded ? `已加载: ${asrStatus.model_id}` : '未加载'}
                                    </span>
                                </div>

                                <div className="flex gap-2 mb-3">
                                    <Button
                                        type="button"
                                        variant="outline"
                                        size="sm"
                                        onClick={handleLoadModel}
                                        disabled={isLoadingModel || asrStatus.loaded}
                                        className="text-xs"
                                    >
                                        {isLoadingModel ? (
                                            <><Loader2 className="h-3 w-3 mr-1 animate-spin" />加载中...</>
                                        ) : (
                                            <><Download className="h-3 w-3 mr-1" />加载模型</>
                                        )}
                                    </Button>
                                    <Button
                                        type="button"
                                        variant="outline"
                                        size="sm"
                                        onClick={handleUnloadModel}
                                        disabled={isUnloadingModel || !asrStatus.loaded}
                                        className="text-xs"
                                    >
                                        {isUnloadingModel ? (
                                            <><Loader2 className="h-3 w-3 mr-1 animate-spin" />卸载中...</>
                                        ) : (
                                            <><MemoryStick className="h-3 w-3 mr-1" />卸载模型</>
                                        )}
                                    </Button>
                                </div>

                                {asrStatus.loaded && (
                                    <div className="text-xs text-gray-500">
                                        <span className="flex items-center gap-1">
                                            <MemoryStick className="h-3 w-3" />
                                            {asrStatus.available_models?.find((m: any) => m.id === asrStatus.model_id)?.size_gb || '?'} GB 内存占用 — 不录音时可卸载释放内存
                                        </span>
                                    </div>
                                )}
                            </div>
                        )}

                        {/* Language sync status */}
                        {asrStatus && asrLanguage !== selectedLanguage && (
                            <div className="mt-2 p-2 bg-blue-50 border border-blue-200 rounded text-xs text-blue-700 flex items-center gap-1">
                                <Server className="h-3 w-3" />
                                正在同步语言 ({selectedLanguage}) 到服务器...
                            </div>
                        )}
                        {asrStatus && asrLanguage === selectedLanguage && (
                            <div className="mt-2 text-xs text-gray-500 flex items-center gap-1">
                                <CheckCircle2 className="h-3 w-3 text-green-500" />
                                服务器语言: {asrLanguage}
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    )
}