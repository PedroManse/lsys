package util

import (
	"sync"
	"strconv"
	"net/http"
	"fmt"
	"time"
	"strings"
	"database/sql"
)

type Account struct {
	ID int64
	Email string
	Name string
	Hash HashResult
}

var (
	// old
	accLock = sync.Mutex{}
	Accounts = make(map[string]*Account)

	// email<->ID
	EmailToID SyncMap[string, int64]
	IDToEmail SyncMap[int64, string]

	// (ID|email)->Acc
	EmailToAccount SyncMap[string, *Account]
	IDToAccount SyncMap[int64, *Account]
)

// [email]->Acc
func AccountsCopy() (cacc map[string]Account) {
	cacc = make(map[string]Account)
	for acc := range IDToAccount.IterValues() {
		cacc[acc.Email] = *acc
	}
	return cacc
}

const AccountCookieName = "mysrv-cookie"

func (A *Account) MakeCookieStr() string {
	return fmt.Sprintf("%s:%v", A.Email, A.Hash)
}

func ReadAccountCookie(str string) (email string, value HashResult, ok bool) {
	steps := strings.SplitN(str, ":", 2)
	if len(steps) != 2 { // no ":" in string
		return "", 0 ,false
	}
	value64, ValidHashResult := strconv.ParseUint(steps[1], 10, HashBitLen)
	if (ValidHashResult != nil) {
		return "", 0 ,false
	}
	return steps[0], HashResult(value64), true
}

func (A *Account)SendCookie(w HttpWriter) {
	SendCookie := http.Cookie{
		Name: AccountCookieName,
		Value: A.MakeCookieStr(),
		HttpOnly: true,
		//Secure: true, // not serving https yet
		Path: "/",
		SameSite: http.SameSiteStrictMode,
		Expires: time.Now().Add(time.Hour * 24 * 10), // 10 days
	}
	http.SetCookie(w, &SendCookie)
}

func GetAccount(email string) (acc *Account, exists bool) {
	return EmailToAccount.Get(email)
}

func NewAccount(email, name, password string) *Account {
	_, used := EmailToAccount.Get(email)
	if (used) { return nil }

	hash := Hash(password)

	a, err := db.Exec(
		`INSERT INTO accounts (email, name, hash) VALUES (?, ?, ?);`,
		email, name, hash,
	)
	if (err != nil) {
		panic(err)
	}
	ID, _ := a.LastInsertId()
	acc := &Account{ID, email, name, hash}

	EmailToID.Set(email, ID)
	EmailToAccount.Set(email, acc)
	IDToEmail.Set(ID, email)
	IDToAccount.Set(ID, acc)

	NewAccountEvent.Alert(*acc)

	return acc
}

func loadAccounts(db *sql.DB) error {
	rows, err := db.Query("SELECT id, email, name, hash FROM accounts")
	if err != nil { return err }

	for rows.Next() {
		var email, name string
		var ID int64
		var hash HashResult

		err = rows.Scan(&ID, &email, &name, &hash)
		if err != nil { return err }

		acc := &Account{ID, email, name, hash}
		EmailToID.Set(email, ID)
		EmailToAccount.Set(email, acc)
		IDToEmail.Set(ID, email)
		IDToAccount.Set(ID, acc)

	}
	return nil
}

var NewAccountEvent = Event[Account]{}
func init() {
	EmailToID.Init()
	EmailToAccount.Init()
	IDToEmail.Init()
	IDToAccount.Init()


	SQLInitScript( "accounts",
`CREATE TABLE IF NOT EXISTS accounts (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	email TEXT NOT NULL UNIQUE,
	name TEXT NOT NULL,
	hash INT NOT NULL
);`)
	SQLInitFunc( "accounts", loadAccounts )
}
