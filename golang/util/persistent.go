package util

import (
	"database/sql"
	_ "github.com/mattn/go-sqlite3"
)

var db *sql.DB

var SQLArea = FLog.NewArea("SQL")

//TODO: mayber SQLScript reading file
type SQLScript struct {
	Name string
	Code string
}

var init_scripts = []SQLScript{}
func SQLInitScript(Name string, Code string) {
	init_scripts = append( init_scripts, SQLScript{Name, Code})
}

func SQLGetSingle(Name, Query string, vars... any) ( info *sql.Row ) {
	return db.QueryRow(Query, vars...)
}

func SQLGet(Name, Query string, vars... any) ( info *sql.Rows, err error) {
	info, err = db.Query(Query, vars...)
	if (err != nil) {
		FLog.Printf(SQLArea, "\x1b[31mFailed executing dynamic script\x1b[0m [%s]: %v\n%s with %+v", Name, err, Query, vars)
	}
	return
}

func SQLDo(Name, Query string, vars... any) ( info sql.Result, err error) {
	info, err = db.Exec(Query, vars...)
	if (err != nil) {
		FLog.Printf(SQLArea, "\x1b[31mFailed executing dynamic script\x1b[0m [%s]: %v\n%s with %+v", Name, err, Query, vars)
	}
	return
}

type SQLFunc struct {
	Name string
	Func func(*sql.DB) error
}
var init_funcs = []SQLFunc{}

func SQLInitFunc(Name string, Func func(*sql.DB) error) {
	init_funcs = append( init_funcs, SQLFunc{Name, Func})
}

func InitSQL(dbfile string) error {
	var err error
	db, err = sql.Open("sqlite3", dbfile)
	if (err != nil) {
		FLog.Printf(SQLArea, "\x1b[31mFailed openning %q with sqlite3 drivers\x1b[0m", dbfile)
		return err
	}
	FLog.Printf(SQLArea, "Successefully openned %q with sqlite3 drivers", dbfile)

	for _, script :=range init_scripts {
		_, err = db.Exec(script.Code)

		if (err != nil) {
			FLog.Printf(SQLArea, "\x1b[31mFailed executing script\x1b[0m [%s]: %v\n%s", script.Name, err, script.Code)
			return err
		}
		FLog.Printf(SQLArea, "script [%s] executed successefully", script.Name)
	}

	for _, fnc :=range init_funcs {
		err = fnc.Func(db)

		if (err != nil) {
			FLog.Printf(SQLArea, "\x1b[31mFailed executing func\x1b[0m [%s]: %v", fnc.Name, err)
			return err
		}
		FLog.Printf(SQLArea, "func [%s] executed successefully", fnc.Name)
	}
	return nil
}

func StopSQL() {
	db.Close()
}
