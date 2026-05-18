import type { VerbId } from './verbs';

export type EntryIntent = {
	verb: VerbId;
	sourceTab: string;
	counterpartyId?: string;
};

const state: { current: EntryIntent | null } = $state({ current: null });

export const entryIntent = {
	get(): EntryIntent | null {
		return state.current;
	},
	set(intent: EntryIntent) {
		state.current = intent;
	},
	clear() {
		state.current = null;
	}
};
