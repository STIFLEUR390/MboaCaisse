-- V6__payments_extras.sql
-- Additional payment columns for MoMo operator tracking and split payments.
--
-- Story 3.4: momo_operator tracks the mobile money operator (orange|mtn).
--            parent_payment_id links split sub-payments to their parent.
-- AD-7: Columns are optional (NULLABLE) for backward compatibility.

ALTER TABLE payments ADD COLUMN momo_operator TEXT;

ALTER TABLE payments ADD COLUMN parent_payment_id TEXT;

-- Index for retrieving sub-payments of a split transaction
CREATE INDEX IF NOT EXISTS idx_payments_parent
    ON payments(parent_payment_id);

-- Index for filtering payments by operator (reporting)
CREATE INDEX IF NOT EXISTS idx_payments_momo_operator
    ON payments(momo_operator);
