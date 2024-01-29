package util

import (
	"time"
	"fmt"
	"io"
	"os"
	"strings"
)

type Area struct {
	ID uint64
	Text string
	Enabled bool
}

type Section []Area
func (Sec Section) String(selection uint64, AD, SOA, EOA string) (out string) {
	var selected bool
	var ars = []string{}
	for _, area := range Sec {
		if (!area.Enabled) {continue}

		selected = (area.ID&selection) != 0
		if (!selected) {continue}
		ars = append(ars, SOA+area.Text+EOA)
	}
	return strings.Join(ars, AD)
}

// if you need an io.Writer you can call Flogger.Writer() pre-speficy the
// selected areas and recieve an FSection, that implements io.Writer
type Flogger struct {
	File io.Writer
	Areas Section

	// formatting:
	StartOfLine string

	TimeFormat string

	BeforeAreas string

	// handled by Section
	StartOfArea string
	AreaDivision string
	EndOfArea string

	AfterAreas string
	// content
	EndOfLine string
}

const (
	ansi_CLEAR = "\x1b[0m"
	ErrNoEnabledAreas = constError("Flogger.Printf no selected areas")
	ErrNoAreas = constError("Flogger.Printf all selected areas disabled")
)

func NewLogger(out io.Writer) *Flogger {
	return &Flogger{
		File: out,
		Areas: []Area{},
		StartOfLine: "[",
		TimeFormat: "02/01 15:04:05",
		BeforeAreas: "] ",
		StartOfArea: "",
		AreaDivision: ansi_CLEAR+" | ",
		EndOfArea: ansi_CLEAR,
		AfterAreas: ": ",
		EndOfLine: ansi_CLEAR+"\n",
	}
}

func (Fl *Flogger) NewArea(text string) (ID uint64) {
	ID = 1<<uint64(len(Fl.Areas))
	Fl.Areas = append(Fl.Areas, Area{ID, text, true})
	return
}

func (Fl *Flogger) Printf(areas uint64, format string, stuff ...any) (int, error) {
	if (areas == 0) {return 0, ErrNoAreas}
	areasText := Fl.Areas.String(areas, Fl.AreaDivision, Fl.StartOfArea, Fl.EndOfArea)
	if (len(areasText) == 0) {return 0, ErrNoEnabledAreas}
	fmttime := time.Now().Format(Fl.TimeFormat)
	fmtcont := fmt.Sprintf(format, stuff...)
	return Fl.File.Write( []byte(
Fl.StartOfLine + fmttime + Fl.BeforeAreas + areasText + Fl.AfterAreas + fmtcont + Fl.EndOfLine,
	))
}

func (Fl Flogger) Enable(areas uint64) {
	for i, area := range Fl.Areas {
		if (area.ID & areas != 0) {
			Fl.Areas[i].Enabled = true
		}
	}
}

func (Fl Flogger) Disable(areas uint64) {
	for i, area := range Fl.Areas {
		if (area.ID & areas != 0) {
			Fl.Areas[i].Enabled = false
		}
	}
}

func (Fl Flogger) Toggle(areas uint64) {
	for i, area := range Fl.Areas {
		if (area.ID & areas != 0) {
			Fl.Areas[i].Enabled = !area.Enabled
		}
	}
}

func (Fl Flogger) Writer(areas uint64) FSection {
	return FSection{Fl, areas}
}

// wrapper for io.Writer but with Flogger info
type FSection struct {
	FLog Flogger
	SelectedAreas uint64
}

// doens't implement Write as documented, since formatting adds more information.
// If you need to access the original io.Writer, you can use FS.FLog.File
func (FS FSection) Write(p []byte) (n int, err error) {
	_, err = FS.FLog.Printf(FS.SelectedAreas, "%s", p)
	return len(p), err
}

// default Flogger

var FLog *Flogger = NewLogger(os.Stderr)
var FLOG_ERROR = FLog.NewArea("\x1b[31m\x1b[5mERROR")
var FLOG_INFO = FLog.NewArea("INFO")

