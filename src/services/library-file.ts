// Reading and writing the saved "Your games" list (library.json).
// Kept free of Vue and Tauri so it can be tested on its own.
import type { Game, GameExecutable } from '@/types/types';
import { randomString } from '@/utils/random-string';

export const LIBRARY_VERSION = 1;

export interface LibraryFile {
    version: number;
    games: Game[];
}

function cleanExecutable(value: unknown): GameExecutable | null {
    if (!value || typeof value !== 'object') return null;
    const e = value as Record<string, unknown>;
    if (typeof e.name !== 'string' || typeof e.os !== 'string') return null;
    return { name: e.name, os: e.os, is_launcher: e.is_launcher === true };
}

function cleanGame(value: unknown): Game | null {
    if (!value || typeof value !== 'object') return null;
    const g = value as Record<string, unknown>;
    if (typeof g.id !== 'string' || g.id === '' || typeof g.name !== 'string') return null;
    const executables = Array.isArray(g.executables)
        ? g.executables.map(cleanExecutable).filter((e): e is GameExecutable => e !== null)
        : [];
    return {
        uid: typeof g.uid === 'string' && g.uid !== '' ? g.uid : randomString(),
        id: g.id,
        name: g.name,
        executables,
        aliases: Array.isArray(g.aliases) ? g.aliases.filter((a): a is string => typeof a === 'string') : [],
        icon_hash: typeof g.icon_hash === 'string' ? g.icon_hash : null,
    };
}

/** Turns saved text back into games, skipping anything broken and any repeats. */
export function parseLibrary(text: string | null | undefined): Game[] {
    if (!text) return [];
    let data: unknown;
    try {
        data = JSON.parse(text);
    } catch {
        return [];
    }
    const list = data && typeof data === 'object' && Array.isArray((data as LibraryFile).games)
        ? (data as LibraryFile).games
        : [];
    const seen = new Set<string>();
    const games: Game[] = [];
    for (const item of list) {
        const game = cleanGame(item);
        if (!game || seen.has(game.id)) continue;
        seen.add(game.id);
        games.push(game);
    }
    return games;
}

/** What gets written to library.json: only what is needed to show and launch each game. */
export function serializeLibrary(games: Game[]): string {
    const file: LibraryFile = {
        version: LIBRARY_VERSION,
        games: games.map(g => ({
            uid: g.uid,
            id: g.id,
            name: g.name,
            executables: g.executables.map(e => ({ name: e.name, os: e.os, is_launcher: e.is_launcher })),
            aliases: g.aliases ?? [],
            icon_hash: g.icon_hash ?? null,
        })),
    };
    return JSON.stringify(file, null, 2);
}
