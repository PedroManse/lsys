pub struct DB(rusqlite::Connection);

pub fn open(path: &str) -> Result<DB, rusqlite::Error> {
	if path==":memory:" {
		rusqlite::Connection::open_in_memory().map(|con|DB(con))
	} else {
		rusqlite::Connection::open(path).map(|con|DB(con))
	}
}

#[allow(dead_code)]
impl DB {
	pub fn schema(&self, command: &str) -> Result<usize, rusqlite::Error> {
		self.0.execute(command, ())
	}

	// on Ok returns number of rows edited
	pub fn exec<P: rusqlite::Params>(&self, command: &str, params: P) -> Result<usize, rusqlite::Error> {
		self.0.prepare_cached(command)?.execute(params)
	}

	// on Ok returns last insert id
	pub fn insert<P: rusqlite::Params>(&self, command: &str, params: P) -> Result<i64, rusqlite::Error> {
		self.0.prepare_cached(command)?.insert(params)
	}

	pub fn last_insert_id(&self) -> i64 {
		self.0.last_insert_rowid()
	}

	pub fn close(self) -> Result<(), (DB, rusqlite::Error)> {
		self.0.close().map_err(|(con, e)| (DB(con), e) )
	}

	pub fn prepare(&self, command: &str) -> Result<rusqlite::CachedStatement, rusqlite::Error> {
		self.0.prepare_cached(command)
	}
}

pub const SQL_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS books (
	ISBN INTEGER NOT NULL,
	book_id INTEGER PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL,
	published DATE NOT NULL,
	last_borrow DATE,
	borrower_id INTEGER
);

CREATE TABLE IF NOT EXISTS authors (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL,
	email TEXT NOT NULL,
)

CREATE TABLE IF NOT EXISTS wrote (
	author_id INTEGER NOT NULL,
	book_id INTEGER NOT NULL,
	UNIQUE(author_id, book_id),
	FOREIGN KEY(author_id) REFERENCES authors(id),
	FOREIGN KEY(ISBN) REFERENCES books(ISBN)
);

CREATE TABLE IF NOT EXISTS borrows (
	user_id INTEGER NOT NULL,
	book_id INTEGER NOT NULL,
	UNIQUE(user_id, book_id),
	FOREIGN KEY(user_id) REFERENCES users(id),
	FOREIGN KEY(ISBN) REFERENCES books(ISBN)
)

"#;
