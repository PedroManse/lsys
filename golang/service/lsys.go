package service

import (
	. "mysrv/util"
	"time"
	"database/sql"
)

type Book struct {
	ISBN uint64
	ID uint64
	Name string
	Published time.Time
	LastBorrow Option[time.Time]
	BorrowerId Option[uint64]
}

func (B Book) UntilFree() Tuple[int, bool] {
	th19, _ := B.LastBorrow.Get()
	ReturnLimit := th19.Add(MaxBorrowTime)
	hoursLeft := ReturnLimit.Sub(time.Now()).Hours()
	days := int(hoursLeft/24)
	if (days < 0) {days = -days}
	return Tuple[int, bool]{days, hoursLeft<=0.1}
}

// max borrow time is ten days
const MaxBorrowTime = 10*24*time.Hour
const TimeFormating = "02-01-2006"
func ParseTime( v string ) (time.Time, error) {
	return time.Parse(TimeFormating, v)
}
func FormatTime( t time.Time ) (string) {
	return t.Format(TimeFormating)
}

// maps
var (
	AvaliableBooks SyncMap[uint64, Book]
	AllBooks SyncMap[uint64, Book]

	IDToAuthor SyncMap[uint64, string]
	AuthorToId SyncMap[string, uint64]

	// book -> (when book was borrowed, by who)
	Borrows SyncMap[uint64, Tuple[time.Time, uint64]]
)

var (
	ListAllBooks = TemplatePage(
		"html/list.gohtml",
		map[string]any{
			"books":&AllBooks,
		},
		[]GOTMPlugin{GOTM_account},
	)
)

func sql_load(db *sql.DB) error {
	rows, e := SQLGet("lsys.sql_load # get authors", `SELECT id, name FROM authors;`)
	if (e != nil) {return e}
	defer rows.Close()
	for rows.Next() {
		var (
			id uint64
			name string
		)
		e := rows.Scan(&id, &name)
		if (e != nil) {return e}
		IDToAuthor.Set(id, name)
		AuthorToId.Set(name, id)
	}

	rows, e = SQLGet(
		"lsys.sql_load # get books",
		`SELECT ISBN, id, name, published, last_borrow, borrower_id FROM books;`,
	)
	if (e != nil) {return e}
	defer rows.Close()
	for rows.Next() {
		var (
			ISBN uint64
			ID uint64
			Name string
			Published_s string
			LastBorrow_s_o *string
			BorrowerId_o *uint64
		)

		e := rows.Scan( &ISBN, &ID, &Name, &Published_s, &LastBorrow_s_o, &BorrowerId_o )
		if (e != nil) {return e}
		Published, e := ParseTime(Published_s)
		if (e != nil) {return e}
		LastBorrow_s := OptPtr(LastBorrow_s_o)
		LastBorrow, _ := OptMapFal(LastBorrow_s, ParseTime)
		BorrowerId := OptPtr(BorrowerId_o)

		b := Book {
			ISBN, ID, Name,
			Published,
			LastBorrow,
			BorrowerId,
		}

		AllBooks.Set(ID, b)
		if (!LastBorrow.Has()) {
			AvaliableBooks.Set(ID, b)
		}
	}

	rows, e = SQLGet(
		"lsys.sql_load # get borrows",
		`SELECT user_id, book_id, time FROM borrowed;`,
	)
	if (e != nil) {return e}
	defer rows.Close()
	for rows.Next() {
		var uid, bid uint64
		var when_s string
		e = rows.Scan(&uid, &bid, &when_s)
		if (e != nil) {panic(e)}
		when, e := ParseTime(when_s)
		if (e != nil) {panic(e)}
		Borrows.Set( bid, Tuple[time.Time, uint64]{when, uid} )
	}

	return nil
}

func init() {
	AvaliableBooks.Init()
	AllBooks.Init()
	IDToAuthor.Init()
	AuthorToId.Init()
	Borrows.Init()

	SQLInitScript( "lsys schema", sql_schema )
	SQLInitFunc( "lsys load", sql_load)
}

const sql_schema = `
CREATE TABLE IF NOT EXISTS books (
	ISBN INTEGER NOT NULL,
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL,
	published TEXT NOT NULL,
	last_borrow TEXT,
	borrower_id INTEGER
);

CREATE TABLE IF NOT EXISTS authors (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS wrote (
	author_id INTEGER NOT NULL,
	ISBN INTEGER NOT NULL,
	UNIQUE(author_id, ISBN),
	FOREIGN KEY(author_id) REFERENCES authors(id),
	FOREIGN KEY(ISBN) REFERENCES books(ISBN)
);

CREATE TABLE IF NOT EXISTS borrowed (
	user_id INTEGER NOT NULL,
	book_id INTEGER NOT NULL,
	time TEXT NOT NULL,
	UNIQUE(user_id, book_id),
	FOREIGN KEY(user_id) REFERENCES accounts(id),
	FOREIGN KEY(book_id) REFERENCES books(id)
);
`
