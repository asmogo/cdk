-- Reserve or import a batch without replacing another operation's proofs.
-- Compare original ciphertext records inside the same transaction as the writes.
CREATE OR REPLACE FUNCTION reserve_supplied_proofs_atomic(p_proofs JSONB, p_operation_id TEXT)
RETURNS VOID
LANGUAGE plpgsql
SECURITY INVOKER
SET search_path = ''
AS $body$
DECLARE
    owner TEXT := public.get_current_wallet_id();
    current_proof public.proof;
    transition JSONB;
    new_proof public.proof;
BEGIN
    FOR transition IN SELECT value FROM pg_catalog.jsonb_array_elements(p_proofs)
        ORDER BY value->'after'->>'y'
    LOOP
        SELECT * INTO current_proof FROM public.proof
            WHERE y = transition->'after'->>'y' AND wallet_id = owner FOR UPDATE;
        IF transition->'before' = 'null'::JSONB THEN
            IF FOUND THEN RAISE EXCEPTION 'Supplied proof already exists'; END IF;
            new_proof := pg_catalog.jsonb_populate_record(NULL::public.proof,
                transition->'after' || pg_catalog.jsonb_build_object(
                    'wallet_id', owner, 'opt_version', 1, 'state', 'RESERVED',
                    'used_by_operation', p_operation_id));
            INSERT INTO public.proof SELECT new_proof.*;
        ELSE
            IF NOT FOUND OR NOT (pg_catalog.to_jsonb(current_proof) @> (transition->'before'))
                OR current_proof.state <> 'UNSPENT' OR current_proof.used_by_operation IS NOT NULL THEN
                RAISE EXCEPTION 'Proof changed or is already reserved';
            END IF;
            UPDATE public.proof SET state = 'RESERVED', used_by_operation = p_operation_id,
                witness = transition->'after'->>'witness'
                WHERE y = current_proof.y AND wallet_id = owner;
        END IF;
    END LOOP;
END;
$body$;
