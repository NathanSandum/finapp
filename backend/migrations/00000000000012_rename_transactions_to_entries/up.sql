ALTER TABLE transactions RENAME TO entries;
ALTER TABLE postings RENAME COLUMN transaction_id TO entry_id;
