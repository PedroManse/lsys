"use strict";

const id = document.getElementById.bind(document);
const query = document.querySelector.bind(document);
const queryAll = document.querySelectorAll.bind(document);
const cquery = (doc, search) => doc.querySelectorAll(search);
const cqueryAll = (doc, search) => doc.querySelectorAll(search);

let questions = [];

window.onload = () => {
	id("question-creator").addEventListener("click", () => {
		const type = id("question-type").value;
		const qtext = id("question-text").value;
		if (type === "not selected") return; // TODO: error popup
		const qid = questions.lenght;
		id("content").appendChild( qtype_dictionary[type](qtext, qid) )
	})
}

const qtype_dictionary = {
	"text": textQuestion,
}

function selectQuestion(question) {
	questions.push({type: "select", question});
	return el("span", [
		el("label", { contenteditable: "true" }, question),
		el("br"),
		el("textarea"),
	])
}

function textQuestion(question) {
	questions.push({type: "text", question});
	return el("span", [
		el("label", { contenteditable: "true" }, question),
		el("br"),
		el("textarea"),
	])
}

const el = createElement;
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


