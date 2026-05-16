import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Input } from './ui/input';
import { Button } from './ui/button';
import { Label } from './ui/label';
import { Eye, EyeOff, Lock, Unlock, Loader2, CheckCircle2, XCircle, Server, MemoryStick, Download } from 'lucide-react';
import { ModelManager } from './WhisperModelManager';
import { ParakeetModelManager } from './ParakeetModelManager';
import { configService } from '@/services/configService';
import { useConfig } from '@/contexts/ConfigContext';
import { LanguageSelection } from './LanguageSelection';


export interface TranscriptModelProps {
    provider: 'localWhisper' | 'parakeet' | 'deepgram' | 'elevenLabs' | 'groq' | 'openai' | 'openaiCompatible';
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
    const [apiKey, setApiKey] = useState<string | null>(transcriptModelConfig.apiKey || null);
    const [showApiKey, setShowApiKey] = useState<boolean>(false);
    const [isApiKeyLocked, setIsApiKeyLocked] = useState<boolean>(true);
    const [isLockButtonVibrating, setIsLockButtonVibrating] = useState<boolean>(false);
    const [uiProvider, setUiProvider] = useState<TranscriptModelProps['provider']>(transcriptModelConfig.provider);

    // OpenAI-Compatible specific state
    const [openaiCompatibleEndpoint, setOpenaiCompatibleEndpoint] = useState<string>(
        transcriptModelConfig.openaiCompatibleEndpoint || 'http://localhost:8765'
    );
    const [openaiCompatibleApiKey, setOpenaiCompatibleApiKey] = useState<string>(
        transcriptModelConfig.openaiCompatibleApiKey || ''
    );
    const [openaiCompatibleModel, setOpenaiCompatibleModel] = useState<string>(
        transcriptModelConfig.provider === 'openaiCompatible' ? transcriptModelConfig.model : 'qwen3-asr-1.7b'
    );
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
        // Serialize current config to detect meaningful changes
        const configKey = JSON.stringify({
            provider: transcriptModelConfig.provider,
            model: transcriptModelConfig.model,
            openaiCompatibleEndpoint: transcriptModelConfig.openaiCompatibleEndpoint,
            openaiCompatibleApiKey: transcriptModelConfig.openaiCompatibleApiKey,
        });
        // Skip initial render and non-meaningful changes
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
                    console.log('Auto-saved transcript config:', transcriptModelConfig.provider, transcriptModelConfig.model);
                } catch (error) {
                    console.error('Failed to auto-save transcript config:', error);
                }
            };
            save();
        }
        prevConfigRef.current = configKey;
    }, [transcriptModelConfig.provider, transcriptModelConfig.model, transcriptModelConfig.openaiCompatibleEndpoint, transcriptModelConfig.openaiCompatibleApiKey]);

    // Sync uiProvider when backend config changes (e.g., after model selection or initial load)
    useEffect(() => {
        setUiProvider(transcriptModelConfig.provider);
    }, [transcriptModelConfig.provider]);

    // Sync OpenAI-Compatible state from config
    useEffect(() => {
        if (transcriptModelConfig.provider === 'openaiCompatible') {
            if (transcriptModelConfig.openaiCompatibleEndpoint) {
                setOpenaiCompatibleEndpoint(transcriptModelConfig.openaiCompatibleEndpoint);
            }
            if (transcriptModelConfig.openaiCompatibleApiKey) {
                setOpenaiCompatibleApiKey(transcriptModelConfig.openaiCompatibleApiKey);
            }
            if (transcriptModelConfig.model) {
                setOpenaiCompatibleModel(transcriptModelConfig.model);
            }
        }
    }, [transcriptModelConfig.openaiCompatibleEndpoint, transcriptModelConfig.openaiCompatibleApiKey, transcriptModelConfig.model, transcriptModelConfig.provider]);

    useEffect(() => {
        if (transcriptModelConfig.provider === 'localWhisper' || transcriptModelConfig.provider === 'parakeet') {
            setApiKey(null);
        }
    }, [transcriptModelConfig.provider]);

    const fetchApiKey = async (provider: string) => {
        try {
            const data = await invoke('api_get_transcript_api_key', { provider }) as string;
            setApiKey(data || '');
        } catch (err) {
            console.error('Error fetching API key:', err);
            setApiKey(null);
        }
    };

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
            // Also fetch ASR status after successful connection
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
            // ASR management endpoints not available — that's OK
            setAsrStatus(null);
        }
    }, [openaiCompatibleEndpoint, transcriptModelConfig.openaiCompatibleEndpoint]);

    // Load ASR model into memory
    const handleLoadModel = async () => {
        setIsLoadingModel(true);
        const endpoint = openaiCompatibleEndpoint || transcriptModelConfig.openaiCompatibleEndpoint || '';
        try {
            const url = `${endpoint.replace(/\/+$/, '')}/v1/asr/load`;
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ model: openaiCompatibleModel || 'qwen3-asr-1.7b' }),
            });
            if (response.ok) {
                await fetchAsrStatus();
            }
        } catch {
            // ignore
        } finally {
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
        } catch {
            // ignore
        } finally {
            setIsUnloadingModel(false);
        }
    };

    // Change ASR language on server
    const handleAsrLanguageChange = async (lang: string) => {
        setAsrLanguage(lang);
        const endpoint = openaiCompatibleEndpoint || transcriptModelConfig.openaiCompatibleEndpoint || '';
        try {
            const url = `${endpoint.replace(/\/+$/, '')}/v1/asr/language`;
            await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ language: lang }),
            });
        } catch {
            // ignore
        }
    };

    // Sync global language preference to ASR server when it changes
    useEffect(() => {
        if (transcriptModelConfig.provider === 'openaiCompatible' && selectedLanguage) {
            handleAsrLanguageChange(selectedLanguage);
        }
    }, [selectedLanguage, transcriptModelConfig.provider]);

    const modelOptions: Record<string, string[]> = {
        localWhisper: [], // Model selection handled by ModelManager component
        parakeet: [], // Model selection handled by ParakeetModelManager component
        'openaiCompatible': [], // Model entered manually or populated from test connection
        deepgram: ['nova-2-phonecall'],
        elevenLabs: ['eleven_multilingual_v2'],
        groq: ['llama-3.3-70b-versatile'],
        openai: ['gpt-4o'],
    };
    const requiresApiKey = transcriptModelConfig.provider === 'deepgram' || transcriptModelConfig.provider === 'elevenLabs' || transcriptModelConfig.provider === 'openai' || transcriptModelConfig.provider === 'groq';

    const handleInputClick = () => {
        if (isApiKeyLocked) {
            setIsLockButtonVibrating(true);
            setTimeout(() => setIsLockButtonVibrating(false), 500);
        }
    };

    const handleWhisperModelSelect = (modelName: string) => {
        setTranscriptModelConfig({
            ...transcriptModelConfig,
            provider: 'localWhisper',
            model: modelName
        });
        if (onModelSelect) {
            onModelSelect();
        }
    };

    const handleParakeetModelSelect = (modelName: string) => {
        setTranscriptModelConfig({
            ...transcriptModelConfig,
            provider: 'parakeet',
            model: modelName
        });
        if (onModelSelect) {
            onModelSelect();
        }
    };

    const handleProviderChange = (value: string) => {
        const provider = value as TranscriptModelProps['provider'];
        setUiProvider(provider);
        if (provider !== 'localWhisper' && provider !== 'parakeet' && provider !== 'openaiCompatible') {
            fetchApiKey(provider);
        }
        // Update the config immediately when switching providers
        if (provider === 'openaiCompatible') {
            setTranscriptModelConfig({
                ...transcriptModelConfig,
                provider: 'openaiCompatible',
                model: openaiCompatibleModel || 'qwen3-asr-1.7b',
                openaiCompatibleEndpoint: openaiCompatibleEndpoint || null,
                openaiCompatibleApiKey: openaiCompatibleApiKey || null,
            });
        } else {
            setTranscriptModelConfig({
                ...transcriptModelConfig,
                provider,
            });
        }
    };

    return (
        <div>
            <div>
                <div className="space-y-4 pb-6">
                    <div>
                        <Label className="block text-sm font-medium text-gray-700 mb-1">
                            Transcript Model
                        </Label>
                        <div className="flex space-x-2 mx-1">
                            <Select
                                value={uiProvider}
                                onValueChange={handleProviderChange}
                            >
                                <SelectTrigger className='focus:ring-1 focus:ring-blue-500 focus:border-blue-500'>
                                    <SelectValue placeholder="Select provider" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="parakeet">⚡ Parakeet (Recommended - Real-time / Accurate)</SelectItem>
                                    <SelectItem value="localWhisper">🏠 Local Whisper (High Accuracy)</SelectItem>
                                    <SelectItem value="openaiCompatible">🔗 OpenAI-Compatible (Custom Server)</SelectItem>
                                    {/* <SelectItem value="deepgram">☁️ Deepgram (Backup)</SelectItem>
                                    <SelectItem value="elevenLabs">☁️ ElevenLabs</SelectItem>
                                    <SelectItem value="groq">☁️ Groq</SelectItem>
                                    <SelectItem value="openai">☁️ OpenAI</SelectItem> */}
                                </SelectContent>
                            </Select>

                            {uiProvider !== 'localWhisper' && uiProvider !== 'parakeet' && uiProvider !== 'openaiCompatible' && (
                                <Select
                                    value={transcriptModelConfig.model}
                                    onValueChange={(value) => {
                                        const model = value as TranscriptModelProps['model'];
                                        setTranscriptModelConfig({ ...transcriptModelConfig, provider: uiProvider, model });
                                    }}
                                >
                                    <SelectTrigger className='focus:ring-1 focus:ring-blue-500 focus:border-blue-500'>
                                        <SelectValue placeholder="Select model" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {(modelOptions[uiProvider] || []).map((model) => (
                                            <SelectItem key={model} value={model}>{model}</SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            )}

                        </div>
                    </div>

                    {/* OpenAI-Compatible Settings */}
                    {uiProvider === 'openaiCompatible' && (
                        <div className="space-y-3 mt-2 p-3 bg-gray-50 rounded-lg border border-gray-200">
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
                                <p className="text-xs text-gray-500 mt-1">
                                    The base URL of your OpenAI-compatible transcription server
                                </p>
                            </div>

                            <div>
                                <Label className="block text-sm font-medium text-gray-700 mb-1">
                                    API Key (optional)
                                </Label>
                                <div className="relative">
                                    <Input
                                        type={showApiKey ? "text" : "password"}
                                        className="pr-20 focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
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
                                <p className="text-xs text-gray-500 mt-1">
                                    The model name to send in transcription requests
                                </p>
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
                                        <span className="text-sm font-medium text-gray-900">ASR Server Status</span>
                                    </div>
                                    
                                    <div className="flex items-center gap-2 mb-3 text-sm">
                                        <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${asrStatus.loaded ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'}`}>
                                            <span className={`w-1.5 h-1.5 rounded-full ${asrStatus.loaded ? 'bg-green-500' : 'bg-gray-400'}`} />
                                            {asrStatus.loaded ? `Loaded: ${asrStatus.model_id}` : 'Not loaded'}
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
                                                <><Loader2 className="h-3 w-3 mr-1 animate-spin" />Loading...</>
                                            ) : (
                                                <><Download className="h-3 w-3 mr-1" />Load Model</>
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
                                                <><Loader2 className="h-3 w-3 mr-1 animate-spin" />Unloading...</>
                                            ) : (
                                                <><MemoryStick className="h-3 w-3 mr-1" />Unload Model</>
                                            )}
                                        </Button>
                                    </div>

                                    {asrStatus.loaded && (
                                        <div className="text-xs text-gray-500">
                                            <span className="flex items-center gap-1">
                                                <MemoryStick className="h-3 w-3" />
                                                {asrStatus.available_models?.find((m: any) => m.id === asrStatus.model_id)?.size_gb || '?'} GB in memory — click Unload to free memory when not recording
                                            </span>
                                        </div>
                                    )}
                                </div>
                            )}

                            {/* Silent sync: push global language preference to ASR server */}
                            {asrStatus && asrLanguage !== selectedLanguage && (
                                <div className="mt-2 p-2 bg-blue-50 border border-blue-200 rounded text-xs text-blue-700 flex items-center gap-1">
                                    <Server className="h-3 w-3" />
                                    Syncing language ({selectedLanguage}) to server...
                                </div>
                            )}
                            {asrStatus && asrLanguage === selectedLanguage && (
                                <div className="mt-2 text-xs text-gray-500 flex items-center gap-1">
                                    <CheckCircle2 className="h-3 w-3 text-green-500" />
                                    Server language: {asrLanguage}
                                </div>
                            )}
                        </div>
                    )}

                    {uiProvider === 'localWhisper' && (
                        <div className="mt-6">
                            <ModelManager
                                selectedModel={transcriptModelConfig.provider === 'localWhisper' ? transcriptModelConfig.model : undefined}
                                onModelSelect={handleWhisperModelSelect}
                                autoSave={true}
                            />
                        </div>
                    )}

                    {uiProvider === 'parakeet' && (
                        <div className="mt-6">
                            <ParakeetModelManager
                                selectedModel={transcriptModelConfig.provider === 'parakeet' ? transcriptModelConfig.model : undefined}
                                onModelSelect={handleParakeetModelSelect}
                                autoSave={true}
                            />
                        </div>
                    )}


                    {requiresApiKey && (
                        <div>
                            <Label className="block text-sm font-medium text-gray-700 mb-1">
                                API Key
                            </Label>
                            <div className="relative mx-1">
                                <Input
                                    type={showApiKey ? "text" : "password"}
                                    className={`pr-24 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 ${isApiKeyLocked ? 'bg-gray-100 cursor-not-allowed' : ''
                                        }`}
                                    value={apiKey || ''}
                                    onChange={(e) => setApiKey(e.target.value)}
                                    disabled={isApiKeyLocked}
                                    onClick={handleInputClick}
                                    placeholder="Enter your API key"
                                />
                                {isApiKeyLocked && (
                                    <div
                                        onClick={handleInputClick}
                                        className="absolute inset-0 flex items-center justify-center bg-gray-100 bg-opacity-50 rounded-md cursor-not-allowed"
                                    />
                                )}
                                <div className="absolute inset-y-0 right-0 pr-1 flex items-center">
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="icon"
                                        onClick={() => setIsApiKeyLocked(!isApiKeyLocked)}
                                        className={`transition-colors duration-200 ${isLockButtonVibrating ? 'animate-vibrate text-red-500' : ''
                                            }`}
                                        title={isApiKeyLocked ? "Unlock to edit" : "Lock to prevent editing"}
                                    >
                                        {isApiKeyLocked ? <Lock className="h-4 w-4" /> : <Unlock className="h-4 w-4" />}
                                    </Button>
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
                    )}
                </div>
            </div>
        </div >
    )
}