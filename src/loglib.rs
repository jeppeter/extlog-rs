

#[cfg(target_os = "windows")]
use crate::loglib_windows::{win_output_debug};
use log::{LevelFilter};
use log::{error, info, trace,warn,debug};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root,RootBuilder,ConfigBuilder};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use std::error::Error;
use std::boxed::Box;
use chrono::{Local,Datelike,Timelike};
//use std::collections::HashMap;
use lazy_static::lazy_static;

use std::sync::{Mutex,Arc};



const DEFAULT_MSG_FMT :&str = "{m}";

lazy_static! {
	static ref LOGGER_LEVEL :Arc<Mutex<i64>> = Arc::new(Mutex::new(0 as i64));
}

fn get_logger_level() -> i64 {
	let scb = LOGGER_LEVEL.lock().unwrap();
	let retv :i64;
	retv = *scb;
	return retv;
}

fn set_logger_level(nv :i64) -> i64 {
	let mut scb = LOGGER_LEVEL.lock().unwrap();
	let retv :i64;

	retv = *scb;
	*scb = nv;
	return retv;
}

fn parse_log_var(s :&str) -> (String,u64,u32) {
	let sarr :Vec<&str> = s.split(",").collect();
	let fname :String;
	let mut fsize :u64 = 0;
	let mut times :u32 = 0;
	if sarr.len() > 1 {
		fname = format!("{}",sarr[0]);
		let bss :String = format!("{}",sarr[1]);
		let bs2 = &bss;
		let bs = bs2.as_bytes();
		let mut number :String = "".to_string();
		let mut unit :String = "".to_string();
		let mut n :usize = bs.len();
		match bs2.find(|c :char| !c.is_digit(10)) {
			Some(vn) => {n = vn},
			None => {},
		}
		let mut idx :usize = 0 ;
		while idx < n {
			number.push(bs[idx] as char);
			idx += 1;
		}

		while idx < bs.len() {
			unit.push(bs[idx] as char);
			idx += 1;
		}


		match number.parse::<u64>() {
			Ok(n) => {fsize = n},
			Err(_e) => {},
		}
		if unit == "b" || unit == "B" {
			fsize = fsize;
		} else if unit == "k" || unit == "K" {
			fsize *= 1024;
		} else if unit == "m" || unit == "M" {
			fsize *= 1024 * 1024;
		} else if unit == "g" || unit == "G" {
			fsize *= 1024 * 1024 * 1024;
		} else if unit == "t" || unit == "T" {
			fsize *= 1024 * 1024 * 1024 * 1024;
		}

		if sarr.len() > 2 {
			let tstr:String = format!("{}",sarr[2]);
			match tstr.parse::<u32>() {
				Ok(n) => {times = n},
				Err(_e) => {},
			}
		}

	} else {
		fname = format!("{}",s);
	}
	return (fname,fsize,times);
}

pub struct ExtLogVar {
	pub lognostderr : bool,
	pub logfiles :Vec<String>,
	pub logapps :Vec<String>,
	pub logverbose :i64,
}

impl Default for ExtLogVar {
	fn default() -> Self {
		Self {
			lognostderr : false,
			logfiles : Vec::new(),
			logapps : Vec::new(),
			logverbose : 0,
		}
	}
}


pub fn ext_init_log(ns :&ExtLogVar) -> Result<(),Box<dyn Error>> {
	let level :LevelFilter;
	let mut rbuiler :RootBuilder;
	let mut cbuild :ConfigBuilder;
	let mut sarr :Vec<String>;
	let retv :i64;
	let nostderr :bool;
	let stderr =ConsoleAppender::builder().encoder(Box::new(PatternEncoder::new(DEFAULT_MSG_FMT))).target(Target::Stderr).build();

	retv = ns.logverbose;

	if retv >= 4 {
		level = log::LevelFilter::Trace;
	} else if retv >= 3 {
		level = log::LevelFilter::Debug;
	} else if retv >= 2 {
		level = log::LevelFilter::Info;
	} else if retv >= 1 {
		level = log::LevelFilter::Warn;
	} else {
		level = log::LevelFilter::Error;
	}

	set_logger_level(retv);

	cbuild = Config::builder();
	rbuiler = Root::builder();
	nostderr = ns.lognostderr;


	if !nostderr {
		cbuild = cbuild.appender(
			Appender::builder()
			.filter(Box::new(ThresholdFilter::new(level)))
			.build("stderr", Box::new(stderr)),
			);
		rbuiler = rbuiler.appender("stderr");
	}


	sarr = ns.logfiles.clone();
	for wf in sarr.iter() {
		let (fname,fsize,times) = parse_log_var(wf);
		if fsize == 0 {
			let logfile = FileAppender::builder().append(false).encoder(Box::new(PatternEncoder::new(DEFAULT_MSG_FMT))).build(&fname)?;
			cbuild = cbuild.appender(Appender::builder().build(&fname, Box::new(logfile)));
			rbuiler = rbuiler.appender(&fname);
		} else {
			let fpattern = format!("{}.{{}}.gz",fname);
			let mut tfiles :u32 = 1;
			if times > 0 {
				tfiles = times;
			}
			let logfile = RollingFileAppender::builder().append(false).encoder(Box::new(PatternEncoder::new(DEFAULT_MSG_FMT))).build(&fname,
				Box::new(CompoundPolicy::new(
					Box::new(SizeTrigger::new(fsize)),
					Box::new(FixedWindowRoller::builder().build(&fpattern,tfiles).unwrap())
					)))?;
			cbuild = cbuild.appender(Appender::builder().build(&fname, Box::new(logfile)));
			rbuiler = rbuiler.appender(&fname);
		}
	}


	sarr = ns.logapps.clone();
	for wf in sarr.iter() {
		let (fname,fsize,times) = parse_log_var(wf);
		if fsize == 0 {
			let logfile = FileAppender::builder().append(true).encoder(Box::new(PatternEncoder::new(DEFAULT_MSG_FMT))).build(wf)?;
			cbuild = cbuild.appender(Appender::builder().build(wf, Box::new(logfile)));
			rbuiler = rbuiler.appender(wf);			
		} else {
			let fpattern = format!("{}.{{}}.gz",fname);
			let mut tfiles :u32 = 1;
			if times > 0 {
				tfiles = times;
			}
			let logfile = RollingFileAppender::builder().append(true).encoder(Box::new(PatternEncoder::new(DEFAULT_MSG_FMT))).build(&fname,
				Box::new(CompoundPolicy::new(
					Box::new(SizeTrigger::new(fsize)),
					Box::new(FixedWindowRoller::builder().build(&fpattern,tfiles).unwrap())
					)))?;
			cbuild = cbuild.appender(Appender::builder().build(&fname, Box::new(logfile)));
			rbuiler = rbuiler.appender(&fname);
		}
	}


	let config = cbuild.build(rbuiler.build(level))?;
	let _ = log4rs::init_config(config)?;

	Ok(())
}



pub fn log_get_timestamp() -> String {
	let now = Local::now();
	return format!("{}/{}/{} {}:{}:{}",now.year(),now.month(),now.day(),now.hour(),now.minute(),now.second());
}

#[cfg(target_os = "windows")]
fn log_output_function_inner(level :i64, outs :&str) {
	if level <= get_logger_level() {
		if level == 0 {
			error!("{}",outs);
		} else if level == 1 {
			warn!("{}",outs);
		} else if level == 2 {
			info!("{}",outs);
		} else if level == 3 {
			debug!("{}",outs);
		} else if level >= 4 {
			trace!("{}",outs);
		}
		win_output_debug(outs);
	}
	return;	
}

#[cfg(not(target_os = "windows"))]
fn log_output_function_inner(level :i64, outs :&str) {
	if level <= get_logger_level() {
		if level == 0 {
			error!("{}",outs);
		} else if level == 1 {
			warn!("{}",outs);
		} else if level == 2 {
			info!("{}",outs);
		} else if level == 3 {
			debug!("{}",outs);
		} else if level >= 4 {
			trace!("{}",outs);
		}
	}
	return;	
}


pub fn log_output_function(level :i64, outs :&str) {
	return log_output_function_inner(level,outs);
}

#[macro_export]
macro_rules! format_str_log {
	($info:tt,$iv:expr,$($arg:tt)+) => {
		let mut c :String= format!("[{}:{}]",file!(),line!());
		c.push_str(&format!("{} ",$info));
		c.push_str(&log_get_timestamp());
		c.push_str(": ");
		c.push_str(&(format!($($arg)+)[..]));
		c.push_str("\n");
		log_output_function($iv, &c);		
	}
}

#[macro_export]
macro_rules! debug_error {
	($($arg:tt)+) => {
		format_str_log!("<ERROR>",0,$($arg)+);
	}
}

#[macro_export]
macro_rules! debug_warn {
	($($arg:tt)+) => {
		format_str_log!("<WARN>",1,$($arg)+);
	}
}


#[macro_export]
macro_rules! debug_info {
	($($arg:tt)+) => {
		format_str_log!("<INFO>",2,$($arg)+);
	}
}

#[macro_export]
macro_rules! debug_debug {
	($($arg:tt)+) => {
		format_str_log!("<DEBUG>",3,$($arg)+);
	}
}


#[macro_export]
macro_rules! debug_trace {
	($($arg:tt)+) => {
		format_str_log!("<TRACE>",4,$($arg)+);
	}
}

#[macro_export]
macro_rules! format_buffer_log {
	($buf:expr,$len:expr,$info:tt,$iv:expr,$($arg:tt)+) => {
		let mut c :String = format!("[{}:{}]",file!(),line!());
		c.push_str(&format!("{} ",$info));
		c.push_str(&log_get_timestamp());
		c.push_str(": ");
		c.push_str(&(format!($($arg)+)[..]));
		let _ptr :*const u8 = $buf as *const u8;
		let  mut _ci :usize;
		let _totallen: usize = $len as usize;
		let mut _lasti :usize = 0;
		let mut _nb :u8;
		c.push_str(&format!(" buffer [{:?}][{}:0x{:x}]",_ptr,_totallen,_totallen));
		_ci = 0;
		while _ci < _totallen {
			if (_ci % 16) == 0 {
				if _ci > 0 {
					c.push_str("    ");
					while _lasti < _ci {
						unsafe{
							_nb = *_ptr.offset(_lasti as isize);	
						}
						
						if _nb >= 0x20 && _nb <= 0x7e {
							c.push(_nb as char);
						} else {
							c.push_str(".");
						}
						_lasti += 1;
					}
				}
				c.push_str(&format!("\n0x{:08x}:", _ci));
			}
			unsafe {_nb = *_ptr.offset(_ci as isize);}			
			c.push_str(&format!(" 0x{:02x}",_nb));
			_ci += 1;
		}

		if _lasti < _ci {
			while (_ci % 16) != 0 {
				c.push_str("     ");
				_ci += 1;
			}

			c.push_str("    ");

			while _lasti < _totallen {
				unsafe {_nb = *_ptr.offset(_lasti as isize);}				
				if _nb >= 0x20 && _nb <= 0x7e {
					c.push(_nb as char);
				} else {
					c.push_str(".");
				}
				_lasti += 1;
			}
		}
		c.push_str("\n");
		log_output_function($iv,&c);
	}
}

#[macro_export]
macro_rules! debug_buffer_error {
	($buf:expr,$len:expr,$($arg:tt)+) => {
		format_buffer_log!($buf,$len,"<ERROR>",0,$($arg)+);
	}
}

#[macro_export]
macro_rules! debug_buffer_warn {
	($buf:expr,$len:expr,$($arg:tt)+) => {
		format_buffer_log!($buf,$len,"<WARN>",1,$($arg)+);
	}
}

#[macro_export]
macro_rules! debug_buffer_info {
	($buf:expr,$len:expr,$($arg:tt)+) => {
		format_buffer_log!($buf,$len,"<INFO>",2,$($arg)+);
	}
}

#[macro_export]
macro_rules! debug_buffer_debug {
	($buf:expr,$len:expr,$($arg:tt)+) => {
		format_buffer_log!($buf,$len,"<DEBUG>",3,$($arg)+);
	}
}

#[macro_export]
macro_rules! debug_buffer_trace {
	($buf:expr,$len:expr,$($arg:tt)+) => {
		format_buffer_log!($buf,$len,"<TRACE>",4,$($arg)+);
	}
}
