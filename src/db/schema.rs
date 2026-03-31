/// Initialize the database schema: create tables and indexes if they don't exist.
pub fn init_schema(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS words (
            id INTEGER PRIMARY KEY,
            word TEXT NOT NULL UNIQUE,
            grid_length INTEGER NOT NULL,
            commonness_score INTEGER NOT NULL,
            is_proper_noun INTEGER NOT NULL DEFAULT 0,
            is_archaic INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS clues (
            id INTEGER PRIMARY KEY,
            word_id INTEGER NOT NULL REFERENCES words(id),
            difficulty TEXT NOT NULL CHECK(difficulty IN ('easy', 'medium', 'hard')),
            clue_text TEXT NOT NULL,
            verified INTEGER NOT NULL DEFAULT 0,
            thumbs_down INTEGER NOT NULL DEFAULT 0,
            model TEXT NOT NULL DEFAULT 'haiku'
        );

        CREATE INDEX IF NOT EXISTS idx_words_grid_length ON words(grid_length);
        CREATE INDEX IF NOT EXISTS idx_words_commonness ON words(commonness_score);
        CREATE INDEX IF NOT EXISTS idx_clues_word_difficulty ON clues(word_id, difficulty);
        CREATE INDEX IF NOT EXISTS idx_clues_verified ON clues(verified);
        ",
    )?;

    // Migration: add model column to existing databases
    let has_model: bool = conn
        .prepare("PRAGMA table_info(clues)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .any(|name| name == "model");
    if !has_model {
        conn.execute_batch("ALTER TABLE clues ADD COLUMN model TEXT NOT NULL DEFAULT 'haiku'")?;
    }

    Ok(())
}
