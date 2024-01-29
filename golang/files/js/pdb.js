"use strict";

const id = document.getElementById.bind(document);
const query = document.querySelector.bind(document);
const queryAll = document.querySelectorAll.bind(document);
const cquery = (doc, search) => doc.querySelectorAll(search);
const cqueryAll = (doc, search) => doc.querySelectorAll(search);

function prepareSlot(slot) {
	const row = slot.getAttribute("row");
	const col = slot.getAttribute("col");

	slot.addEventListener("change", () => {
		const data = slot.value

		fetch("/fspdb", {
			method: "POST",
			body: JSON.stringify({row, col, data}),
		})
	})
}

const colCount = () => Number(query("tr").nextElementSibling.lastElementChild.children[0].getAttribute("col"))+1
const rowCount = () => queryAll("input.slot").length/colCount()
const asize = (count) => Array(count).fill(undefined)

function remCol({target}) {
	fetch("/fspdb", {
		method: "DELETE",
		body: JSON.stringify({
			col:target.getAttribute("col")
		})
	}).then(a=>{
		location.reload()
		//target.parentElement.remove()
		//TODO update other rows/cols
	})
}

function remRow({target}) {
	fetch("/fspdb", {
		method: "DELETE",
		body: JSON.stringify({
			row:target.getAttribute("row")
		})
	}).then(a=>{
		location.reload()
		//TODO update other rows/cols
	})
}

function addRow() {
	fetch("/fspdb", {
		method: "PATCH",
		body: JSON.stringify({
			col: colCount().toString(),
			row: (rowCount()+1).toString(),
		})
	}).then(a=>{
		const rrow = el("button", "X", {
			class: "rem rem-row", type:"button", row: rowCount().toString()
		});
		prepareRow(rrow);
		//rrow.addEventListener("click", remRow);
		const newrow = el("tr", {class:"row"}, [
			rrow,
			...asize(colCount()).map((_, i)=>
				el("td", [el(
					"input", {
						row:rowCount().toString(),
						col:i.toString(),
						type: "text", class: "slot",
					},
				)])
			)
		]);
		console.log(cqueryAll(newrow, "input.slot"))
		cqueryAll(newrow, "input.slot").forEach(prepareSlot);
		query("tbody").insertBefore(
			newrow, query("tbody").lastElementChild
		)
	})
}

function addCol() {
	fetch("/fspdb", {
		method: "PATCH",
		body: JSON.stringify({
			row: rowCount().toString(),
			col: (colCount()+1).toString(),
		})
	}).then(a=>{
		query("tbody > tr.tool-row").insertBefore(
			el("th", {scope:"col"}, [
				el("button", "X", {
					type: "button", class:"rem rem-col", col:colCount()
				})
			]),
			id("add-col").parentElement
		).addEventListener("click", remCol)

		cqueryAll(query("tbody"), "tr.row").forEach((tr, rw)=>{
			prepareSlot(tr.appendChild(el(
				"td", [el("input", {
					class: "slot",
					row: rw.toString(),
					col: colCount().toString(),
				})],
			)));
		})
	})
}

function prepareRow(row) {
	row.addEventListener("click", remRow);
}

function prepareCol(col) {
	col.addEventListener("click", remCol);
}

window.onload = () => {
	queryAll("input.slot").forEach(prepareSlot);
	queryAll("button.rem-row").forEach(prepareRow);
	queryAll("button.rem-col").forEach(prepareCol);
	id("add-col").addEventListener("click", addCol);
	id("add-row").addEventListener("click", addRow);
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
