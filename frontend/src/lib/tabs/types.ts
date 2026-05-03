import type { Component } from 'svelte';

export type JSendSuccess<T> = { status: 'success'; data: T };
export type JSendFail = { status: 'fail'; data: Record<string, string> };
export type JSendError = { status: 'error'; message: string; code?: string };
export type JSend<T> = JSendSuccess<T> | JSendFail | JSendError;

export type Tab = {
	id: string;
	label: string;
	view: Component;
};
