ALTER TABLE guild ADD COLUMN new_player_combat_tutorial_reward INTEGER NOT NULL DEFAULT 5;
ALTER TABLE guild ADD COLUMN new_player_tour_reward INTEGER NOT NULL DEFAULT 5;
ALTER TABLE guild ADD COLUMN weekly_spar_limit INTEGER NOT NULL DEFAULT 3;
ALTER TABLE guild ADD COLUMN weekly_spar_reward INTEGER NOT NULL DEFAULT 3;

ALTER TABLE character ADD COLUMN total_spar_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE character ADD COLUMN weekly_spar_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE character ADD COLUMN total_new_player_tour_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE character ADD COLUMN total_new_player_combat_tutorial_count INTEGER NOT NULL DEFAULT 0;
