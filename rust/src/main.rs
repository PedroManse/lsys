// library system

mod time;
mod sql;
//mod client {}
//mod worker {}

use std::str::FromStr;

use axum::{
	routing::get,
	extract::Query,
};
//use axum::Form;
use maud::{html, Markup};
use serde::Deserialize;

#[tokio::main]
async fn main() {
	let conn = sql::open("db").unwrap();
	conn.schema(sql::SQL_TABLE_SCHEMA).unwrap();

	//let pub_date = ex(time::Date::from_str("1-12-1887"))?;
	//let b = ex(Book::new(&conn, 9780140439083, "A Study In Scarlet".to_string(), vec![], pub_date))?;

	//let mut stmt = conn.prepare("SELECT * FROM books").unwrap();
	//let rows = stmt.query_map([], Book::from_query).unwrap();
	//let bks: Vec<Book> = rows.filter(|a|a.is_ok()).map(|b|b.unwrap()).collect();

	let app = axum::Router::new()
		.route("/", get(display_search) )
		.route("/list", get(display_all) )
		.route("/search", get(perform_search) );

	let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

async fn display_all() -> Markup {
	let conn = sql::open("db").unwrap();
	let mut stmt = conn.prepare("SELECT * FROM books").unwrap();
	let rows = stmt.query_map([], Book::from_query).unwrap();
	let books: Vec<Book> = rows.filter(|a|a.is_ok()).map(|b|b.unwrap()).collect();

	html! { body{
		table {

			thead{ tr {
				td { "ISBN" }
				td { "Name" }
				td { "Authors" }
				td { "Published Date" }
			} }

			tbody{
				@for book in &books {
					tr{
						th { (book.ISBN) }
						td { (book.name) }
						td { ("idk") }
						td { (book.published.to_str_split("/")) }
					}
				}
			}
		}
	} }
}

async fn display_search() -> Markup {
	html! { head{
		script src="https://unpkg.com/htmx.org@1.9.10" {}
	}
	body{
		form hx-get="/search" hx-target="#rser" hx-swap="innerHTML" {
			input name="ISBN" type="number" { }
			input name="name" type="string" { }
			button { "enviar" }
		}
		div id="rser" { }
	} }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case, dead_code)] // for ISBN
pub struct BookQuery {
	ISBN: Option<String>,
	name: Option<String>,
}

async fn perform_search(Query(qry): Query<BookQuery>) -> Markup {
	println!("{:?}", qry);

	html! {
		table {
			thead {
				tr{
					th { "ISBN" }
					th { "Name" }
					th { "Authors" }
					th { "Date" }
				}
			}
			tbody {
				tr{
					td {({ qry.ISBN.map(|i|i.to_string()).unwrap_or("Undefined".to_string()) })}
					td {({ qry.name.unwrap_or("Undefined".to_string()) })}
				}
			}
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case, dead_code)] // for ISBN
pub struct Book {
	ISBN: u64,
	internal_id: i64,

	name: String,
	authors: Vec<String>,
	published: time::Date,

	last_borrow: Option<time::Date>,
	borrower_id: Option<u64>, // ID of current borrower
}

impl Book {
	pub fn from_query(query: &rusqlite::Row) -> Result<Book, rusqlite::Error> {
		let ISBN:u64 = query.get(0)?;
		let internal_id:i64 = query.get(1)?;
		let name:String = query.get(2)?;
		let authors = vec![]; // TODO read authors
		let pub_date:String = query.get(3)?;
		let published = time::Date::from_str(&pub_date).map_err(|_|rusqlite::Error::InvalidQuery)?;
		let last_borrow_date:Option<String> = query.get(4)?;
		let last_borrow = last_borrow_date.map(|b_date|time::Date::from_str(&b_date).unwrap());
		let borrower_id:Option<u64> = query.get(5)?;

		Ok(Book{
			ISBN, internal_id, name,
			authors, published,
			last_borrow, borrower_id,
		})
	}

	pub fn new(
		db: &sql::DB,
		ISBN: u64, name: String,
		authors: Vec<String>, published: time::Date
	) -> Result<Book, rusqlite::Error> {
		// TODO authors in sql too!
		let internal_id = Book::add_to_db(db, ISBN, &name, published)?;

		Ok(Book{
			ISBN, internal_id, name,
			authors, published,
			last_borrow: None, borrower_id: None,
		})
	}

	fn add_to_db(
		db: &sql::DB,
		ISBN: u64, name: &str, published: time::Date
	) -> Result<i64, rusqlite::Error> {
		db.insert(r#"
INSERT INTO books
	(ISBN, name, published)
VALUES
	(?, ?, ?)
		"#, (
			ISBN, name,
			published,
		))
	}
}

