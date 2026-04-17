import { derived, get, writable } from 'svelte/store';
import { checkHealth, querySQL, sendAgentMessage, synthesizeSpeech } from '$lib/platform/api/client';
import { selectionStore } from '$lib/platform/selection/store';
import { setAgentResult, clearAgentResult } from '$lib/platform/render/store';
import {
  addConversationEntry,
  clearConversation,
  conversationStore,
  getConversationContext,
} from '$lib/platform/conversation/store';
import { formatSpeechText } from '$lib/platform/assistant/speech';

type SpeechRecognitionLike = {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  onresult: ((event: any) => void) | null;
  onerror: ((event: any) => void) | null;
  onend: (() => void) | null;
  start: () => void;
  stop: () => void;
};

const browser = typeof window !== 'undefined';

export const commandInputStore = writable('');
export const assistantLoadingStore = writable(false);
export const assistantErrorStore = writable<string | null>(null);
export const backendOnlineStore = writable<boolean | null>(null);
export const voiceSupportedStore = writable(false);
export const isListeningStore = writable(false);
export const ttsEnabledStore = writable(true);

export const hasAssistantError = derived(assistantErrorStore, ($error) => $error !== null);

let recognition: SpeechRecognitionLike | null = null;
let initialized = false;
let healthCheckStarted = false;
let activeAudio: HTMLAudioElement | null = null;
let activeAudioUrl: string | null = null;
let speechGeneration = 0;

function buildAgentContext() {
  const selection = get(selectionStore);

  return {
    selection: selection.items.map((item) => ({
      mmsi: Number(item.mmsi),
      name: String(item.name),
    })),
    selection_mode: selection.mode,
    primary_mmsi: selection.primaryMMSI,
    conversation: getConversationContext(get(conversationStore)),
  };
}

function normalizeSpeechText(text: string) {
  return formatSpeechText(text
    .replace(/\*\*/g, '')
    .replace(/\*/g, '')
    .replace(/#{1,6}\s/g, '')
    .replace(/- /g, ', ')
    .replace(/\n+/g, '. '));
}

function stopActiveAudio() {
  if (activeAudio) {
    activeAudio.pause();
    activeAudio.src = '';
    activeAudio = null;
  }
  if (activeAudioUrl) {
    URL.revokeObjectURL(activeAudioUrl);
    activeAudioUrl = null;
  }
}

function speakWithBrowserFallback(text: string) {
  if (!browser || !('speechSynthesis' in window)) return;

  window.speechSynthesis.cancel();
  const clean = normalizeSpeechText(text);
  const utterance = new SpeechSynthesisUtterance(clean);
  utterance.rate = 1.05;
  utterance.pitch = 1.0;
  window.speechSynthesis.speak(utterance);
}

async function speak(text: string) {
  if (!browser || !get(ttsEnabledStore)) return;

  const generation = ++speechGeneration;
  const clean = normalizeSpeechText(text);
  stopActiveAudio();
  if ('speechSynthesis' in window) {
    window.speechSynthesis.cancel();
  }

  try {
    const audioBlob = await synthesizeSpeech(clean);
    if (generation !== speechGeneration || !get(ttsEnabledStore)) {
      return;
    }

    const objectUrl = URL.createObjectURL(audioBlob);
    const audio = new Audio(objectUrl);
    activeAudio = audio;
    activeAudioUrl = objectUrl;
    audio.onended = () => {
      if (generation !== speechGeneration) return;
      stopActiveAudio();
    };
    audio.onerror = () => {
      if (generation !== speechGeneration) {
        stopActiveAudio();
        return;
      }
      stopActiveAudio();
      speakWithBrowserFallback(clean);
    };
    if (generation !== speechGeneration || !get(ttsEnabledStore)) {
      stopActiveAudio();
      return;
    }
    await audio.play();
  } catch (error) {
    if (generation !== speechGeneration || !get(ttsEnabledStore)) {
      return;
    }
    console.warn('Backend TTS failed, using browser fallback:', error);
    speakWithBrowserFallback(clean);
  }
}

export function initializeAssistant() {
  if (!browser || initialized) return;
  initialized = true;

  const SpeechRecognitionCtor = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
  voiceSupportedStore.set(!!SpeechRecognitionCtor);

  if (SpeechRecognitionCtor) {
    const rec = new SpeechRecognitionCtor() as SpeechRecognitionLike;
    recognition = rec;
    rec.continuous = false;
    rec.interimResults = true;
    rec.lang = 'en-US';

    rec.onresult = (event: any) => {
      const transcript = Array.from(event.results)
        .map((result: any) => result[0].transcript)
        .join('');
      commandInputStore.set(transcript);

      const lastResult = event.results[event.results.length - 1];
      if (lastResult?.isFinal) {
        isListeningStore.set(false);
        window.setTimeout(() => {
          void submitAssistantQuery();
        }, 200);
      }
    };

    rec.onerror = (event: any) => {
      console.warn('Speech recognition error:', event.error);
      isListeningStore.set(false);
      if (event.error === 'not-allowed') {
        assistantErrorStore.set('Microphone access denied. Check browser permissions.');
      }
    };

    rec.onend = () => {
      isListeningStore.set(false);
    };
  }

  if (!healthCheckStarted) {
    healthCheckStarted = true;
    void checkHealth().then((health) => {
      backendOnlineStore.set(health !== null);
    });
  }
}

export function setCommandInput(value: string) {
  commandInputStore.set(value);
}

export function dismissAssistantFeedback() {
  clearAgentResult();
  assistantErrorStore.set(null);
  stopSpeechPlayback();
}

export function clearConversationState() {
  clearConversation();
  clearAgentResult();
  assistantErrorStore.set(null);
}

export function downloadConversation() {
  if (!browser) return;

  const entries = get(conversationStore);
  if (entries.length === 0) return;

  const transcript = entries
    .map((entry) => `[${new Date(entry.timestamp).toLocaleString()}] ${entry.role.toUpperCase()}: ${entry.text}`)
    .join('\n\n');

  const blob = new Blob([transcript], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = `conversation-${new Date().toISOString().replace(/[:.]/g, '-')}.txt`;
  anchor.click();
  URL.revokeObjectURL(url);
}

export function toggleTts() {
  const next = !get(ttsEnabledStore);
  ttsEnabledStore.set(next);
  if (!next) {
    stopSpeechPlayback();
  }
}

export function stopSpeechPlayback(): boolean {
  speechGeneration += 1;
  let stopped = false;

  if (activeAudio) {
    stopActiveAudio();
    stopped = true;
  }

  if (browser && 'speechSynthesis' in window && window.speechSynthesis.speaking) {
    window.speechSynthesis.cancel();
    stopped = true;
  }

  return stopped;
}

export function toggleVoice() {
  if (!recognition) return;

  if (get(isListeningStore)) {
    recognition.stop();
    isListeningStore.set(false);
    return;
  }

  assistantErrorStore.set(null);
  commandInputStore.set('');
  recognition.start();
  isListeningStore.set(true);
}

export async function submitAssistantQuery() {
  const query = get(commandInputStore).trim();
  if (!query || get(assistantLoadingStore)) return;

  commandInputStore.set('');
  assistantLoadingStore.set(true);
  assistantErrorStore.set(null);
  clearAgentResult();
  const agentContext = buildAgentContext();
  addConversationEntry('user', query);

  try {
    const upper = query.toUpperCase();
    if (upper.startsWith('SELECT') || upper.startsWith('WITH')) {
      let sql = query;
      if (!upper.includes('LIMIT')) {
        sql += ' LIMIT 100';
      }
      const result = await querySQL(sql);
      const hasLatLon = result.columns.includes('lat') && (result.columns.includes('lon') || result.columns.includes('lng'));
      const render_as = ['table'];
      let geometry: GeoJSON.FeatureCollection | null = null;
      if (hasLatLon) {
        render_as.push('map');
        geometry = {
          type: 'FeatureCollection',
          features: result.rows.map((row) => ({
            type: 'Feature' as const,
            geometry: {
              type: 'Point' as const,
              coordinates: [Number(row.lon ?? row.lng), Number(row.lat)],
            },
            properties: { ...row },
          })),
        };
      }
      setAgentResult({
        status: 'ok',
        title: 'SQL Result',
        summary: `${result.row_count} row(s) returned`,
        columns: result.columns,
        data: result.rows,
        render_as,
        geometry,
        actions: [],
        warnings: [],
      });
      addConversationEntry('assistant', `${result.row_count} row(s) returned.`);
      return;
    }

    const envelope = await sendAgentMessage(query, agentContext);
    setAgentResult(envelope);
    addConversationEntry('assistant', envelope.summary || envelope.title || 'No response generated.');
    if (envelope.summary) {
      void speak(envelope.summary);
    }
  } catch (err: unknown) {
    assistantErrorStore.set(err instanceof Error ? err.message : String(err));
  } finally {
    assistantLoadingStore.set(false);
  }
}
