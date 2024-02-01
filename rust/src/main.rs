#![allow(dead_code)]
// library system

use axum::{
	Form,
	routing::get,
	extract::State,
};
use maud::{html, Markup};
use sqlx::sqlite::SqlitePoolOptions;;
use std::cell::Cell;
use tokio::sync::Mutex;
use std::sync::Arc;
use serde::Deserialize;

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

	let app = axum::Router::new()
		.route("/", get(display_all) )
		.route("/login", get(display_login).post(perform_login) )
		.with_state(new_shared_state(pool).await);

	let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

type SharedState = Arc<Mutex<ServerState>>;

async fn new_shared_state(db: sqlx::Pool<sqlx::Sqlite>) -> SharedState {
	let books = sqlx::query_as!(
		BookQuery,
		"SELECT time, is_borrow, id, ISBN, name, published, user_id FROM books;"
	).fetch_all(&db).await.expect("can't parse row from books into BookQuery");

	let books:Vec<Book> = books.iter()
		.map(Book::from_query)
		.collect();

	Arc::new( Mutex::new(
		ServerState{
			db,
			books,
		}
	))
}

#[derive(Deserialize, Debug)]
struct FormLogIn {
	email: String,
	pass: String,
}

async fn perform_login( Form(login): Form<FormLogIn> ) -> Markup {
	println!("{:b}", hash(login.pass.as_bytes()));

	html! {}
}

async fn display_login() -> Markup {
	html! { body {
		form method="POST" action="login" {
			input name="email" type="email" placeholder="email" {}
			input name="pass" type="password" placeholder="password" {}
			button { "LogIn" }
		}
	} }
}

#[derive(Deserialize, Debug)]
struct FormRegister {
	name: String,
	email: String,
	pass: u64, // .as_bytes()
}

async fn register(Form(register): Form<FormRegister>) -> Markup {
	println!("{register:#?}");
	html! {}
}

#[derive(Clone)]
struct ServerState {
	db: sqlx::Pool<sqlx::Sqlite>,
	books: Vec<Book>,
}

async fn display_all( State(stt): State<SharedState>) -> Markup {
	let state = stt.lock().await;
	let books = &state.books;

	html! { body{
		table {

			thead{ tr {
				td { "ISBN" }
				td { "Name" }
				td { "Authors" }
				td { "Published Date" }
			} }

			tbody{
				@for book in books {
					tr{
						th { (book.ISBN) }
						td { (book.name) }
						td { ("idk") }
						td { (book.published) }
					}
				}
			}
		}
	} }
}

type UID = i64;
#[derive(Debug)]
struct Account {
	uid: UID,
	name: String,
	email: String,
	pass_hash: String,
}

#[derive(Debug, Clone, Copy)]
enum BorrowStatus { Reserved, Borrowed }
impl BorrowStatus {
	pub fn from(is_borrow: bool) -> Self {
		if is_borrow {
			BorrowStatus::Borrowed
		} else {
			BorrowStatus::Reserved
		}
	}
}
#[derive(Debug, Clone, Copy)]
struct BookStatus {
	uid: UID,
	time: chrono::NaiveDate,
	status: BorrowStatus,
}

type BID = i64;
#[derive(Debug, Clone)]
#[allow(non_snake_case)]
struct Book {
	ISBN: i64,
	bid: BID,
	name: String,
	authors: Vec<String>,
	published: String,
	status: Option<Cell<BookStatus>>,
}
#[allow(non_snake_case)]
#[derive(Debug, Clone)]
struct BookQuery {
	ISBN: i64,
	id: BID,

	name: String,
	published: String,
	user_id: Option<UID>,
	time: Option<chrono::NaiveDate>,
	is_borrow: Option<bool>,
}

impl Book {
	pub fn from_query(info: &BookQuery) -> Self {
		// should not fail, since or is_borrow is NULL and time & user_id are also
		// or is_borrow is Some() and so are time & user_id
		let status = if let Some(is_borrow) = info.is_borrow {
			Some(
				Cell::new(BookStatus{
					uid: info.user_id.unwrap(),
					time: info.time.unwrap(),
					status: BorrowStatus::from(is_borrow),
				})
			)
		} else {
			None
		};
		Book{
			ISBN: info.ISBN.clone(),
			bid: info.id.clone(),
			name: info.name.clone(),
			authors: Vec::new(),
			published: info.published.clone(),
			status: status,
		}
	}

//	pub fn new(
//		db: &sql::DB,
//		ISBN: i64, name: String,
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
//		ISBN: i64, name: &str, published: time::Date
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
}

// djb2
fn hash(st: &[u8]) -> i64 {
	let mut hash: i64 = 5381;
	for chr in st {
		hash = 33*hash+(*chr as i64);
	}
	hash
}

/*

[worker] create new book
INSERT INTO books
	(ISBN, name, published)
VALUES
	(?, ?, ?);

[user] reserve book
UPDATE books SET
	(user_id, time, is_borrow)
VALUES
	(?, ?, false);

[worker] borrow book -- as per user reservation
UPDATE books SET
	(time, is_borrow)
VALUES
	(?, true)
WHERE
	(book_id == ?);

[worker] return book -- as per user IRL action
UPDATE books SET
	(user_id, time, is_borrow)
VALUES
	(NULL, NULL, NULL)
WHERE
	(book_id == ?);

[server] get books
SELECT
	id, ISBN, name, published, user_id, time, is_borrow
FROM
	books;

*/
