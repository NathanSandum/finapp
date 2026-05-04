import type { JSendError } from '$lib/tabs/types';

const BY_CODE: Record<string, string> = {
	'401': 'Your session has expired. Please sign in again.',
	'403': "You don't have access to this.",
	'404': "We couldn't find what you were looking for.",
	'500': 'Something went wrong on our end. Please try again.',
	'502': 'Could not reach the server. Try again in a moment.',
	'503': 'Could not reach the server. Try again in a moment.',
	'504': 'Could not reach the server. Try again in a moment.'
};

const NETWORK_HINTS = ['fetch failed', 'NetworkError', 'Failed to fetch', 'malformed response'];

export function friendlyError(result: JSendError): string {
	if (result.code && BY_CODE[result.code]) return BY_CODE[result.code];
	if (NETWORK_HINTS.some((hint) => result.message.includes(hint))) {
		return 'Could not reach the server. Try again in a moment.';
	}
	return result.message;
}
