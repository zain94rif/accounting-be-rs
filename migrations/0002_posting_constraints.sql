CREATE OR REPLACE FUNCTION enforce_balanced_journal()
RETURNS TRIGGER AS $$
DECLARE
  v_entry_id UUID;
  v_status TEXT;
BEGIN
  v_entry_id := COALESCE(NEW.journal_entry_id, OLD.journal_entry_id);

  SELECT status INTO v_status
  FROM journal_entries
  WHERE id = v_entry_id;

  IF v_status = 'posted' THEN
    RAISE EXCEPTION 'Cannot modify journal lines of a posted entry';
  END IF;

  RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_enforce_balanced_journal ON journal_lines;
CREATE TRIGGER trg_enforce_balanced_journal
AFTER INSERT OR UPDATE OR DELETE ON journal_lines
FOR EACH ROW EXECUTE FUNCTION enforce_balanced_journal();

CREATE OR REPLACE FUNCTION post_journal_entry(p_entry_id UUID)
RETURNS VOID AS $$
DECLARE
  v_debit NUMERIC(18,2);
  v_credit NUMERIC(18,2);
  v_status TEXT;
BEGIN
  SELECT status INTO v_status FROM journal_entries WHERE id = p_entry_id FOR UPDATE;
  IF v_status <> 'draft' THEN
    RAISE EXCEPTION 'Only draft entries can be posted';
  END IF;

  SELECT COALESCE(SUM(debit),0), COALESCE(SUM(credit),0)
    INTO v_debit, v_credit
  FROM journal_lines WHERE journal_entry_id = p_entry_id;

  IF v_debit <> v_credit THEN
    RAISE EXCEPTION 'Entry not balanced: debit % credit %', v_debit, v_credit;
  END IF;

  UPDATE journal_entries
  SET status='posted', posted_at=now()
  WHERE id = p_entry_id;
END;
$$ LANGUAGE plpgsql;
