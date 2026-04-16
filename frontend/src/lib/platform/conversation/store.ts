import { derived, writable } from 'svelte/store';

export interface ConversationEntry {
  role: 'user' | 'assistant';
  text: string;
  timestamp: string;
}

const MAX_CONTEXT_ENTRIES = 12;

export const conversationStore = writable<ConversationEntry[]>([]);

export const hasConversation = derived(conversationStore, ($entries) => $entries.length > 0);

export function addConversationEntry(role: ConversationEntry['role'], text: string) {
  const trimmed = text.trim();
  if (!trimmed) return;

  conversationStore.update((entries) => [
    ...entries,
    {
      role,
      text: trimmed,
      timestamp: new Date().toISOString(),
    },
  ]);
}

export function clearConversation() {
  conversationStore.set([]);
}

export function getConversationContext(entries: ConversationEntry[]) {
  return entries.slice(-MAX_CONTEXT_ENTRIES);
}
