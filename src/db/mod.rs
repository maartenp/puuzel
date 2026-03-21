pub mod schema;

/// Open a SQLite database at the given path, enabling WAL journal mode and initializing the schema.
pub fn open_database(path: &std::path::Path) -> rusqlite::Result<rusqlite::Connection> {
    let conn = rusqlite::Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    schema::init_schema(&conn)?;
    Ok(conn)
}

/// Open an in-memory SQLite database with WAL mode and initialized schema (for testing).
pub fn open_in_memory() -> rusqlite::Result<rusqlite::Connection> {
    let conn = rusqlite::Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    schema::init_schema(&conn)?;
    Ok(conn)
}

/// Insert a word into the words table and return its row id.
pub fn insert_word(
    conn: &rusqlite::Connection,
    word: &str,
    grid_length: i64,
    commonness_score: i32,
    is_proper_noun: bool,
    is_archaic: bool,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO words (word, grid_length, commonness_score, is_proper_noun, is_archaic) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![word, grid_length, commonness_score, is_proper_noun as i32, is_archaic as i32],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Insert a clue for a word and return its row id.
pub fn insert_clue(
    conn: &rusqlite::Connection,
    word_id: i64,
    difficulty: &str,
    clue_text: &str,
    verified: bool,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO clues (word_id, difficulty, clue_text, verified, thumbs_down) VALUES (?1, ?2, ?3, ?4, 0)",
        rusqlite::params![word_id, difficulty, clue_text, verified as i32],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Query words by grid length and minimum commonness score at a given difficulty level.
/// Only returns words that have at least one verified, non-thumbs-down clue at the requested difficulty.
/// Returns up to 500 words in random order.
pub fn words_for_length(
    conn: &rusqlite::Connection,
    length: usize,
    min_commonness: i32,
    difficulty: &str,
) -> rusqlite::Result<Vec<(i64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT w.id, w.word FROM words w
         WHERE w.grid_length = ?1 AND w.commonness_score >= ?2
         AND EXISTS (SELECT 1 FROM clues c WHERE c.word_id = w.id AND c.difficulty = ?3 AND c.verified = 1 AND c.thumbs_down = 0)
         ORDER BY RANDOM() LIMIT 500",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![length as i64, min_commonness, difficulty],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
    )?;
    rows.collect()
}

/// Get a verified, non-thumbs-down clue for a word at the given difficulty level.
/// Returns None if no such clue exists.
pub fn get_clue_for_word(
    conn: &rusqlite::Connection,
    word_id: i64,
    difficulty: &str,
) -> rusqlite::Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT clue_text FROM clues WHERE word_id = ?1 AND difficulty = ?2 AND verified = 1 AND thumbs_down = 0 ORDER BY RANDOM() LIMIT 1",
    )?;
    let mut rows = stmt.query_map(rusqlite::params![word_id, difficulty], |row| {
        row.get::<_, String>(0)
    })?;
    match rows.next() {
        Some(result) => result.map(Some),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let conn = open_in_memory().expect("open_in_memory failed");
        // Verify tables exist via sqlite_master
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('words', 'clues')",
                [],
                |row| row.get(0),
            )
            .expect("query failed");
        assert_eq!(count, 2, "Expected both 'words' and 'clues' tables");
    }

    #[test]
    fn test_insert_and_query_word() {
        let conn = open_in_memory().expect("open_in_memory failed");
        let word_id = insert_word(&conn, "HUIS", 4, 5, false, false).expect("insert_word failed");
        assert!(word_id > 0);

        insert_clue(&conn, word_id, "easy", "Woonplaats", true).expect("insert_clue failed");

        let results = words_for_length(&conn, 4, 1, "easy").expect("words_for_length failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "HUIS");
    }

    #[test]
    fn test_commonness_filter() {
        let conn = open_in_memory().expect("open_in_memory failed");

        let low_id = insert_word(&conn, "RARE", 4, 2, false, false).expect("insert failed");
        insert_clue(&conn, low_id, "easy", "Zeldzaam", true).expect("insert_clue failed");

        let high_id = insert_word(&conn, "DEUR", 4, 5, false, false).expect("insert failed");
        insert_clue(&conn, high_id, "easy", "Opening in muur", true).expect("insert_clue failed");

        let results = words_for_length(&conn, 4, 4, "easy").expect("words_for_length failed");
        assert_eq!(results.len(), 1, "Only DEUR (commonness=5) should be returned with min_commonness=4");
        assert_eq!(results[0].1, "DEUR");
    }

    #[test]
    fn test_clue_verified_filter() {
        let conn = open_in_memory().expect("open_in_memory failed");

        let id1 = insert_word(&conn, "BOOM", 4, 5, false, false).expect("insert failed");
        insert_clue(&conn, id1, "easy", "Groot gewas", true).expect("insert_clue failed");

        let id2 = insert_word(&conn, "ROOS", 4, 5, false, false).expect("insert failed");
        // Insert unverified clue
        insert_clue(&conn, id2, "easy", "Bloem", false).expect("insert_clue failed");

        let results = words_for_length(&conn, 4, 1, "easy").expect("words_for_length failed");
        assert_eq!(results.len(), 1, "Only BOOM with verified clue should be returned");
        assert_eq!(results[0].1, "BOOM");
    }

    #[test]
    fn test_get_clue() {
        let conn = open_in_memory().expect("open_in_memory failed");

        let word_id = insert_word(&conn, "FIETS", 5, 5, false, false).expect("insert failed");
        insert_clue(&conn, word_id, "medium", "Tweewielig voertuig", true).expect("insert_clue failed");

        let clue = get_clue_for_word(&conn, word_id, "medium").expect("get_clue failed");
        assert_eq!(clue, Some("Tweewielig voertuig".to_string()));
    }

    #[test]
    fn test_thumbs_down_excluded() {
        let conn = open_in_memory().expect("open_in_memory failed");

        let word_id = insert_word(&conn, "TAFEL", 5, 5, false, false).expect("insert failed");
        let clue_id = insert_clue(&conn, word_id, "easy", "Meubel", true).expect("insert_clue failed");

        // Mark clue as thumbs down via direct SQL
        conn.execute(
            "UPDATE clues SET thumbs_down = 1 WHERE id = ?1",
            rusqlite::params![clue_id],
        )
        .expect("update failed");

        let clue = get_clue_for_word(&conn, word_id, "easy").expect("get_clue failed");
        assert_eq!(clue, None, "Thumbs-down clue should not be returned");

        // Also confirm words_for_length excludes it
        let results = words_for_length(&conn, 5, 1, "easy").expect("words_for_length failed");
        assert_eq!(results.len(), 0, "Word with only thumbs-down clue should be excluded");
    }
}
