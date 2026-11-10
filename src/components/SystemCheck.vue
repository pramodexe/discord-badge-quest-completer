<script setup lang="ts">
// The startup check. Runs once when the app opens and shows a short report only
// when something needs attention. Everything is fine -> nothing is shown.
import { invoke } from '@tauri-apps/api/core';
import { computed, onMounted, ref } from 'vue';
import { isTauri } from '@/services/game-launcher';

interface Check {
  id: string;
  title: string;
  status: 'ok' | 'warn' | 'fail';
  detail: string;
  fix: string | null;
  commands: [string, string][];
}

const checks = ref<Check[]>([]);
const open = ref(false);
const running = ref(false);
const copied = ref('');

const failed = computed(() => checks.value.some(c => c.status === 'fail'));
const needsAttention = computed(() => checks.value.some(c => c.status !== 'ok'));

async function run(showEvenIfFine = false) {
  if (!isTauri()) return; // browser preview: nothing to check
  running.value = true;
  try {
    checks.value = await invoke<Check[]>('system_check');
  } catch (e) {
    checks.value = [{ id: 'internal', title: 'Startup check', status: 'warn', detail: `The check could not run: ${e}`, fix: null, commands: [] }];
  } finally {
    running.value = false;
  }
  open.value = needsAttention.value || showEvenIfFine;
}

async function copy(command: string) {
  try {
    await navigator.clipboard.writeText(command);
    copied.value = command;
    setTimeout(() => { if (copied.value === command) copied.value = ''; }, 1500);
  } catch {
    // Clipboard not allowed: the command is still visible and selectable.
  }
}

onMounted(() => run());
defineExpose({ run });
</script>

<template>
  <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6" role="dialog" aria-modal="true" aria-labelledby="system-check-title">
    <div class="flex max-h-full w-full max-w-2xl flex-col overflow-hidden rounded-2xl border border-line bg-panel shadow-2xl">
      <header class="border-b border-line-soft px-6 py-5">
        <h2 id="system-check-title" class="text-xl font-semibold">
          {{ failed ? 'Something needs fixing first' : needsAttention ? 'Before you start' : 'Everything is ready' }}
        </h2>
        <p class="mt-1 text-sm text-muted">
          {{ failed ? 'Fake games cannot start until the red item below is fixed.' : 'The app checked what it needs to make Discord show you as playing.' }}
        </p>
      </header>

      <ul class="flex-1 space-y-3 overflow-y-auto px-6 py-5">
        <li v-for="check in checks" :key="check.id" class="rounded-xl border border-line-soft bg-ink px-4 py-3">
          <div class="flex items-start gap-3">
            <span
              class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full text-xs font-bold text-ink"
              :class="{ 'bg-ok': check.status === 'ok', 'bg-flame-hi': check.status === 'warn', 'bg-bad': check.status === 'fail' }"
              :aria-label="check.status === 'ok' ? 'OK' : check.status === 'warn' ? 'Warning' : 'Problem'"
            >{{ check.status === 'ok' ? '✓' : '!' }}</span>
            <div class="min-w-0 flex-1">
              <p class="font-semibold">{{ check.title }}</p>
              <p class="mt-0.5 break-words text-sm text-muted">{{ check.detail }}</p>
              <p v-if="check.fix && check.status !== 'ok'" class="mt-2 text-sm">{{ check.fix }}</p>
              <ul v-if="check.commands.length && check.status !== 'ok'" class="mt-2 space-y-1.5">
                <li v-for="[system, command] in check.commands" :key="system" class="text-xs">
                  <span class="text-faint">{{ system }}</span>
                  <div class="mt-0.5 flex items-center gap-2">
                    <code class="min-w-0 flex-1 select-all break-all rounded-lg bg-raised px-2.5 py-1.5 font-mono text-cream">{{ command }}</code>
                    <button type="button" class="shrink-0 rounded-lg border border-line px-2.5 py-1.5 text-muted hover:text-cream" @click="copy(command)">
                      {{ copied === command ? 'Copied' : 'Copy' }}
                    </button>
                  </div>
                </li>
              </ul>
            </div>
          </div>
        </li>

        <li class="rounded-xl border border-line-soft bg-ink px-4 py-3 text-sm text-muted">
          <span class="font-semibold text-cream">Also check in Discord:</span>
          Settings, then Activity Privacy: turn on sharing your detected activity. The app cannot see this setting.
        </li>
      </ul>

      <footer class="flex items-center justify-end gap-3 border-t border-line-soft px-6 py-4">
        <button type="button" class="rounded-xl border border-line px-4 py-2 text-muted hover:text-cream disabled:opacity-50" :disabled="running" @click="run(true)">
          {{ running ? 'Checking…' : 'Check again' }}
        </button>
        <button type="button" class="rounded-xl bg-flame px-4 py-2 font-semibold text-ink hover:bg-flame-hi" @click="open = false">
          {{ failed ? 'Continue anyway' : 'Continue' }}
        </button>
      </footer>
    </div>
  </div>
</template>
