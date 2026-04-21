ALTER TABLE postings RENAME COLUMN entry_id TO transaction_id;
ALTER TABLE entries RENAME TO transactions;
