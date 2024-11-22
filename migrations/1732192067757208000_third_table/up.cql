CREATE TABLE IF NOT EXISTS encryption_db.payment_attempts (
  payment_id text,
  merchant_id text,
  attempt_id text,
  status text,
  PRIMARY KEY ((payment_id, merchant_id), attempt_id)
);

CREATE TABLE IF NOT EXISTS encryption_db.payment_attempts2 (
  payment_id text,
  merchant_id text,
  attempt_id text,
  status text,
  PRIMARY KEY ((payment_id, merchant_id), attempt_id)
);

CREATE TABLE IF NOT EXISTS encryption_db.payment_attempts3 (
  payment_id text,
  merchant_id text,
  attempt_id text,
  status text,
  PRIMARY KEY ((payment_id, merchant_id), attempt_id)
);
