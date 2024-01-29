package main

import (
	"net/http"
	. "mysrv/util"
	"mysrv/service"
)

func CreateHandler(w HttpWriter, r HttpReq, info map[string]any) (render bool, ret_r any) {
	ret_r = make(map[string]any)
	ret := ret_r.(map[string]any)
	ret["failed"] = false

	if (r.Method == "GET") {
		return true, ret
	}

	email := r.FormValue("email")
	username := r.FormValue("username")
	password := r.FormValue("password")
	if ( email == "" || username == "" || password == "" ) {
		ret["failed"] = true
		ret["failReason"] = "Missing account parameter"
		return true, ret
	}

	acc := NewAccount(email, username, password)
	if (acc == nil) {
		ret["failed"] = true
		ret["failReason"] = "Email already in use"
		ret["failEmail"] = true
		return true, ret
	}

	acc.SendCookie(w)
	http.Redirect(w, r, "/", http.StatusSeeOther)
	return false, ret
}

func LoginHandler(w HttpWriter, r HttpReq, info map[string]any) (render bool, ret_r any) {
	ret_r = make(map[string]any)
	ret := ret_r.(map[string]any)
	ret["failed"] = false

	if (r.Method == "GET") {
		return true, ret
	}

	email := r.FormValue("email")
	password := r.FormValue("password")
	if ( email == "" || password == "" ) {
		ret["failed"] = true
		ret["failReason"] = "Missing login parameter"
		return true, ret
	}

	acc, exists := GetAccount(email)
	if (!exists || acc.Hash != Hash(password)) {
		ret["failed"] = true
		ret["failReason"] = "Wrong password or email"
		return true, ret
	}

	acc.SendCookie(w)
	http.Redirect(w, r, "/", http.StatusSeeOther)
	return false, ret
}

var ( // system pages
	index = LogicPage(
		"html/sys/index.gohtml", nil,
		[]GOTMPlugin{GOTM_account},
		func (w HttpWriter, r HttpReq, info map[string]any) (bool, any) {
			if (r.URL.Path != "/") { missing.ServeHTTP(w, r) }
			return r.URL.Path == "/", nil
		},
	)
	register = LogicPage(
		"html/sys/register.gohtml", nil,
		[]GOTMPlugin{GOTM_account},
		CreateHandler,
	)
	login = LogicPage(
		"html/sys/login.gohtml", nil,
		[]GOTMPlugin{GOTM_account},
		LoginHandler,
	)

	missing = TemplatePage(
		"html/sys/missing.gohtml", nil,
		[]GOTMPlugin{GOTM_account, GOTM_urlInfo, GOTM_log},
	)
	users = TemplatePage(
		"html/sys/users.gohtml", nil,
		[]GOTMPlugin{GOTM_account, GOTM_mustacc, GOTM_accounts},
	)
)

func main() {
	InitSQL("sqlite3.db")

	// site-wide service
	http.Handle("/", index)
	http.Handle("/users", users)
	http.Handle("/login", login)
	http.Handle("/register", register)
	http.Handle("/favicon.ico", StaticFile("./files/dice.ico"))
	http.Handle("/files/", http.StripPrefix("/files", http.FileServer(http.Dir("./files/"))))

	http.Handle("/lsys/list/all", service.ListAllBooks)

	// real-time WebSocket chat
	FLog.Printf(FLOG_INFO, "Running")
	panic(http.ListenAndServe("0.0.0.0:8080", nil))
}
