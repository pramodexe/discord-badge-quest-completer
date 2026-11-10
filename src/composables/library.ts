// The Library's "Your games" list, kept between sessions.
// In the app it is the file library.json next to the games (see the README);
// in the browser preview it falls back to the browser's own storage.
import { createGlobalState } from '@vueuse/core';
import { invoke } from '@tauri-apps/api/core';
import { ref, watch } from 'vue';
import type { Game } from '@/types/types';
import { isTauri } from '@/services/game-launcher';
import { parseLibrary, serializeLibrary } from '@/services/library-file';
import { useGlobalState } from './app-state';

const PREVIEW_KEY = 'library.games';

async function readSaved(): Promise<string | null> {
    if (isTauri()) return await invoke<string | null>('load_library');
    try {
        return localStorage.getItem(PREVIEW_KEY);
    } catch {
        return null;
    }
}

async function writeSaved(text: string): Promise<void> {
    if (isTauri()) {
        await invoke('save_library', { data: text });
        return;
    }
    try {
        localStorage.setItem(PREVIEW_KEY, text);
    } catch {
        // Storage blocked: the list just won't be kept in the preview.
    }
}

export const useLibrary = createGlobalState(() => {
    const { addLog } = useGlobalState();
    const games = ref<Game[]>([]);
    const loaded = ref(false);

    // Saves run one after another, so an older list can never overwrite a newer one.
    let queue: Promise<void> = Promise.resolve();
    function save() {
        const text = serializeLibrary(games.value);
        queue = queue
            .then(() => writeSaved(text))
            .catch(error => addLog('error', `Could not save your games list: ${error}`));
    }

    async function load() {
        try {
            const restored = parseLibrary(await readSaved());
            // Keep anything added while the file was being read.
            const ids = new Set(restored.map(g => g.id));
            games.value = [...restored, ...games.value.filter(g => !ids.has(g.id))];
            if (restored.length) addLog('info', `Restored ${restored.length} saved game${restored.length === 1 ? '' : 's'}.`);
        } catch (error) {
            addLog('warning', String(error));
        } finally {
            loaded.value = true;
        }
    }

    // Save on every add or remove, but never before the saved list is loaded
    // (otherwise an empty list could overwrite it at startup).
    watch(games, () => { if (loaded.value) save(); }, { deep: true });

    load();

    return { games, loaded };
});
