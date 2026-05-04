import type { Component } from 'svelte';

export type JSendSuccess<T> = { status: 'success'; data: T };
export type JSendFail = { status: 'fail'; data: Record<string, string> | { code: string; message: string } };
export type JSendError = { status: 'error'; message: string; code?: string };
export type JSend<T> = JSendSuccess<T> | JSendFail | JSendError;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type Tab = {
	id: string;
	label: string;
	view: Component<any>;
	loader: () => Promise<JSend<any>>;
};
