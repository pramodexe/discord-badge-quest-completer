import { describe, expect, it } from 'vitest';
import { parseLibrary, serializeLibrary } from './library-file';
import type { Game } from '@/types/types';

const game: Game = {
    uid: 'u1',
    id: '123',
    name: 'GTA Vice City',
    executables: [{ name: 'win64/vicecity.exe', os: 'win32', is_launcher: false }],
    aliases: ['Vice City'],
    themes: ['Action'],
    icon_hash: 'abc123',
};

describe('library file', () => {
    it('saves and loads the same games back', () => {
        const { themes: _notSaved, ...saved } = game;
        expect(parseLibrary(serializeLibrary([game]))).toEqual([saved]);
    });

    it('keeps the order games were added in', () => {
        const second = { ...game, uid: 'u2', id: '456', name: 'Second' };
        expect(parseLibrary(serializeLibrary([game, second])).map(g => g.id)).toEqual(['123', '456']);
    });

    it('starts empty when nothing was saved or the text is broken', () => {
        expect(parseLibrary(null)).toEqual([]);
        expect(parseLibrary('')).toEqual([]);
        expect(parseLibrary('{ broken')).toEqual([]);
        expect(parseLibrary('[]')).toEqual([]);
        expect(parseLibrary('{"games": "nope"}')).toEqual([]);
    });

    it('skips broken entries and repeats, keeps the good ones', () => {
        const text = JSON.stringify({
            version: 1,
            games: [
                { id: '1', name: 'Good', executables: [{ name: 'a.exe', os: 'win32' }, { bad: true }] },
                { id: '1', name: 'Repeat' },
                { name: 'No id' },
                'not a game',
                { id: '2', name: 'No executables' },
            ],
        });
        const games = parseLibrary(text);
        expect(games.map(g => g.name)).toEqual(['Good', 'No executables']);
        expect(games[0].executables).toEqual([{ name: 'a.exe', os: 'win32', is_launcher: false }]);
        expect(games[1].executables).toEqual([]);
    });

    it('gives every game a uid', () => {
        const games = parseLibrary(JSON.stringify({ version: 1, games: [{ id: '9', name: 'X', executables: [] }] }));
        expect(games[0].uid).toBeTruthy();
    });
});
