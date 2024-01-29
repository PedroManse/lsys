"use strict";

const WS_SERVER = `ws://${location.host}/wschat`
const id = document.getElementById.bind(document)

let socket = null;
let userId = null;
let userHash = null;

// idea, Package Connect to make use other WS services
function Connect(onmsg) {
	socket = new WebSocket(WS_SERVER);

	socket.addEventListener("close", (event) => {
		console.log(event);
		alert("WS CLOSED");
		window.location.reload();
	})

	socket.addEventListener("message", (event) => {
		// set-username / server hello
		if (userId === null) {
			const info = JSON.parse(event.data);
			userId = info.id
			userHash = info.hash

			socket.send(JSON.stringify({
				action:"set-username",
				id:userId,
				hash:userHash,
				info: {
					email,
					name,
				}
			}))
		} else {
			onmsg(JSON.parse(event.data));
		}
	});
}

function escapeHTML(text) {
	return text
		.replace(/&/g, "&amp;")
		.replace(/</g, "&lt;")
		.replace(/>/g, "&gt;");
}

const Replaceble = [
	["<", "&lt;"],
	[">", "&gt;"],
	//[/^{$/gm, "<div>" ],
	//[/^{#(.*?)(\.(.*))?$/gm, "<div id=\"$1\" class=\"$3\">"],
	//[/^{\.(.*?)#(.*)$/gm, "<div class=\"$1\" id=\"$2\">"],
	//[/^{\.(.*)$/gm, "<div class=\"$1\">"],
	//[/^}$/gm, "</div>"],
	/*
	``
	#include <stdio.h>
	int main() {
		printf("hello world!\n")
	}
	``
	*/
	[/^(\w+)``$/gm, "<code class=\"$1\">"],
	[/^``$/gm, "</code>"],
	//[tux]=(https://en.wikipedia.org/wiki/Tux_(mascot))
	[/\[(.*)\]=\((.*)\)/gm, "<a href\=\"$2\">$1</a>"],
	// [tux]!(https://en.wikipedia.org/wiki/Tux_(mascot)#/media/File:Tux.svg)
	[/\[(.*)\]!\((.*)\)/gm, "<img alt\=\"$1\" src\=\"$2\"></img>"],
	//TODO: include css that makes this work
	// _EOF_(End of File)
	[/\b(?<!\\)_(.*?)_\((.*?)\)/gm, "<span class=\"popup\" explanation=\"$2\"><i>$1</i></span>" ],
	// _EOF_
	[/\b(?<!\\)_(.*?)_\b/gm, "<i>$1</i>"],
	// ~REDACTED~
	[/\B(?<!\\)\~(.*?)\~\B/gm, "<strike>$1</strike>"],
	// *IMPORTANT*
	[/\B(?<!\\)\*(.*?)\*/gm, "<strong>$1</strong>"],
	[/ ;; /gm, "<br>"],
	// # H1 #
	// ### H3 ###
	// ###### H6 ######
	[/\B(?<!\\)###### (.*?) ######\B/gm, "<h6>$1</h6>"],
	[/\B(?<!\\)##### (.*?) #####\B/gm, "<h5>$1</h5>"],
	[/\B(?<!\\)#### (.*?) ####\B/gm, "<h4>$1</h4>"],
	[/\B(?<!\\)### (.*?) ###\B/gm, "<h3>$1</h3>"],
	[/\B(?<!\\)## (.*?) ##\B/gm, "<h2>$1</h2>"],
	[/\B(?<!\\)# (.*?) #\B/gm, "<h1>$1</h1>"],
	/*
	C`
	#include <stdio.h>
	int main() {
		printf("hello world!\n")
	}
	`
	*/
	[/^(\w*)?`(.*?)`/gm, "<code class=\"$1\">$2</code>"],
	// \_ -> _; \* -> * ...
	[/\\\*/gm, "\*"],
	[/\\_/gm, "_"],
	[/\\#/gm, "#"],
	[/\\\[/gm, "["],
	[/\\\]/gm, "]"],
];

function parseMD(text) {
	Replaceble.forEach((rnr)=>{
		text=text.replaceAll(rnr[0], rnr[1])
	})
	return text;
}

function onMessage({action, from, msg}) {
	switch (action) {
		case "user-msg":

			msg = escapeHTML(msg);
			msg = parseMD(msg);

			id("chatlog").appendChild(createElement(
				"div", {class:"chat-msg"}, [
					createElement("h3", {style: {color: "white"}}, from+":"),
					createElement("h3", {style: {color: "white"}}, msg),
				]
			))
			break;
		case "server-msg":
			id("chatlog").appendChild(createElement(
				"div", {class:"server-msg"}, [
					createElement("p", msg),
				]
			))
			break;
	}
}

window.onload = () => {
	Connect(onMessage)

	const sendmsg = ()=>{
		const msg = id("msg").value
		let rmsg = escapeHTML(msg);
		rmsg = parseMD(rmsg);
		id("chatlog").appendChild(createElement(
			"div", {class:"chat-msg"}, [
				createElement("h3", {style: {color: "white"}}, name+":"),
				createElement("h3", {style: {color: "white"}}, rmsg),
			]
		))

		socket.send(JSON.stringify({
			action:"message",
			id:userId,
			hash: userHash,
			info:{msg},
		}))
	}

	id("send").addEventListener("click", sendmsg)

	id("msg").addEventListener("keydown", ({key})=>{
		if (key === "Enter") {
			sendmsg();
			id("msg").value = "";
		}
	})
}

function createElement(name, elements=[], attributes=null) {
	if (typeof elements === "object" && !Array.isArray(elements)) {
		[elements, attributes] = [attributes, elements];
	}

	const el = document.createElement(name);
	for (const attr in attributes) {
		if (attr === "style") {
			for (const stl in attributes[attr]) {
				el.style[stl] = attributes[attr][stl];
			}
			continue;
		}
		el.setAttribute(attr, attributes[attr]);
	}
	if (Array.isArray(elements)) {
		el.append(...elements);
	} else if (typeof elements === "string") {
		el.innerHTML = elements;
	}
	return el;
}

