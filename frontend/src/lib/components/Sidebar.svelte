<script lang="ts">
  import { onMount } from 'svelte';
  import { drawModeStore, setDrawMode } from '$lib/platform/map/draw-mode';
  import { clearSelection } from '$lib/platform/selection/store';
  import { duckdbStatus } from '$lib/platform/duckdb/client';
  import { fetchSkills } from '$lib/platform/api/client';
  import { agentResultStore } from '$lib/platform/render/store';
  import { conversationStore, hasConversation } from '$lib/platform/conversation/store';
  import {
    assistantErrorStore,
    backendOnlineStore,
    clearConversationState,
    dismissAssistantFeedback,
    downloadConversation,
    initializeAssistant,
    isListeningStore,
    toggleTts,
    toggleVoice,
    ttsEnabledStore,
    voiceSupportedStore,
  } from '$lib/platform/assistant/store';

  const tools = [
    { id: 'none', label: 'Select', icon: 'S', title: 'Click to select vessels' },
    { id: 'polygon', label: 'Polygon', icon: 'P', title: 'Draw polygon to select vessels' },
    { id: 'radius', label: 'Radius', icon: 'R', title: 'Click to set center, drag for radius' },
  ] as const;

  let loadedSkills = $state<string[]>([]);

  onMount(() => {
    initializeAssistant();
    void fetchSkills()
      .then((skills) => {
        loadedSkills = skills.map((skill) => skill.name).sort();
      })
      .catch(() => {
        loadedSkills = [];
      });
  });
</script>

<aside class="sidebar">
  <section class="sidebar-section">
    <div class="section-label">Tools</div>
    <div class="tool-list">
      {#each tools as tool}
        <button
          class="tool-btn"
          class:active={$drawModeStore === tool.id}
          title={tool.title}
          onclick={() => {
            setDrawMode(tool.id as 'none' | 'polygon' | 'radius');
            if (tool.id !== 'none') clearSelection();
          }}
        >
          <span class="tool-icon">{tool.icon}</span>
          <span class="tool-label">{tool.label}</span>
        </button>
      {/each}
    </div>
  </section>

  <section class="sidebar-section status-grid">
    <div>
      <div class="section-label">Backend</div>
      <div class="status-value" class:ok={$backendOnlineStore === true} class:error={$backendOnlineStore === false}>
        {$backendOnlineStore === null ? '--' : $backendOnlineStore ? 'Online' : 'Offline'}
      </div>
    </div>
    <div>
      <div class="section-label">DuckDB</div>
      <div class="status-value" class:ok={$duckdbStatus === 'ready'} class:error={$duckdbStatus === 'error'}>
        {$duckdbStatus === 'loading' ? 'Loading' : $duckdbStatus === 'ready' ? 'Ready' : $duckdbStatus === 'error' ? 'Error' : '--'}
      </div>
    </div>
  </section>

  <section class="sidebar-section">
    <div class="section-header">
      <div class="section-label">Skills</div>
      <div class="conversation-count">{loadedSkills.length} loaded</div>
    </div>
    {#if loadedSkills.length > 0}
      <div class="skill-list">
        {#each loadedSkills as skillName}
          <div class="skill-chip" class:highlight={skillName === 'speech_output'}>{skillName}</div>
        {/each}
      </div>
    {:else}
      <div class="empty-text">No skill metadata available.</div>
    {/if}
  </section>

  <section class="sidebar-section">
    <div class="section-header">
      <div class="section-label">Results</div>
      {#if $assistantErrorStore || $agentResultStore.status}
        <button class="ghost-btn" onclick={dismissAssistantFeedback}>Dismiss</button>
      {/if}
    </div>
    <div class="result-panel">
      {#if $assistantErrorStore}
        <div class="result-title error-text">Error</div>
        <div class="result-text error-text">{$assistantErrorStore}</div>
      {:else if $agentResultStore.status}
        <div class="result-title">{$agentResultStore.title || 'Response'}</div>
        <div class="result-text">{$agentResultStore.summary || 'No summary generated.'}</div>
      {:else}
        <div class="empty-text">Results will appear here.</div>
      {/if}
    </div>
  </section>

  <section class="sidebar-section">
    <div class="section-header">
      <div class="section-label">Voice</div>
      <div class="shortcut-hint">Space = voice</div>
    </div>
    <div class="voice-controls">
      <button
        class="voice-btn"
        class:active={$isListeningStore}
        onclick={toggleVoice}
        disabled={!$voiceSupportedStore}
        title={$voiceSupportedStore ? ($isListeningStore ? 'Stop listening' : 'Start voice input') : 'Voice input unavailable'}
      >
        {$isListeningStore ? 'Stop Mic' : 'Start Mic'}
      </button>
      <button class="tts-btn" class:active={$ttsEnabledStore} onclick={toggleTts}>
        {$ttsEnabledStore ? 'TTS On' : 'TTS Off'}
      </button>
    </div>
    {#if !$voiceSupportedStore}
      <div class="empty-text">Speech recognition is not available in this browser.</div>
    {/if}
  </section>

  <section class="sidebar-section transcript-section">
    <div class="section-header">
      <div class="section-label">Conversation</div>
      <div class="conversation-count">{$conversationStore.length} messages</div>
    </div>
    <div class="conversation-actions">
      <button class="utility-btn" onclick={downloadConversation} disabled={!$hasConversation}>Download</button>
      <button class="utility-btn" onclick={clearConversationState} disabled={!$hasConversation}>Clear</button>
    </div>
  </section>
</aside>

<style>
  .sidebar {
    position: absolute;
    top: 10px;
    left: 10px;
    bottom: 10px;
    width: min(320px, calc(100vw - 20px));
    z-index: 11;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    background: rgba(15, 20, 30, 0.94);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    box-shadow: 0 10px 32px rgba(0, 0, 0, 0.45);
    overflow-y: auto;
  }

  .sidebar-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-bottom: 10px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .sidebar-section:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .section-label {
    color: #7d8698;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .tool-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .skill-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .skill-chip {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    color: #c0d0e0;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 11px;
    padding: 6px 10px;
  }

  .skill-chip.highlight {
    border-color: rgba(255, 215, 0, 0.28);
    color: #ffd700;
    background: rgba(255, 215, 0, 0.08);
  }

  .tool-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid transparent;
    color: #c0d0e0;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
    padding: 10px 12px;
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
  }

  .tool-btn:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .tool-btn.active {
    background: rgba(255, 215, 0, 0.12);
    border-color: rgba(255, 215, 0, 0.28);
    color: #ffd700;
  }

  .tool-icon {
    width: 18px;
    text-align: center;
    color: inherit;
  }

  .status-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .status-value {
    margin-top: 4px;
    color: #9aa3b2;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
  }

  .status-value.ok {
    color: #4ade80;
  }

  .status-value.error {
    color: #f87171;
  }

  .result-panel {
    min-height: 120px;
    padding: 12px;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    overflow-y: auto;
  }

  .result-title {
    color: #ffd700;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.05em;
    margin-bottom: 8px;
    text-transform: uppercase;
  }

  .result-text,
  .empty-text {
    color: #c0d0e0;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .empty-text {
    color: #6b7280;
  }

  .error-text {
    color: #f87171;
  }

  .ghost-btn,
  .utility-btn,
  .voice-btn,
  .tts-btn {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    color: #d6dde8;
    cursor: pointer;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 11px;
    padding: 8px 10px;
  }

  .ghost-btn:hover,
  .utility-btn:hover,
  .voice-btn:hover,
  .tts-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
  }

  .utility-btn:disabled,
  .voice-btn:disabled,
  .tts-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .voice-controls,
  .conversation-actions {
    display: flex;
    gap: 8px;
  }

  .voice-btn,
  .tts-btn,
  .utility-btn {
    flex: 1;
  }

  .voice-btn.active {
    background: rgba(255, 80, 80, 0.18);
    border-color: rgba(255, 80, 80, 0.45);
    color: #ffd4d4;
  }

  .tts-btn.active {
    background: rgba(255, 215, 0, 0.12);
    border-color: rgba(255, 215, 0, 0.25);
    color: #ffd700;
  }

  .shortcut-hint,
  .conversation-count {
    color: #6b7280;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 10px;
  }

  @media (max-width: 900px) {
    .sidebar {
      width: min(280px, calc(100vw - 20px));
    }
  }
</style>
