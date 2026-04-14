<script lang="ts">
  import { querySQL, sendAgentMessage } from '$lib/platform/api/client';
  import { checkHealth } from '$lib/platform/api/client';
  import { setAgentResult, clearAgentResult, agentResultStore } from '$lib/platform/render/store';

  let inputValue = $state('');
  let isLoading = $state(false);
  let lastError = $state<string | null>(null);
  let backendOnline = $state<boolean | null>(null);

  // Voice state
  let isListening = $state(false);
  let voiceSupported = $state(false);
  let ttsEnabled = $state(true);
  let recognition: any = null;

  // Check browser support for Web Speech API
  $effect(() => {
    const SpeechRecognition = (globalThis as any).SpeechRecognition || (globalThis as any).webkitSpeechRecognition;
    voiceSupported = !!SpeechRecognition;
    if (SpeechRecognition) {
      recognition = new SpeechRecognition();
      recognition.continuous = false;
      recognition.interimResults = true;
      recognition.lang = 'en-US';

      recognition.onresult = (event: any) => {
        const transcript = Array.from(event.results)
          .map((r: any) => r[0].transcript)
          .join('');
        inputValue = transcript;

        // If final result, auto-submit
        const lastResult = event.results[event.results.length - 1];
        if (lastResult.isFinal) {
          isListening = false;
          // Small delay to let user see transcript before submitting
          setTimeout(() => handleSubmit(), 200);
        }
      };

      recognition.onerror = (event: any) => {
        console.warn('Speech recognition error:', event.error);
        isListening = false;
        if (event.error === 'not-allowed') {
          lastError = 'Microphone access denied. Check browser permissions.';
        }
      };

      recognition.onend = () => {
        isListening = false;
      };
    }
  });

  function toggleVoice() {
    if (!recognition) return;
    if (isListening) {
      recognition.stop();
      isListening = false;
    } else {
      lastError = null;
      inputValue = '';
      recognition.start();
      isListening = true;
    }
  }

  /** Speak text using browser TTS */
  function speak(text: string) {
    if (!ttsEnabled || !('speechSynthesis' in globalThis)) return;
    // Cancel any ongoing speech
    speechSynthesis.cancel();
    // Clean up markdown formatting for speech
    const clean = text
      .replace(/\*\*/g, '')
      .replace(/\*/g, '')
      .replace(/#{1,6}\s/g, '')
      .replace(/- /g, ', ')
      .replace(/\n+/g, '. ');
    const utterance = new SpeechSynthesisUtterance(clean);
    utterance.rate = 1.05;
    utterance.pitch = 1.0;
    speechSynthesis.speak(utterance);
  }

  // Check backend health on mount
  $effect(() => {
    checkHealth().then(h => {
      backendOnline = h !== null;
    });
  });

  // Derive display text from agent result store
  let agentResult = $state<{ summary: string; title: string } | null>(null);
  const unsub = agentResultStore.subscribe(r => {
    if (r.status) {
      agentResult = { summary: r.summary, title: r.title };
    } else {
      agentResult = null;
    }
  });

  async function handleSubmit() {
    const query = inputValue.trim();
    if (!query) return;

    inputValue = '';
    isLoading = true;
    lastError = null;
    clearAgentResult();

    try {
      // If it looks like SQL, run it directly
      const upper = query.toUpperCase();
      if (upper.startsWith('SELECT') || upper.startsWith('WITH')) {
        let sql = query;
        if (!upper.includes('LIMIT')) {
          sql += ' LIMIT 100';
        }
        const result = await querySQL(sql);
        // Wrap SQL result into an agent envelope shape
        const hasLatLon = result.columns.includes('lat') && (result.columns.includes('lon') || result.columns.includes('lng'));
        const render_as = ['table'];
        let geometry: GeoJSON.FeatureCollection | null = null;
        if (hasLatLon) {
          render_as.push('map');
          geometry = {
            type: 'FeatureCollection',
            features: result.rows.map(r => ({
              type: 'Feature' as const,
              geometry: {
                type: 'Point' as const,
                coordinates: [Number(r.lon ?? r.lng), Number(r.lat)],
              },
              properties: { ...r },
            })),
          };
        }
        setAgentResult({
          status: 'ok',
          title: `SQL Result`,
          summary: `${result.row_count} row(s) returned`,
          columns: result.columns,
          data: result.rows,
          render_as,
          geometry,
          actions: [],
          warnings: [],
        });
      } else {
        // Send to agent — returns full envelope
        const envelope = await sendAgentMessage(query);
        setAgentResult(envelope);
        // Speak the summary if TTS is enabled
        if (envelope.summary) {
          speak(envelope.summary);
        }
      }
    } catch (err: unknown) {
      lastError = err instanceof Error ? err.message : String(err);
    } finally {
      isLoading = false;
    }
  }

  function dismiss() {
    clearAgentResult();
    lastError = null;
    // Stop any ongoing speech
    if ('speechSynthesis' in globalThis) {
      speechSynthesis.cancel();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
    if (e.key === 'Escape') {
      dismiss();
    }
  }
</script>

<div class="command-bar">
  {#if agentResult || lastError}
    <div class="result-panel">
      {#if lastError}
        <div class="result-error">{lastError}</div>
      {:else if agentResult}
        <div class="result-title">{agentResult.title}</div>
        <div class="result-text">{agentResult.summary}</div>
      {/if}
      <button class="dismiss-btn" onclick={dismiss}>
        Esc to dismiss
      </button>
    </div>
  {/if}
  <div class="command-input-wrapper">
    <span class="prompt-char">&gt;</span>
    <input
      type="text"
      bind:value={inputValue}
      onkeydown={handleKeydown}
      placeholder={isListening
        ? 'Listening...'
        : backendOnline === false
          ? 'Backend offline — start backend on :3001'
          : 'SQL query or natural language...'}
      class="command-input"
      class:listening={isListening}
      disabled={isLoading}
    />
    {#if isLoading}
      <span class="loading-indicator">...</span>
    {:else}
      {#if voiceSupported}
        <button
          class="voice-btn"
          class:active={isListening}
          onclick={toggleVoice}
          title={isListening ? 'Stop listening' : 'Voice input'}
        >
          {isListening ? '⏹' : '🎤'}
        </button>
      {/if}
      <button
        class="tts-btn"
        class:active={ttsEnabled}
        onclick={() => { ttsEnabled = !ttsEnabled; if (!ttsEnabled && 'speechSynthesis' in globalThis) speechSynthesis.cancel(); }}
        title={ttsEnabled ? 'TTS on — click to mute' : 'TTS off — click to enable'}
      >
        {ttsEnabled ? '🔊' : '🔇'}
      </button>
      <button class="send-btn" onclick={handleSubmit} disabled={!inputValue.trim()}>
        Send
      </button>
    {/if}
    {#if backendOnline !== null}
      <span class="status-dot" class:online={backendOnline} class:offline={!backendOnline}
        title={backendOnline ? 'Backend connected' : 'Backend offline'}></span>
    {/if}
  </div>
</div>

<style>
  .command-bar {
    position: absolute;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    width: min(600px, calc(100vw - 32px));
    z-index: 10;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .result-panel {
    background: rgba(15, 20, 30, 0.95);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    padding: 12px;
    max-height: 200px;
    overflow-y: auto;
  }

  .result-title {
    color: #FFD700;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 11px;
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 6px;
  }

  .result-text {
    color: #c0d0e0;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.5;
  }

  .result-error {
    color: #ff6b6b;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
  }

  .dismiss-btn {
    display: block;
    margin-top: 8px;
    background: none;
    border: none;
    color: #555;
    font-size: 11px;
    cursor: pointer;
    font-family: 'SF Mono', 'Fira Code', monospace;
  }

  .command-input-wrapper {
    display: flex;
    align-items: center;
    background: rgba(15, 20, 30, 0.92);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    padding: 8px 12px;
    gap: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  }

  .prompt-char {
    color: #FFD700;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 14px;
    font-weight: bold;
  }

  .command-input {
    flex: 1;
    background: none;
    border: none;
    color: #e0e0e0;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 13px;
    outline: none;
  }

  .command-input::placeholder {
    color: #555;
  }

  .command-input.listening::placeholder {
    color: #FFD700;
    animation: pulse 1s infinite;
  }

  .command-input:disabled {
    opacity: 0.6;
  }

  .voice-btn, .tts-btn {
    background: none;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 14px;
    cursor: pointer;
    opacity: 0.6;
    transition: opacity 0.15s, background 0.15s;
  }

  .voice-btn:hover, .tts-btn:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.05);
  }

  .voice-btn.active {
    opacity: 1;
    background: rgba(255, 50, 50, 0.2);
    border-color: rgba(255, 50, 50, 0.5);
    animation: pulse 1s infinite;
  }

  .tts-btn.active {
    opacity: 0.8;
  }

  .send-btn {
    background: rgba(255, 215, 0, 0.15);
    color: #FFD700;
    border: 1px solid rgba(255, 215, 0, 0.3);
    border-radius: 4px;
    padding: 4px 12px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
    cursor: pointer;
  }

  .send-btn:hover:not(:disabled) {
    background: rgba(255, 215, 0, 0.25);
  }

  .send-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .loading-indicator {
    color: #FFD700;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 14px;
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-dot.online {
    background: #4ade80;
    box-shadow: 0 0 4px #4ade80;
  }

  .status-dot.offline {
    background: #ef4444;
    box-shadow: 0 0 4px #ef4444;
  }
</style>
