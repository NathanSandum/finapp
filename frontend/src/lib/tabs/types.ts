import type { Component } from 'svelte';
import type { VerbId } from '$lib/views/entries/verbs';

export type JSendSuccess<T> = { status: 'success'; data: T };
export type JSendFail = { status: 'fail'; data: Record<string, string> };
export type JSendError = { status: 'error'; message: string; code?: string };
export type JSend<T> = JSendSuccess<T> | JSendFail | JSendError;

export type TabGroup = 'overview' | 'statements' | 'mechanics';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type Tab = {
	id: string;
	label: string;
	group: TabGroup;
	newEntryVerb?: VerbId;
	view: Component<any>;
	loader: () => Promise<JSend<any>>;
};
