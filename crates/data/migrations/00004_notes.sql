-- Add notes column to character_base.
ALTER TABLE character_base ADD COLUMN notes TEXT NOT NULL DEFAULT '';
