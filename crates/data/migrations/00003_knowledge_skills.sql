-- Add knowledge_skills_json column to character_base.
-- Knowledge skills are free-form (player-defined name + category + rating).
ALTER TABLE character_base ADD COLUMN knowledge_skills_json TEXT NOT NULL DEFAULT '[]';
