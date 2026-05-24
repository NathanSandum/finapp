// Anti-corruption layer barrel. Views never import these directly — the
// reactive stores in $lib/data own all calls and translate wire <-> domain.
export * as entriesBackend from './entries';
export * as accountsBackend from './accounts';
export * as recurringBackend from './recurring';
export * as anchorsBackend from './anchors';
export * as bootstrapBackend from './bootstrap';
