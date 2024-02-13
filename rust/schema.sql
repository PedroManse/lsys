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
	(1499669402, "A Study In Scarlet", "1878"),
	(0553293354, "Foundation"        , "1998"),
	(0000000005, "The Raven"         , "1978"),
	(0000000006, "The Black Cat"     , "1978");

INSERT INTO books
	(ISBN)
VALUES
	(1499669402), (1499669402), (1499669402),
	(0553293354), (0000000005), (0000000006),
	(0000000006), (1499669402), (0553293354);

INSERT INTO authors
	(id, name)
VALUES
	(1, "Edgar Allan Poe"),
	(2, "Isaac Asimov"),
	(3, "Arthur Conan Doyle");

INSERT INTO wrote
	(ISBN, author_id)
VALUES
	(0000000005, 1),
	(0000000006, 1),
	(0553293354, 2),
	(1499669402, 3),
	(0553293354, 3);

INSERT INTO accounts
	(name,email,pass_hash,is_worker)
VALUES
	('manse','pmanse@lsys.com','a6f85bd2-a2a5-360b-a4e4-13b8eb84b14e',true),
	('manse','pedro@manse.com','a6f85bd2-a2a5-360b-a4e4-13b8eb84b14e',false);

-- select books to display
--SELECT
--books.id, books.ISBN, books.user_id, books.time, books.is_borrow, book_info.name, book_info.published, group_concat(authors.name)
--FROM books
--INNER JOIN authors, wrote USING (ISBN), book_info USING (ISBN)
--WHERE wrote.author_id == authors.id
--GROUP BY books.id;

