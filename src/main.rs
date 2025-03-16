use std::{
	fmt::Display,
	fs::{self, DirEntry, FileType},
	io::{self, Error},
	ops::{Index, IndexMut},
	path::PathBuf,
	process::exit,
};

use clap::Parser;
use colored::{ColoredString, Colorize};
use crossterm::{
	cursor::{MoveTo, position},
	execute,
};

#[derive(Parser)]
struct Args {
	/// Do not ignore entries starting with '.'
	#[arg(short, long)]
	all: bool,

	/// Do not list implied '.' and '..'
	#[arg(short = 'A', long)]
	almost_all: bool,
}

trait RemoveLastCharacters {
	fn remove_last_characters(&mut self, number_of_chars_to_remove: u32);
}

impl RemoveLastCharacters for String {
	fn remove_last_characters(&mut self, number_of_chars_to_remove: u32) {
		*self = self.chars().rev().collect();
		for _ in 0..number_of_chars_to_remove {
			self.pop();
		}
		*self = self.chars().rev().collect();
	}
}

struct DirectoryEntry(DirEntry);

impl DirectoryEntry {
	fn path(&self) -> PathBuf {
		self.0.path()
	}

	fn file_type(&self) -> io::Result<FileType> {
		self.0.file_type()
	}
}

impl Display for DirectoryEntry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let res = self
			.path()
			.into_os_string()
			.into_string()
			.unwrap_or_default();
		write!(f, "{res}")
	}
}

#[derive(Clone)]
struct Column {
	items: Vec<ColoredString>,
	item_max_size: usize,
}

impl Column {
	const fn new(items: Vec<ColoredString>, item_max_size: usize) -> Self {
		Self {
			items,
			item_max_size,
		}
	}
}

fn main() {
	let args = Args::parse();

	let (width, height) = term_size::dimensions().map_or_else(
		|| {
			println!("Unable to get term size :(");
			(0, 0)
		},
		|dimension| (dimension.0, dimension.1),
	);

	let paths = match fs::read_dir(".") {
		Ok(paths) => paths,
		Err(e) => {
			println!("Error occured during directory reading : {e}");
			let error_number = e.raw_os_error().map_or(1, |error_number| error_number);
			exit(error_number)
		}
	};

	let mut files = Vec::new();

	if args.all {
		files.push(".".blue());
		files.push("..".blue());
	}

	for path in paths {
		handle_path(path, &args, &mut files);
	}

	let mut len_counter = 0;
	let mut colums = Vec::new();
	let mut column_counter = 0;
	let mut need_to_fill_col = false;
	for file in files {
		if len_counter > width || need_to_fill_col {
			if need_to_fill_col {
				let col: &mut Column = colums.index_mut(column_counter);
				col.items.push(file.clone());
				if file.len() > col.item_max_size {
					col.item_max_size = file.len();
				}
				if col.items.len() >= height - 3 {
					column_counter += 1;
					len_counter += col.item_max_size;
					need_to_fill_col = false;
				}
			}
		} else {
			let col = Column::new(vec![file.clone()], file.len());
			colums.push(col);
			need_to_fill_col = true;
		}
	}

	let mut is_first_col = true;
	let mut offset_row = 0;
	let mut offset_col = 0;
	let mut original_cursor_location = position().unwrap();
	for column in colums.clone() {
		for file in column.items.clone() {
			if !is_first_col {
				let _ = execute!(
					io::stdout(),
					MoveTo(u16::try_from(offset_col).unwrap(), offset_row),
				);
			}
			println!("{file}");
			offset_row += 1;
		}
		if is_first_col {
			original_cursor_location.1 =
				position().unwrap().1 - u16::try_from(column.items.len()).unwrap();
		}
		offset_row = original_cursor_location.1;
		offset_col += column.item_max_size + 1;
		let _ = execute!(
			io::stdout(),
			MoveTo(original_cursor_location.0, original_cursor_location.1),
		);
		is_first_col = false;
	}
	let _ = execute!(
		io::stdout(),
		MoveTo(
			original_cursor_location.0,
			original_cursor_location.1 + u16::try_from(colums.index(0).items.len()).unwrap()
		),
	);
}

fn handle_path(path: Result<DirEntry, Error>, args: &Args, files: &mut Vec<ColoredString>) {
	let dir_entry = match path {
		Ok(dir_entry) => DirectoryEntry(dir_entry),
		Err(e) => {
			println!("Error occured during directory reading : {e}");
			let error_number = e.raw_os_error().map_or(1, |error_number| error_number);
			exit(error_number)
		}
	};

	let file_type = match dir_entry.file_type() {
		Ok(file_type) => file_type,
		Err(e) => {
			println!("Error while trying to get the file type : {e}");
			let error_number = e.raw_os_error().map_or(1, |error_number| error_number);
			exit(error_number)
		}
	};

	let mut string = dir_entry.to_string();
	if string.is_empty() {
		println!("Error when trying to convert the DirectoryEntry into a String");
		exit(1)
	}
	string.remove_last_characters(2);
	let final_string = if file_type.is_dir() {
		string.blue()
	} else {
		string.green()
	};
	if string.starts_with('.') {
		if args.all || args.almost_all {
			files.push(final_string);
		}
	} else {
		files.push(final_string);
	}
}
