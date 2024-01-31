#![allow(dead_code)]
// library system

mod sql;
use axum::{
	routing::get,
	Extension,
	//extract::State,
};
//use axum::Form;
use maud::{html, Markup};
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() {
	dotenvy::dotenv().unwrap();
	let db_connection_str = std::env::var("DATABASE_URL")
		.expect("DATABASE_URL not set in env");

	// set up connection pool
	let pool = SqlitePoolOptions::new()
		.max_connections(5)
		.acquire_timeout(std::time::Duration::from_secs(3))
		.connect(&db_connection_str).await
		.expect("can't connect to database");


	sqlx::query(sql::TABLE_SCHEMA)
		.execute(&pool).await
		.expect("can't setup database schema");

	let sst = Arc::new(
		Mutex::new(
			ServerState{db: pool, visits: 0}
		)
	);
	//let sst = ServerState{db: pool, visits: 0};
	let app = axum::Router::new()
		.route("/", get(display_search) )
		.route("/list", get(display_all) )
		.layer(Extension(sst));

	let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

//use std::pin::Pin;
use tokio::sync::Mutex;
use std::sync::{Arc};

type SharedState = Arc<Mutex<ServerState>>;

#[derive(Clone)]
struct ServerState {
	db: sqlx::Pool<sqlx::Sqlite>,
	visits: u64,
}

use axum::debug_handler;
#[debug_handler]
async fn display_all(
	state: Extension<SharedState>
) -> Markup {
	//let stt = &state.lock().unwrap().db;
	////let mut state = state_rw.write().unwrap();
	//let book = sqlx::query!("SELECT * FROM books;")
	//	.fetch_one(stt).await
	//	.unwrap();
	//println!("{book:#?}");
	let mut stt = state.lock().await;
	stt.visits += 1;

	html! {body{
		h1 { "hi" }
		h1 { "we've had " (stt.visits) " hits to this page" }
	}}

	//html! { body{
	//	table {

	//		thead{ tr {
	//			td { "ISBN" }
	//			td { "Name" }
	//			td { "Authors" }
	//			td { "Published Date" }
	//		} }

	//		tbody{
	//			@for book in &books {
	//				tr{
	//					th { (book.ISBN) }
	//					td { (book.name) }
	//					td { ("idk") }
	//					td { (book.published.to_str_split("/")) }
	//				}
	//			}
	//		}
	//	}
	//} }
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

type UID = u64;
#[derive(Debug)]
struct Account {
	uid: UID,
	name: String,
	email: String,
	pass_hash: String,
}

#[derive(Debug, Clone)]
enum Status {
	Reserved(UID),
	Borrowed(UID),
}

type BID = u64;
#[derive(Debug, Clone)]
#[allow(non_snake_case, dead_code)] // for ISBN
struct Book {
	ISBN: u64,
	bid: BID,

	name: String,
	authors: Vec<String>,
	published: String,

	status: Option<Status>,
	borrower_id: Option<u64>, // ID of current borrower
}

//impl Book {
//	pub fn new(
//		db: &sql::DB,
//		ISBN: u64, name: String,
//		authors: Vec<String>, published: time::Date
//	) -> Result<Book, rusqlite::Error> {
//		// TODO authors in sql too!
//		let internal_id = Book::add_to_db(db, ISBN, &name, published)?;
//
//		Ok(Book{
//			ISBN, internal_id, name,
//			authors, published,
//			last_borrow: None, borrower_id: None,
//		})
//	}
//
//	fn add_to_db(
//		db: &sql::DB,
//		ISBN: u64, name: &str, published: time::Date
//	) -> Result<i64, rusqlite::Error> {
//		db.insert(r#"
//INSERT INTO books
//	(ISBN, name, published)
//VALUES
//	(?, ?, ?)
//		"#, (
//			ISBN, name,
//			published,
//		))
//	}
//}

