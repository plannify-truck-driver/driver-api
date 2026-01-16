INSERT INTO "drivers" ("pk_driver_id", "firstname", "lastname", "gender", "email", "password_hash", "phone_number", "is_searchable", "allow_request_professional_agreement", "language", "rest_json", "mail_preferences", "created_at", "verified_at", "last_login_at", "deactivated_at", "refresh_token_hash") VALUES
('123e4567-e89b-12d3-a456-426614174000', 'Test', 'Test', NULL, 'test.user@example.com', '$argon2id$v=19$m=19456,t=2,p=1$GvJ0zPtHLrLN0ubKYXtqdw$dAqS9mMzUO55YVmiWPESW60AagJ5px+803z3nuEmH48', NULL, 't', 'f', 'fr', NULL, 0, '2026-01-01 00:00:00', '2026-01-01 00:00:00', NULL, NULL, '$argon2id$v=19$m=19456,t=2,p=1$keLh9hYB2aBoQ9/59bZ/2g$lu1ieiTyfpvGuNdZ//brM+b7GvbaleGkmRI02fqc8Jo'),
('123e4567-e89b-12d3-a456-426614174001', 'Test-bis', 'Test', NULL, 'test-bis.user@example.com', '$argon2id$v=19$m=19456,t=2,p=1$GvJ0zPtHLrLN0ubKYXtqdw$dAqS9mMzUO55YVmiWPESW60AagJ5px+803z3nuEmH48', NULL, 't', 'f', 'fr', NULL, 0, '2026-01-01 00:00:00', '2026-01-01 00:00:00', NULL, NULL, '$argon2id$v=19$m=19456,t=2,p=1$keLh9hYB2aBoQ9/59bZ/2g$lu1ieiTyfpvGuNdZ//brM+b7GvbaleGkmRI02fqc8Jo');

INSERT INTO "workdays" ("date", "fk_driver_id", "start_time", "end_time", "rest_time", "overnight_rest") VALUES
('2025-12-31', '123e4567-e89b-12d3-a456-426614174000', '08:17:00', '18:52:00', '00:30:00', 'f'),
('2026-01-01', '123e4567-e89b-12d3-a456-426614174000', '08:00:00', '17:45:00', '01:00:00', 't'),
('2026-01-15', '123e4567-e89b-12d3-a456-426614174000', '07:34:00', '19:03:00', '00:00:00', 't'),
('2026-01-31', '123e4567-e89b-12d3-a456-426614174000', '07:40:00', NULL, '00:30:00', 'f'),
('2026-02-01', '123e4567-e89b-12d3-a456-426614174000', '08:40:00', '17:21:00', '01:30:00', 'f'),
('2027-01-01', '123e4567-e89b-12d3-a456-426614174000', '08:14:00', '13:31:00', '01:45:00', 't'),
('2026-01-01', '123e4567-e89b-12d3-a456-426614174001', '07:15:00', '17:00:00', '00:45:00', 'f'),
('2026-01-02', '123e4567-e89b-12d3-a456-426614174001', '09:00:00', NULL, '00:30:00', 'f');

INSERT INTO "workday_garbage" ("workday_date", "fk_driver_id", "created_at", "scheduled_deletion_date") VALUES
('2026-01-15', '123e4567-e89b-12d3-a456-426614174000', '2026-02-10 11:30:00', '2026-03-11');