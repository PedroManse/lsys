// library system

mod time;
mod sql;
//mod client {}
//mod worker {}

fn ex<A, E: ToString>(a: Result<A, E>) -> Result<A, String> {
	a.map_err(|e|e.to_string())
}

use std::str::FromStr;
fn main() -> Result<(), String> {
	let conn = ex(sql::open("db"))?;
	ex(conn.schema(sql::SQL_TABLE_SCHEMA))?;

	//let pub_date = ex(time::Date::from_str("1-12-1887"))?;
	//let b = ex(Book::new(&conn, 9780140439083, "A Study In Scarlet".to_string(), vec![], pub_date))?;

	let mut input = String::new();
	println!("action:");
	ex(stda:io::stdin().read_line(&mut input))?;
	println!("`{input}`");

	let mut stmt = ex(conn.prepare("SELECT * FROM books"))?;
	let rows = ex(stmt.query_map([], Book::from_query))?;
	let bks: Vec<Book> = rows.filter(|a|a.is_ok()).map(|b|b.unwrap()).collect();
	println!("{:#?}", bks);

	Ok(())
}

#[derive(Debug)]
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

