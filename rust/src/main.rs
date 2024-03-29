#![allow(non_snake_case)]
#![allow(dead_code)]
// library system

use axum::{
	Form,
	routing::get,
	extract::State,
	response::Redirect,
	extract::Query,
};
#[allow(unused_imports)]
use axum::debug_handler;
//TODO minijinja
use maud::{html, Markup, DOCTYPE};
use sqlx::sqlite::SqlitePoolOptions;
use serde::{Deserialize};
use chrono::{NaiveDate};
use std::{
	sync::Arc,
	collections::HashMap,
};
use uuid::Uuid;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::services::{ServeDir, ServeFile};
mod types;
use types::*;

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
		.route("/book", get( display_book ))
		.route("/reserve", get(display_reserve_book).post(perform_reserve))
		.route("/test", get(dtest))
		.layer(CookieManagerLayer::new())
		.nest_service("/files",
			ServeDir::new("files")
				.fallback(ServeFile::new("files/404.html"))
		)
		.with_state(new_shared_state(pool).await);

	let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

//TODO: use sub-state to account maps, book maps
// : each sub-state could have it's own Mutex instead of a 'global' mutex
type SharedState = Arc<tokio::sync::Mutex<ServerState>>;
#[derive(Clone, Debug)]
struct ServerState {
	db: sqlx::Pool<sqlx::Sqlite>,
	bid_to_book: HashMap<Bid, Book>,
	uuid_to_account: HashMap<Uuid, Arc<Account>>,
	uid_to_account: HashMap<Uid, Arc<Account>>,
	email_to_uid: HashMap<String, Uid>,
	ISBN_to_authors: HashMap<Bid, Vec<Arc<Author>>>,
	aid_to_authors: HashMap<Aid, Arc<Author>>,
	visits: i64,
}

async fn new_shared_state(db: sqlx::Pool<sqlx::Sqlite>) -> SharedState {
	let accounts = sqlx::query_as!(
		AccountQuery,
		"SELECT id, name, email, pass_hash, is_worker FROM accounts;",
	).fetch_all(&db).await.expect("can't parse row from accounts into AccountQuery");

	let authors = sqlx::query_as!(
		Author,
		"SELECT id, name FROM authors",
	).fetch_all(&db).await.expect("can't parse row from authors into Author");

	let wrotes = sqlx::query!(
		"SELECT author_id, ISBN FROM wrote",
	).fetch_all(&db).await.expect("can't parse row from wrote");

	//TODO: impl for Author
	// : Author::update_maps(Self, &mut state, Vec<ISBN>)
	// : Self.update_map(&mut state, ISBN)
	let mut ISBN_to_authors = HashMap::<ISBN, Vec<Arc<Author>>>::new();
	let mut ISBN_to_anames = HashMap::<ISBN, Vec<String>>::new();
	let mut aid_to_authors = HashMap::<Aid, Arc<Author>>::new();
	for author in authors {
		let author = Arc::new(author);
		aid_to_authors.insert(author.id as Aid, author);
	}

	for wrote in wrotes {
		let author = aid_to_authors
			.get(&wrote.author_id)
			.expect("author_id in Wrote doens't match to an author");
		match ISBN_to_anames.get_mut(&wrote.ISBN) {
			Some(authors)=>{authors.push(author.name.clone())}
			None=>{
				ISBN_to_anames.insert(wrote.ISBN, vec![author.name.clone()]);
			}
		}
		match ISBN_to_authors.get_mut(&wrote.ISBN) {
			Some(authors)=>{authors.push(Arc::clone(author))}
			None=>{
				ISBN_to_authors.insert(wrote.ISBN, vec![Arc::clone(author)]);
			}
		}
	}

	let books = sqlx::query_as!(
		BookQuery,
		"SELECT * FROM books INNER JOIN book_info USING(ISBN);"
	).fetch_all(&db).await.expect("can't parse row from books into BookQuery");

	let mut bid_to_book = HashMap::new();
	for book in books {
		let book = Book::from_query(&book, ISBN_to_anames.get(&book.ISBN));
		bid_to_book.insert(book.bid, book);
	}

	let mut state = ServerState{
		db,
		bid_to_book,
		uuid_to_account: HashMap::new(),
		uid_to_account: HashMap::new(),
		email_to_uid: HashMap::new(),
		aid_to_authors,
		ISBN_to_authors,
		visits: 0,
	};

	accounts.iter()
		.map(Account::from_query)
		.for_each( |acc| Account::update_maps(&mut state, acc) );

	Arc::new( tokio::sync::Mutex::new( state ))
}

async fn read_state(
	stt: SharedState,
) -> ServerState {
	let state = Arc::clone(&stt);
	let state = state.lock().await;
	state.clone()
}

fn read_account(
	state: ServerState,
	cookies: Cookies,
	red: Redirect,
) -> Result<Arc<Account>, Redirect> {
	let acc = cookies.get(COOKIE_UUID_NAME).ok_or(red.clone())?;
	let acc = acc.value().to_owned();
	let acc = Uuid::parse_str(&acc).or(Err(red.clone()))?;
	let acc = state.uuid_to_account.get(&acc).ok_or(red)?;
	Ok(acc.clone())
}

fn make_redirect(
	url: String,
) -> Redirect {
	Redirect::to(url.as_str())
}

#[debug_handler]
async fn display_reserve_book(
	State(stt): State<SharedState>,
	cookies: Cookies,
	Form(reserve): Form<ReserveBookForm>
) -> Result<Markup, Redirect> {
	let state = read_state(stt).await;
	let loginback = make_redirect(format!("/login?goto=/reserve?bid={}", reserve.bid));
	read_account(state.clone(), cookies, loginback)?;

	let book = state.bid_to_book.get(&reserve.bid);
	let book = book.map(|book|{
		let status = &book.status.get();
		if let BorrowStatus::Avaliable = status {
			view_avaliable_book(book)
		} else {
			view_reserved_book(book, status)
		}
	});

	Ok(
		book.unwrap_or(
			view_404(format!("/reserve?bid={}", reserve.bid))
		)
	)
}
async fn dtest(
	State(stt): State<SharedState>,
) -> Markup {
	let state = Arc::clone(&stt);
	let mut state = state.lock().await;
	state.visits+=1;
	html!{
		p{(state.visits)}
	}
}

async fn perform_reserve(
	State(stt): State<SharedState>,
	cookies: Cookies,
	Form(reserve): Form<ReserveBookForm>
) -> Result<Markup, Redirect> {
	let state = Arc::clone(&stt);
	let state = state.lock().await;
	let loginback = make_redirect(format!("/login?goto=/book?bid={}", reserve.bid));
	let home = make_redirect("/".to_owned());
	let acc = read_account(state.clone(), cookies, loginback)?;
	let book = state.bid_to_book.get(&reserve.bid).ok_or(home.clone())?;
	let reserve_error = book.reserve(&acc, &state.db);
	if let Some(error) = reserve_error {
		return Ok( match error {
			ReserveBookError::Reserved(until)=>html!{ p { (until.format("%d/%m/%y")) } },
			ReserveBookError::Borrowed(until)=>html!{ p { (until.format("%d/%m/%y")) } },
			ReserveBookError::DBError(err)=>html!{ p { (err) } },
		} )
	}
	let book = book.clone();
	//TODO RwLock instead of mutex
	// this part drops the immutable state in the mutex then recovers it as mutable this can,
	// however, cause colision when the state is not yet updated, but in this function the book
	drop(state);
	let state = Arc::clone(&stt);
	let mut state = state.lock().await;
	state.bid_to_book.insert(book.bid, book.clone());
	Ok(view_book(book.clone(), &acc))
}
	//state.bid_to_book.insert(book.bid, book.clone());
	//TODO: display_reserve_book
	//if let Some(error) = reserve_error {
	//	Ok(match error {
	//		ReserveBookError::Reserved(until) => {
	//			//TODO tell when book will be free
	//			view_error(
	//				"book already reserved for the next {} days, you can still borrow it at the library".to_owned()
	//			)
	//		},
	//		ReserveBookError::Borrowed(until) => {
	//			//TODO tell when book will be free
	//			view_error(
	//				"book still borrowed for the next {} days".to_owned()
	//			)
	//		},
	//	})
	//} else {

#[debug_handler]
async fn perform_login(
	State(stt): State<SharedState>,
	cookies: Cookies,
	Query(goto): Query<Goto>,
	Form(login): Form<FormLogin>,
) -> Result<Redirect, Markup> {
	let state = Arc::clone(&stt);
	let state = state.lock().await;

	let goto = match &goto.goto {
		Some(goto)=>goto.as_str(),
		None=>"/",
	};

	let acc = state.email_to_uid.get(&login.email).ok_or(view_login("No such email", "", goto))?;
	let acc = state.uid_to_account.get(acc).ok_or(view_login("Can't find account", "", goto))?;
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

async fn display_book(
	State(stt): State<SharedState>,
	cookies: Cookies,
	Query(bid): Query<BookParam>,
) -> Result<Markup, Redirect> {
	let state = read_state(stt).await;
	let loginback = make_redirect(format!("/login?goto=/book?bid={}", bid.bid));
	let acc = read_account(state.clone(), cookies, loginback)?;

	let req_book = state.bid_to_book.get(&bid.bid);

	Ok( match req_book {
		Some(book)=>view_book(book.clone(), &acc),
		None=>view_404(format!("/book?Bid={}", bid.bid)),
	} )
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

	Ok( Redirect::to("/") )
}

async fn display_all(
	State(stt): State<SharedState>,
	cookies: Cookies,
) -> Result<Markup, Redirect> {
	let state = Arc::clone(&stt);
	let state = state.lock().await;

	//let books = &state.books;

	//TODO make into function
	let cookie = cookies.get(COOKIE_UUID_NAME);
	let acc = cookie.ok_or(Redirect::to("/login"))?.value().to_owned();
	let acc = Uuid::parse_str(&acc).or(Err(Redirect::to("/login")))?;
	state.uuid_to_account.get(&acc).ok_or(Redirect::to("/login"))?;

	let mut books = state.bid_to_book
		.clone()
		.into_values()
		.collect::<Vec<Book>>();
	books.sort_by_key(|book|book.name.clone());

	Ok( view_all_books(&books) )
}

// password String -> hash i64 -> [u8] -> v3_uuid String
impl Account {
	fn update_maps(state: &mut ServerState, acc: Self) {
		let acc = Arc::new(acc);
		state.uuid_to_account.insert(acc.uuid, Arc::clone(&acc));
		state.uid_to_account.insert(acc.uid, Arc::clone(&acc));
		state.email_to_uid.insert(acc.email.clone(), acc.uid);
	}

	async fn new(
		db: &sqlx::Pool<sqlx::Sqlite>,
		form: FormRegister,
	) -> Result<Self, String> {
		if form.name.is_empty() || form.email.is_empty() || form.pass.is_empty() {
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
			uid: uid as Uid,
			name: form.name,
			email: form.email,
			pass_hash,
			uuid: Uuid::new_v4(),
			is_worker: false,
		})
	}

	fn from_query(info: &AccountQuery) -> Self {
		Self{
			uid: info.id as Uid,
			name: info.name.clone(),
			email: info.email.clone(),
			pass_hash: info.pass_hash.clone(),
			uuid: Uuid::new_v4(),
			is_worker: info.is_worker,
		}
	}
}

fn view_login(log_error: &str, reg_error: &str, goto: &str) -> Markup {
	html! { (DOCTYPE) head {
		link rel="stylesheet" type="text/css" href="/files/css/login.css"{}
	} body {
		p style="color: red;"{(log_error)}

		p style="color: red;"{(reg_error)}
		fieldset {
			legend {"Login"}
			form method="POST" action={"/login?goto="(goto)} {
				label for="login-email" {"email:"}
				input id="login-email" name="email" type="email" placeholder="email" {}
				br {}
				label for="login-pass" {"password:"}
				input id="login-pass" name="pass" type="password" placeholder="password" {}
				br {}
				button { "LogIn" }
			}
		}

		fieldset {
			legend {"Register"}
			form method="POST" action={"/register?goto="(goto)} {
				label for="register-username" {"username:"}
				input id="register-username" name="name" type="text" placeholder="username" {}
				br {}
				label for="register-email" {"email:"}
				input id="register-email" name="email" type="email" placeholder="email" {}
				br {}
				label for="register-password" {"password:"}
				input id="register-password" name="pass" type="password" placeholder="password" {}
				br {}
				button { "Register" }
			}
		}

	} }
}

fn view_error(error_desc: String) -> Markup {
	html! { (DOCTYPE) body {
		h1 { (error_desc) }
	} }
}

fn view_404(query: String) -> Markup {
	html! { (DOCTYPE) body {
		h1 { {"Can't find query: " (query)} }
	} }
}

fn view_reserved_book(book: &Book, status: &BorrowStatus) -> Markup {
	html!{ (DOCTYPE) head {
		meta charset="UTF-8"{}
		link rel="stylesheet" type="text/css" href="/files/css/book.css"{}
		title { {"LSYS - " (book.name)} }
	} body {
		article {
			aside { img
				src={"/files/img/books/"(book.ISBN)}
				onerror="this.src='/files/img/missing'" {}
			}

			section {
				h1 id="book-name" { i { (book.name) } }
				h5 id="ISBN" { (book.ISBN) }
				h2 { { "Published in: " (book.published) } }
			}

			section {
				p { (format!("{book:#?}")) }
			}

			section {
				h2 {{"Status: " (status.to_string())}}
				@match status {
					BorrowStatus::Reserved(uid, when)=>{
						@let days = days_until(when.to_owned());
						p { {"days:" (days)} }
						//p {"Still possible to read this book inside the library"}
						//p {"Book will be taken in {{.borrower_time_left}} days"}
					},
					BorrowStatus::Borrowed(uid, until)=>{
						@let days = days_until(until.to_owned());
						@if days == 0 {
							p {"Book should be returned today!"}
						} @else if days > 0 {
							p { {"Book may be returned in, at most," (days) " days"} }
						} @ else {
							p { {"Book should have been returned " (days) " days ago"} }
						}
					},
					_=>{ p {"A Grave Server Error Just Happend"} }
				}
			}
		}
	} }
}

fn view_avaliable_book(book: &Book) -> Markup {
	html! { (DOCTYPE) head {
		meta charset="UTF-8"{}
		link rel="stylesheet" type="text/css" href="/files/css/book.css"{}
		title { {"LSYS - " (book.name)} }
	} body {
		article {
			aside { img
				src={"/files/img/books/"(book.ISBN)}
				onerror="this.src='/files/img/missing'" {}
			}

			section {
				h1 id="book-name" { i { (book.name) } }
				h5 id="ISBN" { (book.ISBN) }
				h2 { { "Published in: " (book.published) } }
			}

			section {
				p { (format!("{book:#?}")) }
			}

			section {
				form method="POST" action={"/reserve?bid="(book.bid)}{
					input style="display: none;" name="bid" value=(book.bid){}
					button { "Reserve!" }
				}
			}
		}
	} }
}

fn view_book(book: Book, viewer: &Account) -> Markup {
	html!{ (DOCTYPE) head {
		meta charset="UTF-8"{}
		link rel="stylesheet" type="text/css" href="/files/css/book.css"{}
		title { {"LSYS - " (book.name)} }
	} body {
		article {
			aside { img
				src={"/files/img/books/"(book.ISBN)}
				onerror="this.src='/files/img/missing'" {}
			}

			section {
				h1 id="book-name" { i { (book.name) } }
				h5 id="ISBN" { (book.ISBN) }
				h2 { { "Published in: " (book.published) } }
			}

			section {
				@let status = book.status.get();
				@let (with_viewer, until) = status.is_with_viewer(viewer.uid);
				p {{"is avaliable: "(status.is_avaliable())}}
				@if status.is_avaliable() {
					a href={"/reserve?bid=" (book.bid)} {"Reserve"}
				} @else if with_viewer {
					@let until = until.unwrap();
					@let days = days_until(until);
					//TODO check if will borrow or will return
					@if days == 0 {
						p { "This book is with you, and should be returned today" }
					} @else if days > 0 {
						p { "This book is with you, and should be returned in " (days) " days" }
					} @else {
						p { "This book is with you, and should have been returned " (days) " days ago" }
					}
				} @else {
					// TODO
					p {"LOSER"}
				}
			}
		}
	} }
}

//TODO
//fn view_status(book, uid)

fn view_all_books(books: &Vec<Book>) -> Markup {
	html! { (DOCTYPE) body{
		table {

			thead{ tr {
				td { "ISBN" }
				td { "Name" }
				td { "Authors" }
				td { "Published Date" }
			} }

			tbody{
			@for book in books { tr{
				th {
					(book.ISBN)
				}
				td { a href={"/book?bid="(book.bid)}{ i { (book.name) } } }
				td {
					@for author in &book.authors {
						p {(author)}
					}
				}
				td { (book.published) }
			} }
			}
		}
	} }
}

fn days_until(when: NaiveDate) -> i64 {
	let today = chrono::Utc::now().date_naive();
	(when-today).num_days()
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
