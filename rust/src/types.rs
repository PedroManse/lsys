use serde::Deserialize;
use chrono::Duration;
use uuid::Uuid;
use chrono::{NaiveDate};
use std::cell::Cell;
use sqlx::{Pool, Sqlite};

pub type Bid = i64;
#[allow(clippy::upper_case_acronyms)]
pub type ISBN = i64;
#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct Book {
	pub ISBN: i64,
	pub bid: Bid,
	pub name: String,
	pub authors: Vec<String>,
	pub published: String,
	pub status: Cell<BorrowStatus>,
}

#[derive(Debug, Clone, Copy)]
pub enum BorrowStatus {
	Avaliable,
	Reserved(Uid, NaiveDate),
	Borrowed(Uid, NaiveDate),
}

#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct BookForm {
	pub ISBN: ISBN,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct NewBookForm {
	pub ISBN: ISBN,
	pub name: String,
	pub published: String,
}

#[derive(Debug, Deserialize)]
pub struct ReserveBookForm {
	pub bid: Bid,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct BookQuery {
	pub ISBN: ISBN,
	pub id: Bid,
	pub name: String,
	pub published: String,
	pub user_id: Option<Uid>,
	pub time: Option<NaiveDate>,
	pub is_borrow: Option<bool>,
}

#[derive(Debug)]
pub enum ReserveBookError {
	Reserved(NaiveDate),
	Borrowed(NaiveDate),
	DBError(String),
}

impl Book {
	//TODO: db update
	pub fn reserve(&self, account: &Account, db: &Pool<Sqlite>) -> Option<ReserveBookError> {
		match self.status.get() {
			BorrowStatus::Reserved(_, until) => {
				Some(ReserveBookError::Reserved(until))
			},
			BorrowStatus::Borrowed(_, until) => {
				Some(ReserveBookError::Borrowed(until))
			},
			BorrowStatus::Avaliable => {
				let lim = chrono::Utc::now().date_naive() + Duration::days(7);
				self.status.set(BorrowStatus::Reserved(account.uid, lim));
				None
			},
		}
	}

	pub fn from_query(info: &BookQuery, authors: Option<&Vec<String>>) -> Self {
		// should not fail, since or is_borrow is NULL and time & user_id are also
		// or is_borrow is Some() and so are time & user_id
		let authors = match authors {
			Some(authors)=>authors.clone(),
			None=>Vec::<String>::new(),
		};
		let status = BorrowStatus::from(info.is_borrow, info.user_id, info.time);
		Book{
			ISBN: info.ISBN,
			bid: info.id,
			name: info.name.to_owned(),
			authors,
			published: info.published.to_owned(),
			status: Cell::new(status),
		}
	}

	//fn from_form(info: &BookForm) {

	//}
}
// TODO could use uuid_v3 with week + email + year, to keep UUIDs
impl std::string::ToString for BorrowStatus {
	fn to_string(&self) -> String {
		match self {
			BorrowStatus::Avaliable => "avaliable",
			BorrowStatus::Reserved(_,_) => "reserved",
			BorrowStatus::Borrowed(_,_) => "borrowed",
		}.to_string()
	}
}

impl BorrowStatus {
	// if .0 -> .1 = Some
	pub fn is_with_viewer(self, viewer: Uid) -> (bool, Option<NaiveDate>) {
		match self {
			BorrowStatus::Avaliable => (false, None),
			BorrowStatus::Reserved(owner,until) => (viewer==owner, Some(until)),
			BorrowStatus::Borrowed(owner,until) => (viewer==owner, Some(until)),
		}
	}
	pub fn is_avaliable(self) -> bool {
		matches!(self, BorrowStatus::Avaliable)
	}
	pub fn is_reserved(self) -> bool {
		matches!(self, BorrowStatus::Reserved(_, _))
	}
	pub fn is_borrowed(self) -> bool {
		matches!(self, BorrowStatus::Borrowed(_, _))
	}
	pub fn from(
		is_borrow: Option<bool>,
		uid: Option<Uid>,
		date: Option<NaiveDate>
	) -> Self {
		//TODO can panic
		match is_borrow {
			Some(is_borrow)=>{
				if is_borrow {
					BorrowStatus::Borrowed(uid.unwrap(), date.unwrap())
				} else {
					BorrowStatus::Reserved(uid.unwrap(), date.unwrap())
				}},
			None=>BorrowStatus::Avaliable
		}
	}
}

#[derive(Debug, Deserialize)]
pub struct BookParam {
	pub bid: i64,
}

pub type Aid = i64;
#[derive(Debug, Clone)]
pub struct Author {
	pub id: i64,
	pub name: String,
}

pub type Uid = i64;

#[derive(Debug, Clone)]
pub struct Account {
	pub uid: Uid,
	pub name: String,
	pub email: String,
	pub pass_hash: String,
	pub uuid: Uuid,
	pub is_worker: bool,
}

#[derive(Debug, Clone)]
pub struct AccountQuery {
	pub id: i64,
	pub name: String,
	pub email: String,
	pub pass_hash: String,
	pub is_worker: bool,
}

#[derive(Deserialize, Debug)]
pub struct FormLogin {
	pub email: String,
	pub pass: String,
}

#[derive(Deserialize, Debug)]
pub struct FormRegister {
	pub name: String,
	pub email: String,
	pub pass: String,
}

