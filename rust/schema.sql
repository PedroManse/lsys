DROP TABLE IF EXISTS accounts;
CREATE TABLE IF NOT EXISTS accounts (
	id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL,
	email TEXT NOT NULL UNIQUE,
	pass_hash TEXT NOT NULL,
	is_worker BOOL NOT NULL DEFAULT false
);

DROP TABLE IF EXISTS books;
CREATE TABLE IF NOT EXISTS books (
	id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	ISBN INTEGER NOT NULL,

	user_id INTEGER DEFAULT NULL,
	time DATE DEFAULT NULL,
	is_borrow BOOL DEFAULT NULL,
	CHECK((time IS NULL) == (user_id IS NULL)),
	CHECK((time IS NULL) == (is_borrow IS NULL))
);

DROP TABLE IF EXISTS book_info;
CREATE TABLE IF NOT EXISTS book_info (
	ISBN INTEGER NOT NULL PRIMARY KEY,
	name TEXT NOT NULL,
	published TEXT NOT NULL,
	FOREIGN KEY(ISBN) REFERENCES books(ISBN)
);

DROP TABLE IF EXISTS authors;
CREATE TABLE IF NOT EXISTS authors (
	id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL UNIQUE
);

DROP TABLE IF EXISTS wrote;
CREATE TABLE IF NOT EXISTS wrote (
	author_id INTEGER NOT NULL,
	ISBN INTEGER NOT NULL,
	UNIQUE(author_id, ISBN),
	FOREIGN KEY(author_id) REFERENCES authors(id),
	FOREIGN KEY(ISBN) REFERENCES books(ISBN)
);

DROP TABLE IF EXISTS borrow_log;
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

INSERT INTO book_info
	(ISBN, name, published)
VALUES
	(3,"A Study In Scarlet","1878"),
	(4,"Foundation","1998"),
	(5,"The Raven","1978"),
	(6,"The Black Cat","1978");

-- Could to info, ISBN -> (name, published)
-- instead of keeping the same value in _every_ book
INSERT INTO books
	(id, ISBN)
VALUES
	(1,3), (2,3), (3,3),
	(4,4), (5,5), (6,6),
	(7,6), (8,3);

INSERT INTO authors
	(id, name)
VALUES
	(1, "Edgar Allan Poe"),
	(2, "Isac Asimov"),
	(3, "Arthur Conan Doyle");

INSERT INTO wrote
	(author_id, ISBN)
VALUES
	(3, 3),
	(2, 4),
	(3, 4),
	(1, 5),
	(1, 6);
