-- Add edition to campaigns so all characters in a campaign share the same ruleset.
ALTER TABLE campaigns ADD COLUMN edition TEXT NOT NULL DEFAULT 'SR4' CHECK (edition IN ('SR4', 'SR5'));
