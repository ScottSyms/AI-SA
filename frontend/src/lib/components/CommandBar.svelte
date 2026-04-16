<script lang="ts">
  import { onMount } from 'svelte';
  import {
    assistantLoadingStore,
    backendOnlineStore,
    commandInputStore,
    dismissAssistantFeedback,
    initializeAssistant,
    isListeningStore,
    setCommandInput,
    stopSpeechPlayback,
    submitAssistantQuery,
    toggleVoice,
    voiceSupportedStore,
  } from '$lib/platform/assistant/store';

  function handleInput(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    setCommandInput(target.value);
  }

  function shouldHandleVoiceShortcut(event: KeyboardEvent): boolean {
    if (event.code !== 'Space') return false;
    if (event.repeat || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return false;

    const target = event.target as HTMLElement | null;
    if (!target) return true;

    const tagName = target.tagName;
    if (tagName === 'INPUT' || tagName === 'TEXTAREA' || tagName === 'SELECT') return false;
    if (target.isContentEditable) return false;

    return true;
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (shouldHandleVoiceShortcut(event)) {
      event.preventDefault();

      if (stopSpeechPlayback()) {
        return;
      }

      if (!$voiceSupportedStore) {
        return;
      }

      toggleVoice();
      return;
    }

    if (event.key === 'Escape') {
      dismissAssistantFeedback();
    }
  }

  function handleInputKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      void submitAssistantQuery();
    }
  }

  onMount(() => {
    initializeAssistant();
  });
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="command-bar">
  <div class="command-input-wrapper">
    <span class="prompt-char">&gt;</span>
    <input
      type="text"
      value={$commandInputStore}
      oninput={handleInput}
      onkeydown={handleInputKeydown}
      placeholder={$isListeningStore
        ? 'Listening...'
        : $backendOnlineStore === false
          ? 'Backend offline - start backend on :3001'
          : 'SQL query or natural language...'}
      class="command-input"
      class:listening={$isListeningStore}
      disabled={$assistantLoadingStore}
    />
    {#if $assistantLoadingStore}
      <span class="loading-indicator">...</span>
    {:else}
      {#if $voiceSupportedStore}
        <button
          class="voice-btn"
          class:active={$isListeningStore}
          onclick={toggleVoice}
          title={$isListeningStore ? 'Stop listening' : 'Voice input'}
        >
          {$isListeningStore ? 'Mic Off' : 'Mic'}
        </button>
      {/if}
      <button class="send-btn" onclick={() => void submitAssistantQuery()} disabled={!$commandInputStore.trim()}>
        Send
      </button>
    {/if}
    {#if $backendOnlineStore !== null}
      <span class="status-dot" class:online={$backendOnlineStore} class:offline={!$backendOnlineStore}
        title={$backendOnlineStore ? 'Backend connected' : 'Backend offline'}></span>
    {/if}
  </div>
</div>

<style>
  .command-bar {
    position: absolute;
    bottom: 16px;
    left: 16px;
    right: 16px;
    width: auto;
    z-index: 10;
  }

  .command-input-wrapper {
    display: flex;
    align-items: center;
    background: rgba(15, 20, 30, 0.92);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 10px;
    padding: 10px 12px;
    gap: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  }

  .prompt-char {
    color: #ffd700;
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
    color: #ffd700;
    animation: pulse 1s infinite;
  }

  .command-input:disabled {
    opacity: 0.6;
  }

  .voice-btn,
  .send-btn {
    border-radius: 6px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
    padding: 6px 10px;
    cursor: pointer;
  }

  .voice-btn {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #d6dde8;
  }

  .voice-btn.active {
    background: rgba(255, 80, 80, 0.18);
    border-color: rgba(255, 80, 80, 0.45);
    color: #ffd4d4;
  }

  .send-btn {
    background: rgba(255, 215, 0, 0.15);
    color: #ffd700;
    border: 1px solid rgba(255, 215, 0, 0.3);
  }

  .send-btn:hover:not(:disabled),
  .voice-btn:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .send-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .loading-indicator {
    color: #ffd700;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 14px;
    animation: pulse 1s infinite;
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

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

</style>
