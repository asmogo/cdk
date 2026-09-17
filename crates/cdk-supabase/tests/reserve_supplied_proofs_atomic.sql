-- Run with psql -v ON_ERROR_STOP=1 in a disposable database after applying
-- the Supabase migrations. All fixture data and privilege changes are rolled back.
BEGIN;
GRANT ALL ON public.proof TO authenticated;
SET LOCAL ROLE authenticated;
SET LOCAL request.jwt.claims = '{"sub":"supplied-proof-test"}';

INSERT INTO public.proof(y, mint_url, state, unit, amount, keyset_id, secret, c,
    derivation_index, created_by_operation)
    VALUES ('coin-z', 'https://mint.example', 'UNSPENT', 'sat', 64, 'keyset',
        'encrypted-secret', 'encrypted-c', 23, 'origin-operation');

DO $test$
DECLARE
    original JSONB;
    reserved JSONB;
    external JSONB;
    failed BOOLEAN;
BEGIN
    SELECT to_jsonb(p) INTO original FROM public.proof p WHERE y = 'coin-z';
    reserved := original || '{"state":"RESERVED","used_by_operation":"new-operation","witness":"signed-input"}'::JSONB;
    external := reserved || '{"y":"coin-a","derivation_index":null}'::JSONB;

    -- A reservation acquired after the read defeats the entire batch, including
    -- an external coin sorted before it. The original owner keeps its proof.
    UPDATE public.proof SET state = 'RESERVED', used_by_operation = 'other-operation' WHERE y = 'coin-z';
    failed := FALSE;
    BEGIN
        PERFORM public.reserve_supplied_proofs_atomic(jsonb_build_array(
            jsonb_build_object('before', NULL, 'after', external),
            jsonb_build_object('before', original, 'after', reserved)), 'new-operation');
    EXCEPTION WHEN raise_exception THEN failed := TRUE;
    END;
    ASSERT failed, 'stale proof should reject the batch';
    ASSERT (SELECT used_by_operation = 'other-operation' FROM public.proof WHERE y = 'coin-z');
    ASSERT NOT EXISTS (SELECT 1 FROM public.proof WHERE y = 'coin-a');

    -- A stale absence cannot overwrite a concurrently imported proof.
    failed := FALSE;
    BEGIN
        PERFORM public.reserve_supplied_proofs_atomic(jsonb_build_array(
            jsonb_build_object('before', NULL, 'after', reserved)), 'new-operation');
    EXCEPTION WHEN raise_exception THEN failed := TRUE;
    END;
    ASSERT failed, 'stale absence should reject the import';
    ASSERT (SELECT used_by_operation = 'other-operation' FROM public.proof WHERE y = 'coin-z');

    -- Existing coins retain their ciphertext and origin; new inputs are imported.
    UPDATE public.proof SET state = 'UNSPENT', used_by_operation = NULL WHERE y = 'coin-z';
    SELECT to_jsonb(p) INTO original FROM public.proof p WHERE y = 'coin-z';
    PERFORM public.reserve_supplied_proofs_atomic(jsonb_build_array(
        jsonb_build_object('before', original, 'after', reserved),
        jsonb_build_object('before', NULL, 'after', external)), 'new-operation');
    ASSERT (SELECT state = 'RESERVED' AND used_by_operation = 'new-operation'
        AND secret = 'encrypted-secret' AND c = 'encrypted-c' AND derivation_index = 23
        AND created_by_operation = 'origin-operation' AND witness = 'signed-input'
        FROM public.proof WHERE y = 'coin-z');
    ASSERT (SELECT state = 'RESERVED' AND used_by_operation = 'new-operation'
        FROM public.proof WHERE y = 'coin-a');

    -- An encrypted row read from another wallet cannot authorize a reservation.
    PERFORM set_config('request.jwt.claims', '{"sub":"other-wallet"}', TRUE);
    failed := FALSE;
    BEGIN
        PERFORM public.reserve_supplied_proofs_atomic(jsonb_build_array(
            jsonb_build_object('before', original, 'after', reserved)), 'new-operation');
    EXCEPTION WHEN raise_exception THEN failed := TRUE;
    END;
    ASSERT failed, 'wallet isolation should reject reservation';
    ASSERT NOT EXISTS (SELECT 1 FROM public.proof WHERE y IN ('coin-a', 'coin-z'));
END;
$test$;
ROLLBACK;
