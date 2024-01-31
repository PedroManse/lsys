/*
DROP TABLE IF EXISTS borrow_log;
DROP TABLE IF EXISTS status;
DROP TABLE IF EXISTS wrote;
DROP TABLE IF EXISTS authors;
DROP TABLE IF EXISTS books;
DROP TABLE IF EXISTS accounts;
*/

pub const TABLE_SCHEMA: &str = r#"

CREATE TABLE IF NOT EXISTS accounts (
	id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL,
	email TEXT NOT NULL,
	pass INTEGER NOT NULL,
	is_worker BOOL NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS books (
	id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	ISBN INTEGER NOT NULL,
	name TEXT NOT NULL,
	published TEXT NOT NULL,
	user_id INTEGER DEFAULT NULL,
	time DATE DEFAULT NULL,
	is_borrow BOOL DEFAULT NULL,
	CHECK((time IS NULL) == (user_id IS NULL)),
	CHECK((time IS NULL) == (is_borrow IS NULL))
);

CREATE TABLE IF NOT EXISTS authors (
	id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS wrote (
	author_id INTEGER NOT NULL,
	ISBN INTEGER NOT NULL,
	UNIQUE(author_id, ISBN),
	FOREIGN KEY(author_id) REFERENCES authors(id),
	FOREIGN KEY(ISBN) REFERENCES books(ISBN)
);

CREATE TABLE IF NOT EXISTS borrow_log (
	user_id INTEGER NOT NULL,
	book_id INTEGER NOT NULL,
	borrow_time TEXT NOT NULL,
	return_time TEXT NOT NULL,
	CHECK(borrow_time != return_time),
	UNIQUE(borrow_time, user_id, book_id),
	UNIQUE(return_time, user_id, book_id),
	FOREIGN KEY(user_id) REFERENCES accounts(id),
	FOREIGN KEY(book_id) REFERENCES books(id)
);

"#;

/*

[worker] create new book
INSERT INTO books
	(ISBN, name, published)
VALUES
	(?, ?, ?);

[user] reserve book
UPDATE books SET
	(user_id, time, is_borrow)
VALUES
	(?, ?, false);

[worker] borrow book -- as per user reservation
UPDATE books SET
	(time, is_borrow)
VALUES
	(?, true)
WHERE
	(book_id == ?);

[worker] return book -- as per user IRL action
UPDATE books SET
	(user_id, time, is_borrow)
VALUES
	(NULL, NULL, NULL)
WHERE
	(book_id == ?);

[server] get books
SELECT
	id, ISBN, name, published, user_id, time, is_borrow
FROM
	books;

*/
