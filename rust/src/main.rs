#![allow(dead_code)]
// library system

//TODO minijinja
use axum::{
	Form,
	routing::get,
	extract::State,
	response::Redirect,
	extract::Query,
};
#[allow(unused_imports)]
use axum::debug_handler;
use maud::{html, Markup};
use sqlx::sqlite::SqlitePoolOptions;
use serde::{Deserialize};
use std::{
	cell::Cell,
	sync::Arc,
	collections::HashMap,
};
use uuid::Uuid;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

const COOKIE_UUID_NAME: &str = "lsys-uuid";
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
		.route("/register", get(display_login).post(perform_register) )
		.layer(CookieManagerLayer::new())
		.with_state(new_shared_state(pool).await);

	let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

type SharedState = Arc<tokio::sync::Mutex<ServerState>>;

async fn new_shared_state(db: sqlx::Pool<sqlx::Sqlite>) -> SharedState {
	let books = sqlx::query_as!(
		BookQuery,
		"SELECT time, is_borrow, id, ISBN, name, published, user_id FROM books;"
	).fetch_all(&db).await.expect("can't parse row from books into BookQuery");

	let books:Vec<Book> = books.iter()
		.map(Book::from_query)
		.collect();

	let accounts = sqlx::query_as!(
		AccountQuery,
		"SELECT id, name, email, pass_hash, is_worker FROM accounts;",
	).fetch_all(&db).await.expect("can't parse row from accounts into AccountQuery");

	let mut state = ServerState{
		db,
		books,
		uuid_to_account: HashMap::new(),
		uid_to_account: HashMap::new(),
		email_to_uid: HashMap::new(),
	};

	accounts.iter()
		.map(Account::from_query)
		.for_each( |acc| Account::update_maps(&mut state, acc) );


	Arc::new( tokio::sync::Mutex::new( state ))
}

#[debug_handler]
async fn perform_login(
	State(stt): State<SharedState>,
	cookies: Cookies,
	Query(goto): Query<Goto>,
	Form(login): Form<FormLogin>
) -> Result<Redirect, Markup> {
	let state = Arc::clone(&stt);
	let state = state.lock().await;

	let goto = match &goto.goto {
		Some(goto)=>goto.as_str(),
		None=>"/",
	};

	let acc = state.email_to_uid.get(&login.email).ok_or(view_login("No such email", "", goto))?;
	let acc = state.uid_to_account.get(&acc).ok_or(view_login("Can't find account", "", goto))?;
	let pass_hash = hash_pass(login.pass.as_bytes());
	if acc.pass_hash != pass_hash {
		Err(view_login("Wrong password", "", goto))
	} else {
		cookies.add(
			Cookie::build((COOKIE_UUID_NAME, acc.uuid.to_string()))
				.path("/")
				.secure(false)
				.http_only(true).into()
		);
		Ok(Redirect::to( goto ))
	}
}

async fn display_login(
	Query(goto): Query<Goto>,
) -> Markup {
	let goto = match &goto.goto {
		Some(goto)=>goto.as_str(),
		None=>"/",
	};

	view_login("", "", goto)
}

async fn perform_register(
	State(stt): State<SharedState>,
	cookies: Cookies,
	Query(goto): Query<Goto>,
	Form(register): Form<FormRegister>,
) -> Result<Redirect, Markup> {

	let goto = match &goto.goto {
		Some(goto)=>goto.as_str(),
		None=>"/",
	};
	let mut state = stt.lock().await;
	let acc = Account::new(&state.db, register).await;
	let acc = acc.map_err(|e| view_login("", e.as_str(), goto))?;

	cookies.add(
		Cookie::build((COOKIE_UUID_NAME, acc.uuid.to_string()))
			.path("/")
			.secure(false)
			.http_only(true).into()
	);
	Account::update_maps(&mut state, acc);
	println!("{state:#?}");

	Ok( Redirect::to("/") )
}

async fn display_all(
	State(stt): State<SharedState>,
	cookies: Cookies,
) -> Result<Markup, Redirect> {
	let state = Arc::clone(&stt);
	let state = state.lock().await;

	let books = &state.books;

	//TODO make into function
	let cookie = cookies.get(COOKIE_UUID_NAME);
	let acc = cookie.ok_or(Redirect::to("/login"))?.value().to_owned();
	let acc = Uuid::parse_str(&acc).or(Err(Redirect::to("/login")))?;
	let acc = state.uuid_to_account.get(&acc).ok_or(Redirect::to("/login"))?;
	println!("{:#?}", acc);

	Ok(view_all_books(books))
}

// password String -> hash i64 -> [u8] -> v3_uuid String
impl Account {
	fn update_maps(state: &mut ServerState, acc: Self) {
		//TODO: test Arc without Box
		let acc = Arc::new(Box::new(acc));
		state.uuid_to_account.insert(acc.uuid, Arc::clone(&acc));
		state.uid_to_account.insert(acc.uid, Arc::clone(&acc));
		state.email_to_uid.insert(acc.email.clone(), acc.uid);
	}

	async fn new(
		db: &sqlx::Pool<sqlx::Sqlite>,
		form: FormRegister,
	) -> Result<Self, String> {
		if form.name.len() == 0 || form.email.len() == 0 || form.pass.len() == 0 {
			return Err("Field with not input".to_string());
		}

		let pass_hash = hash_pass(form.pass.as_bytes());
		let result = sqlx::query!(
	"INSERT INTO accounts
		(name, email, pass_hash)
	VALUES
		(?, ?, ?)", form.name, form.email, pass_hash,
	).execute(db).await;

		let result = result.map_err(|e|e.to_string())?;
		let uid = result.last_insert_rowid();

		// TODO check for repeated UUIDs even if UUID level unlikely
		Ok(Account{
			uid: uid as UID,
			name: form.name,
			email: form.email,
			pass_hash,
			uuid: Uuid::new_v4(),
			is_worker: false,
		})
	}

	fn from_query(info: &AccountQuery) -> Self {
		Self{
			uid: info.id as UID,
			name: info.name.clone(),
			email: info.email.clone(),
			pass_hash: info.pass_hash.clone(),
			uuid: Uuid::new_v4(),
			is_worker: info.is_worker,
		}
	}
}

impl Book {
	fn from_query(info: &BookQuery) -> Self {
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
}

fn view_login(log_error: &str, reg_error: &str, goto: &str) -> Markup {
	html! { body {
		p style="color: red;"{(log_error)}

		form method="POST" action=({format!("login?goto={goto}")}) {
			label for="login-email" {"Email:"}
			input id="login-email" name="email" type="email" placeholder="email" {}
			"  "
			label for="login-pass" {"Password:"}
			input id="login-pass" name="pass" type="password" placeholder="password" {}
			button { "LogIn" }
		}

		p style="color: red;"{(reg_error)}
		form method="POST" action=({format!("register?goto=/{goto}")}) {
			label for="register-username" {"User Name:"}
			input id="register-username" name="name" type="text" placeholder="username" {}
			"  "
			label for="register-email" {"Email:"}
			input id="register-email" name="email" type="email" placeholder="email" {}
			"  "
			label for="register-password" {"Password:"}
			input id="register-password" name="pass" type="password" placeholder="password" {}
			button { "Register" }
		}
	} }
}

fn view_all_books(books: &Vec<Book>) -> Markup {
	html! { body{
		table {

			thead{ tr {
				td { "ISBN" }
				td { "Name" }
				td { "Authors" }
				td { "Published Date" }
			} }

			tbody{
			@for book in books { tr{
				th { (book.ISBN) }
				td { (book.name) }
				td { ("idk") }
				td { (book.published) }
			} }
			}
		}
	} }
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

//TODO: use sub-state to account maps, book maps
#[derive(Clone, Debug)]
struct ServerState {
	db: sqlx::Pool<sqlx::Sqlite>,
	books: Vec<Book>,
	uuid_to_account: HashMap<Uuid, Arc<Box<Account>>>,
	uid_to_account: HashMap<UID, Arc<Box<Account>>>,
	email_to_uid: HashMap<String, UID>,
}

#[derive(Deserialize, Debug)]
struct FormLogin {
	email: String,
	pass: String,
}

#[derive(Deserialize, Debug)]
struct FormRegister {
	name: String,
	email: String,
	pass: String,
}

#[derive(Debug, Clone)]
struct AccountQuery {
	id: i64,
	name: String,
	email: String,
	pass_hash: String,
	is_worker: bool,
}

// TODO could use uuid_v3 with week + email + year, to keep UUIDs
type UID = i64;

#[derive(Debug, Clone)]
struct Account {
	uid: UID,
	name: String,
	email: String,
	pass_hash: String,
	uuid: Uuid,
	is_worker: bool,
}

#[derive(Debug, Clone, Copy)]
struct BookStatus {
	uid: UID,
	time: chrono::NaiveDate,
	status: BorrowStatus,
}

type BID = i64;

#[allow(non_snake_case)]
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
enum BorrowStatus { Reserved, Borrowed }
impl BorrowStatus {
	fn from(is_borrow: bool) -> Self {
		if is_borrow {
			BorrowStatus::Borrowed
		} else {
			BorrowStatus::Reserved
		}
	}
}

// djb2
fn hash(st: &[u8]) -> i64 {
	let mut hash: i64 = 5381;
	for chr in st {
		hash = 33*hash+(*chr as i64);
	}
	hash
}

fn hash_pass(pass: &[u8]) -> String {
	let pass_hash = hash(pass).to_le_bytes();
	Uuid::new_v3(&Uuid::NAMESPACE_OID, &pass_hash).to_string()
}

#[derive(Debug, Deserialize)]
struct Goto {
	goto: Option<String>,
}

