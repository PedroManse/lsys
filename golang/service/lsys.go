package service

import (
	. "mysrv/util"
	"time"
	"database/sql"
	"strconv"
	"net/http"
)

type Book struct {
	ISBN uint64
	ID uint64
	Name string
	Published string
	NextBorrow Option[time.Time]
	LastBorrow Option[time.Time]
	BorrowerId Option[int64]
}

// (days it will still be reserved), false
// (days since it should have been returned), true
func (B Book) UntilFree() Tuple[int, bool] {
	th19, _ := B.LastBorrow.Get()
	ReturnLimit := th19.Add(MaxBorrowTime)
	hoursLeft := ReturnLimit.Sub(time.Now()).Hours()
	days := int(hoursLeft/24)
	if (days < 0) {days = -days}
	return Tuple[int, bool]{days, hoursLeft<=0.1}
}

//func (B Book) StillReserved() Tuple[int, bool] { }

// max borrow time is ten days
const MaxBorrowTime = 10*24*time.Hour
const TimeFormating = "02-01-2006"
func ParseTime( v string ) (time.Time, error) {
	return time.Parse(TimeFormating, v)
}
func FormatTime( t time.Time ) (string) {
	return t.Format(TimeFormating)
}
const HTMLTimeFormating = "2006-01-02"
func ParseHTMLTime( v string ) (time.Time, error) {
	return time.Parse(HTMLTimeFormating, v)
}

// maps
var (
	AvaliableBooks SyncMap[uint64, Book]
	AllBooks SyncMap[uint64, Book]

	IDToAuthor SyncMap[uint64, string]
	AuthorToId SyncMap[string, uint64]

	// book -> (when book was borrowed, by who)
	Borrows SyncMap[uint64, Tuple[time.Time, int64]]

	// all reserved books
	UIDToToBorrowRequest SyncMap[int64, []uint64]
)

var HTMLError = InlineComponent(`<h1>{{.}}</h1>`)

var DocError = InlineComponent(`
<!DOCTYPE html>
<html> <head>
	<meta charset="UTF-8">
	<title>LSYS - Error!</title>
</head> <body>
	<h1>{{.}}</h1> <a href="/lsys/">LSYS - home</a>
</body> </html>
`)

var (
	ListAllBooks = TemplatePage(
		"html/list.gohtml",
		map[string]any{
			"books":&AllBooks,
		},
		[]GOTMPlugin{GOTM_account},
	)
	ListAvaliableBooks = TemplatePage(
		"html/list.gohtml",
		map[string]any{
			"books":&AvaliableBooks,
		},
		[]GOTMPlugin{GOTM_account},
	)
	DisplayBook = LogicPage(
		"html/book.gohtml", nil,
		[]GOTMPlugin{GOTM_account, GOTM_URLID("bid")},
		gatherBookInfo,
	)
	AddBook = LogicPage(
		"html/add.gohtml", nil,
		[]GOTMPlugin{GOTM_account, GOTM_mustacc},
		addBook,
	)
)

func gatherBookInfo(w HttpWriter, r HttpReq, info map[string]any) (bool, any) {
	bid, e := info["urlid"].(map[string]Tuple[uint64, error])["bid"].Unpack()
	if (e != nil) {
		DocError.Render(w, e)
		return false, nil
	}
	binfo := make(map[string]any)
	binfo["ID"] = bid
	book, ok := AllBooks.Get(bid)
	if (!ok) {
		DocError.Render(w, "No book with such ID")
		return false, nil
	}
	binfo["book"] = book
	binfo["avail"] = !book.BorrowerId.Has()
	if (book.BorrowerId.Has()) {
		UID, _ := book.BorrowerId.Get()
		acc := IDToAccount.GetI(UID)
		//TODO better has method && doesn't have account resolution
		if (book.LastBorrow.Has()) {
			binfo["borrower_name"] = acc.Name
			binfo["borrower_email"] = acc.Email
			binfo["borrower_status"] = "borrowed"
		} else if (book.NextBorrow.Has()) {
			binfo["borrower_name"] = acc.Name
			binfo["borrower_email"] = acc.Email
			binfo["borrower_status"] = "reserved"
			//TODO fix this; should provide expected return time when borrowed
			// : should provide expected borrow time and return time when reserved
			//EPickUpTime := BorrowRequestExpectedPickup.GetI(Tuple{acc.ID, book.ID})
			//hoursLeft := EPickUpTime.Sub(time.Now()).Hours()
			//past := hoursLeft<=0.1
			//binfo["borrower_time_left"] = int(hoursLeft/24)
			//binfo["borrower_time_lim"] = nil
		} else {
			binfo["borrower_error"] = "Can't find borrower account"
			binfo["borrower_name"] = "?"
			binfo["borrower_email"] = "?"
		}

		days, past := book.UntilFree().Unpack()
		binfo["borrower_past"] = past
		binfo["borrower_days_left"] = days
	}
	return true, binfo
}

func addBook(w HttpWriter, r HttpReq, info map[string]any) (bool, any) {
	accid := info["acc"].(map[string]any)["id"].(int64)
	inf := AttachAccountInfo.GetI(accid)
	isWorker := inf.GetI("lsys.is_worker").(int64)
	if (isWorker == 0) {
		DocError.Render(w, "Only workers can add books")
		return false, nil
	}
	if (r.Method=="POST") {

		r.ParseForm()
		ISBN_s := r.FormValue("ISBN")
		ISBN, e := strconv.ParseUint(ISBN_s, 10, 64)
		if (e != nil) {
			DocError.Render(w, e)
			return false, nil
		}

		name := r.FormValue("name")
		pubdate := r.FormValue("published")
		ID_s, e := sql_add_book(ISBN_s, name, pubdate)
		ID:=uint64(ID_s) // signed -> unsigned

		if (e != nil) {
			DocError.Render(w, e)
			return false, nil
		}

		b := Book{
			ISBN, ID, name,
			pubdate,
			OptPtr[time.Time](nil),
			OptPtr[time.Time](nil),
			OptPtr[int64](nil),
		}
		AvaliableBooks.Set(ID, b)
		AllBooks.Set(ID, b)
		// redirect to book page
		ID_str := strconv.FormatUint(ID, 10)
		http.Redirect(w, r, "/lsys/book?bid="+ID_str, http.StatusSeeOther)
	}
	return true, nil
}

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
		`SELECT ISBN, id, name, published, last_borrow, next_borrow, borrower_id FROM books;`,
	)
	if (e != nil) {return e}
	defer rows.Close()
	for rows.Next() {
		var (
			ISBN uint64
			ID uint64
			Name string
			Published string
			LastBorrow_s_o *string
			NextBorrow_s_o *string
			BorrowerId_o *int64
		)

		e := rows.Scan(
			&ISBN, &ID,
			&Name, &Published,
			&LastBorrow_s_o,
			&NextBorrow_s_o,
			&BorrowerId_o,
		)
		if (e != nil) {return e}
		LastBorrow_s := OptPtr(LastBorrow_s_o)
		LastBorrow, _ := OptMapFal(LastBorrow_s, ParseTime)
		NextBorrow_s := OptPtr(NextBorrow_s_o)
		NextBorrow, _ := OptMapFal(NextBorrow_s, ParseTime)

		BorrowerId := OptPtr(BorrowerId_o)

		// if reserved, update map of user requests
		//ureqs := UIDToToBorrowRequest.GetO(UID).Default([]uint64{})
		//ureqs = append(ureqs, BID)
		//UIDToToBorrowRequest.Set(UID, ureqs)

		b := Book{
			ISBN, ID, Name,
			Published,
			NextBorrow,
			LastBorrow,
			BorrowerId,
		}

		AllBooks.Set(ID, b)
		// has not been picked up nor is reserved
		if !(BorrowerId.Has()) {
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
		var uid int64
		var bid uint64
		var when_s string
		e = rows.Scan(&uid, &bid, &when_s)
		if (e != nil) {return e}
		when, e := ParseTime(when_s)
		if (e != nil) {return e}
		Borrows.Set( bid, Tuple[time.Time, int64]{when, uid} )
	}

	return nil
}

func sql_add_book(ISBN, name string, published string) (int64, error) {
	res, e := SQLDo("lsys add book", `
INSERT INTO books
	(ISBN, name, published)
VALUES
	(?, ?, ?)
`, ISBN, name, published)
	if (e != nil) {return 0, e}
	return res.LastInsertId()
}

func init() {
	AvaliableBooks.Init()
	AllBooks.Init()
	IDToAuthor.Init()
	AuthorToId.Init()
	Borrows.Init()
	UIDToToBorrowRequest.Init()

	AttachInfo("lsys", "is_worker", "BOOL NOT NULL DEFAULT FALSE")

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
	next_borrow TEXT,
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
`

