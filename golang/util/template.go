package util

import (
	unsafeTemplate "text/template"
	"html/template"
	"net/http"
	"io"
)

type HttpWriter = http.ResponseWriter
type HttpReq = *http.Request
type Plugin = func(w HttpWriter, r HttpReq, info map[string]any) (render bool, addinfo any)
// TODO: add "terminator" flag to plugin
// GOTM_mustacc is added as a guard, to relieve the
// : programmer of the duty to check if the user is logged in
// : plugins after GOTM_mustacc shoudln't be executed
type GOTMPlugin struct {
	Name string
	Plug Plugin
}

/*

Content Server/Renderer Creators
Static(File|Component)
Templated(Logiced)?(Plugged)?Page
Dynamic(Plugged)?Page
TemplatedComponent

NOTE if the Content Server has a render engine, and info map
(map[string]any) can be provided (or may be nil). That info map will be
passed to the render engine as the root of information

Plugged defines if there should be an []GOTMPlugin to run before rendering.
NOTE GOTMPlugin can read/write to requests, read/write to the info map, stop
execution of futher plugins and disabled rendering for a request

Templated attaches a Template file to the Content Server
NOTE /templates/*.gohtml will be parsed before the template file
Logiced attaches a user defined function to the end of the PluginList
NOTE Logiced may be used even if Plugged is no specified

Dynamic is TemplatedLogiced without the Template

Page means the creator retusn a ContentServer, that implements:
interaface {
	ServeHTTP (w HttpWriter, r HttpReq)
}

Component mans the creator return a ContentRenderer, that implements:
interaface {
	Render(io.Writer, any)
	RenderString(any) string
}

*/

type ContentServer = http.Handler
type ContentRenderer interface {
	Render(w io.Writer, info any)
	RenderString(info any) string
}

/*
TemplatedComponent
InlineComponent
InlineUnsafeComponent
DynamicPluggedPage
DynamicPage
TemplatedLogicedPluggedPage
TemplatedPluggedPage
TemplatedLogicedPage
TemplatedPage
StaticFile
StaticComponent
TemplatePage
LogicPage
*/

func TemplatedComponent ( filename string ) ContentRenderer {
	return templatedComponent{tmpl(filename)}
}

func InlineComponent ( filename string ) ContentRenderer {
	return templatedComponent{inlinetmpl(filename)}
}

func InlineUnsafeComponent ( filename string ) ContentRenderer {
	return templatedComponent{inlinetmpl_u(filename)}
}

// DynamicPage without plugins is just a user function
func DynamicPluggedPage(info map[string]any, plugins []GOTMPlugin, fn Plugin) ContentServer {
	return dynamicPage{ infomap(info), logicpluglist(fn, plugins) }
}
var DynamicPage = DynamicPluggedPage

func TemplatedLogicedPluggedPage(file string, info map[string]any, plugins []GOTMPlugin, fn Plugin) (ContentServer) {
	return templatedPage{ tmpl(file), infomap(info), logicpluglist(fn, plugins) }
}

func TemplatedPluggedPage(file string, info map[string]any, plugins []GOTMPlugin) (ContentServer) {
	return templatedPage{ tmpl(file), infomap(info), pluglist(plugins) }
}

func TemplatedLogicedPage(file string, info map[string]any, fn Plugin) (ContentServer) {
	return templatedPage{ tmpl(file), infomap(info), logicpluglist(fn, nil) }
}

func TemplatedPage(file string, info map[string]any) (ContentServer) {
	return templatedPage{ tmpl(file), infomap(info), pluglist(nil) }
}

func StaticFile(filename string) ContentServer { return staticServer{filename} }
var StaticComponent = StaticFile

// legacy alliases
var LogicPage = TemplatedLogicedPluggedPage
var TemplatePage = TemplatedPluggedPage

// the 4 Content Servers
// as is
type staticServer struct { Filename string }
func (s staticServer) ServeHTTP (w HttpWriter, r HttpReq) {
	http.ServeFile(w, r, s.Filename)
}

type anyTemplate interface {
	Execute(io.Writer, any) error
}

// only render
type templatedComponent struct {
	Template anyTemplate
}
func (Tc templatedComponent) Render(w io.Writer, einfo any) {
	e := Tc.Template.Execute(w, einfo)
	if ( e != nil ) {panic(e)}
}

func (Tc templatedComponent) RenderString(einfo any) (s string) {
	var WB = WriteBuffer{}
	WB.Init()
	e := Tc.Template.Execute(WB, einfo)
	if ( e != nil ) {panic(e)}
	return WB.String()
}

func (Tc templatedComponent) RenderBytes(einfo any) (s []byte) {
	var WB = WriteBuffer{}
	e := Tc.Template.Execute(WB, einfo)
	if ( e != nil ) {panic(e)}
	return WB.Bytes()
}

// only preprocess/process
type dynamicPage struct {
	Info map[string]any
	Plugins []GOTMPlugin
}

func (dp dynamicPage) ServeHTTP (w HttpWriter, r HttpReq) {
	for _, plug := range dp.Plugins {
		_, dp.Info[plug.Name] = plug.Plug(w, r, dp.Info)
	}
}

// preprocess/process, render
type templatedPage struct {
	Template anyTemplate
	Info map[string]any
	Plugins []GOTMPlugin
}

func (tp templatedPage) ServeHTTP (w HttpWriter, r HttpReq) {
	var render = true
	var prender bool

	for _, plug := range tp.Plugins {
		prender, tp.Info[plug.Name] = plug.Plug(w, r, tp.Info)
		render = render&&prender
	}

	if (render) {
		e := tp.Template.Execute(w, tp.Info)
		if (e != nil) {
			FLog.Printf(FLOG_ERROR, "Panic in Template %w", e)
			panic(e)
		}
	}
}

// Creator helper funcs
func tmpl(filename string) anyTemplate {
	return template.Must(
		template.Must(
			template.ParseFiles(filename),
		).ParseGlob("templates/*.gohtml"),
	)
}

// _u suffix means unsafe (non-html) template
func tmpl_u(filename string) anyTemplate {
	return unsafeTemplate.Must(
		unsafeTemplate.Must(
			unsafeTemplate.ParseFiles(filename),
		).ParseGlob("templates/*.gohtml"),
	)
}

func inlinetmpl(str string) anyTemplate {
	return template.Must(
		template.Must(
			template.New("inlined Template").Parse(str),
		).ParseGlob("templates/*.gohtml"),
	)
}

func inlinetmpl_u(str string) anyTemplate {
	return unsafeTemplate.Must(
		unsafeTemplate.Must(
			unsafeTemplate.New("inlined Template").Parse(str),
		).ParseGlob("templates/*.gohtml"),
	)
}

func infomap(inf map[string]any) map[string]any {
	if (inf == nil) {
		inf = make(map[string]any)
	}
	return inf
}

func logicpluglist(fn Plugin, plugins []GOTMPlugin) []GOTMPlugin {
	if (plugins == nil) {
		return []GOTMPlugin{GOTMPlugin{"logic", fn}}
	} else {
		return append(plugins, GOTMPlugin{"logic", fn})
	}
}

func pluglist(plugins []GOTMPlugin) []GOTMPlugin {
	if (plugins == nil) {
		plugins = []GOTMPlugin{}
	}
	return	plugins
}

