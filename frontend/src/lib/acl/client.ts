// Anti-corruption layer (ACL) — outbound HTTP.
//
// Only modules under $lib/acl call this. Everything else (views, stores)
// talks to the backend through the typed acl/* functions, which translate
// wire shapes to domain types.

export { apiFetch } from '$lib/api/client';
