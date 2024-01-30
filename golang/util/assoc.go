package util

import (
	"fmt"
	"database/sql"
)

const assoc_script = `
CREATE TABLE IF NOT EXISTS assoc_data (
	%s
);
`

const assoc_closer = `accid INTEGER PRIMARY KEY,
	FOREIGN KEY(accid) REFERENCES accounts(id)
`

type AccountInfo = SyncMap[string, any]

type info struct {
	area string
	name string
	sqlcmd string
}
var infos = []info{}
var names []string
var AttachAccountInfo SyncMap[int64, AccountInfo]

// DataCommand not be non-nullable without defaut param
func AttachInfo(InfoArea, InfoName, DataCommand string) {
	// prepare DataCommand for next command
	infos = append(infos, info{InfoArea, InfoName, DataCommand})
	names = append(names, InfoName)
}

func UpdateAttachedInfo(InfoArea, InfoName string, accid int64, InfoValue any) (r sql.Result, e error) {
	qstring := fmt.Sprintf(`UPDATE assoc_data SET assoc_%s_%s=? WHERE accid=?`, InfoArea, InfoName)
	r, e = SQLDo("util/assoc.UpdateAttachedInfo#"+InfoArea, qstring, InfoValue, accid)
	if (e == nil) {
		info, ok := AttachAccountInfo.Get(accid)
		if (ok) {
			info.Set(InfoArea+"."+InfoName, InfoValue)
			AttachAccountInfo.Set(accid, info)
		}
	}
	return
}

func InitAssoc() {
	AttachAccountInfo.Init()

	var assoc_cmd = assoc_script

	for _, info := range infos {
		assoc_cmd = fmt.Sprintf(assoc_cmd, fmt.Sprintf("assoc_%s_%s %s,\n\t%%s", info.area, info.name, info.sqlcmd))
	}
	assoc_cmd = fmt.Sprintf(assoc_cmd, assoc_closer)

	_, e := SQLDo("util/assoc.InitAssoc#Create Table", assoc_cmd)
	if (e != nil) {panic(e)}
	if (len(names) == 0) {return} // nothing to select

	var qstring string
	for i, info := range infos {
		qstring+=fmt.Sprintf("assoc_%s_%s", info.area, info.name)
		if (i != len(infos)-1) {
			qstring+=","
		}
	}

	// god i hate this
	// can't use ("?", qstring) because sql escapes it into from a raw string
	rows, e := SQLGet("util/assoc.InitAssoc#Read Table", "SELECT accid, "+qstring+" FROM assoc_data;")
	if (e != nil) {panic(e)}
	defer rows.Close()
	for rows.Next() {
		var accinfo SyncMap[string, any]
		accinfo.Init()

		var infoGetters = make([]any, len(names)+1)
		for i := range infoGetters {
			infoGetters[i] = new(any)
		}

		e:=rows.Scan(infoGetters...)
		if (e != nil) {panic(e)}
		accid := (*infoGetters[0].(*any)).(int64)

		for index, info := range infos {
			accinfo.Set(info.area+"."+info.name, *(infoGetters[1+index].(*any)))
		}
		AttachAccountInfo.Set(accid, accinfo)
	}
	_, e = SQLDo("util/assoc.InitAssoc#Populate Table", `INSERT OR IGNORE INTO assoc_data (accid) SELECT id FROM accounts;`)
	if (e!=nil) {panic(e)}
}

func assoc_NewAccountEvent(acc Account) (die bool) {
	_, e := SQLDo(
		"util/assoc.NewAccountEvent#Insert in Table",
		`INSERT INTO assoc_data (accid) VALUES (?);`, acc.ID,
	)
	if (e != nil){panic(e)}

	// account assoc utility is not used
	if (len(names) == 0) {
		return
	}

	var qstring string
	for i, info := range infos {
		qstring+=fmt.Sprintf("assoc_%s_%s", info.area, info.name)
		if (i != len(infos)-1) {
			qstring+=","
		}
	}

	// god i hate this // can't use "?", qstring because sql escapes it into a raw string
	row := SQLGetSingle(
		"util/assoc.InitAssoc#Read Table",
		"SELECT "+qstring+" FROM assoc_data WHERE accid=?;", acc.ID,
	)

	var accinfo SyncMap[string, any]
	accinfo.Init()

	var infoGetters = make([]any, len(names))
	for i := range infoGetters {
		infoGetters[i] = new(any)
	}

	e=row.Scan(infoGetters...)
	if (e != nil) {panic(e)}

	for index, info := range infos {
		accinfo.Set(info.area+"."+info.name, *(infoGetters[index].(*any)))
	}
	AttachAccountInfo.Set(acc.ID, accinfo)

	return false
}

func init() {
	NewAccountEvent.Listen(assoc_NewAccountEvent)
}
