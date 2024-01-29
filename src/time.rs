use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum DateError {
	MonthParseError,
	YearParseError,
	DayParseError,
	DayRangeError,
	WrongLenError,
}

impl ToString for DateError {
	fn to_string(&self) -> String {
		match self {
			DateError::MonthParseError=>"Month Error",
			DateError::YearParseError=>"Year Parsing Error",
			DateError::DayParseError=>"Day Parsing Error",
			DateError::DayRangeError=>"Day range error",
			DateError::WrongLenError=>"Wrong Lenght Error",
		}.to_string()
	}
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum Month {
	Jan, Feb, Mar, Apr,
	May, Jun, Jul, Aug,
	Sep, Oct, Nov, Dec,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Date {
	year: i32,
	month: Month,
	day: u8,
}

pub fn is_leap_year(year: &i32) -> bool {
	year%400 == 0 || (year%4 == 0 && year%100!=0)
}

pub fn valid_day(year: &i32, month: &Month, day: &u8) -> bool {
	let is_leap = is_leap_year(year);
	let max_day = match month {
		Month::Jan=>31,
		Month::Feb=>if is_leap {29} else {28},
		Month::Mar=>31,
		Month::Apr=>30,
		Month::May=>31,
		Month::Jun=>30,
		Month::Jul=>31,
		Month::Aug=>31,
		Month::Sep=>30,
		Month::Oct=>31,
		Month::Nov=>30,
		Month::Dec=>31,
	};
	!(day.clone() == 0 || day.clone() > max_day)
}

//pub fn month_from_int(i: i32) -> Option<Month> {
//	match i {
//		1 => Some(Month::Jan),
//		2 => Some(Month::Feb),
//		3 => Some(Month::Mar),
//		4 => Some(Month::Apr),
//		5 => Some(Month::May),
//		6 => Some(Month::Jun),
//		7 => Some(Month::Jul),
//		8 => Some(Month::Aug),
//		9 => Some(Month::Sep),
//		10 => Some(Month::Out),
//		11 => Some(Month::Nov),
//		12 => Some(Month::Dec),
//		_ => None,
//	}
//}

// _should_ provide month_from_name and month_from_int
// instead of needlessly comparing several int strings

impl std::str::FromStr for Month {
	type Err = DateError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s.to_lowercase().as_str() {
			"jan"|"01"|"1" => Month::Jan,
			"feb"|"02"|"2" => Month::Feb,
			"mar"|"03"|"3" => Month::Mar,
			"apr"|"04"|"4" => Month::Apr,
			"may"|"05"|"5" => Month::May,
			"jun"|"06"|"6" => Month::Jun,
			"jul"|"07"|"7" => Month::Jul,
			"aug"|"08"|"8" => Month::Aug,
			"sep"|"09"|"9" => Month::Sep,
			"out"|"10" => Month::Oct,
			"nov"|"11" => Month::Nov,
			"dec"|"12" => Month::Dec,
			_ => {return Err(DateError::MonthParseError);}
		})
	}
}

impl Month {
	pub fn to_str(self) -> String {
		match self {
			Month::Jan=>"Jan",
			Month::Feb=>"Feb",
			Month::Mar=>"Mar",
			Month::Apr=>"Apr",
			Month::May=>"May",
			Month::Jun=>"Jun",
			Month::Jul=>"Jul",
			Month::Aug=>"Aug",
			Month::Sep=>"Sep",
			Month::Oct=>"Oct",
			Month::Nov=>"Nov",
			Month::Dec=>"Dec",
		}.to_string()
	}
	#[allow(dead_code)]
	pub fn to_str_long(self) -> String {
		match self {
			Month::Jan=>"January",
			Month::Feb=>"February",
			Month::Mar=>"March",
			Month::Apr=>"April",
			Month::May=>"May",
			Month::Jun=>"June",
			Month::Jul=>"July",
			Month::Aug=>"August",
			Month::Sep=>"September",
			Month::Oct=>"October",
			Month::Nov=>"November",
			Month::Dec=>"December",
		}.to_string()
	}
}

impl Date {
	pub fn to_str(&self) -> String {
		format!("{}-{}-{}", self.day, self.month.to_str(), self.year)
	}
	pub fn to_str_split(&self, split: &str) -> String {
		format!("{}{split}{}{split}{}", self.day, self.month.to_str(), self.year)
	}
}

impl std::fmt::Display for Date {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}-{}-{}", self.day, self.month.to_str(), self.year)
	}
}

impl std::str::FromStr for Date {
	type Err = DateError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let split: Vec<String> = s.split(['-', '/']).map(|a|a.to_string()).collect();

		let day_str = split.get(0).ok_or(DateError::WrongLenError)?;
		let day = day_str.parse().or(Err(DateError::DayParseError))?;

		let month_str = split.get(1).ok_or(DateError::WrongLenError)?;
		let month = Month::from_str(month_str)?;

		let year_str = split.get(2).ok_or(DateError::WrongLenError)?;
		let year = year_str.parse().or(Err(DateError::YearParseError))?;

		if !valid_day(&year, &month, &day) {
			Err(DateError::DayRangeError)
		} else {
			Ok(Date{ year, month, day })
		}
	}
}

impl rusqlite::ToSql for Date {
	fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
		let date_str = self.to_string();
		Ok(rusqlite::types::ToSqlOutput::from(date_str))
	}
}
