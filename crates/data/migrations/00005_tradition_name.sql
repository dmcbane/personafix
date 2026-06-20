-- Add tradition_name column to character_base.
-- Stores the Magician sub-tradition (Hermetic / Shaman / Other).
-- NULL for SR4 characters (which use totem/tradition qualities instead).
ALTER TABLE character_base ADD COLUMN tradition_name TEXT;
