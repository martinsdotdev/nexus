-- Cloud-mode account profile (ADR-0010): a user-chosen display name, shown in presence and
-- the collaborator roster in place of the email local-part once set. Null until the user
-- picks one; the email local-part stays the fallback.
alter table app_user add column display_name text;
