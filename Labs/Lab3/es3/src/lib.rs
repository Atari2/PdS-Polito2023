use clap::Parser;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;

#[derive(Parser)]
pub struct Args {
    pub cal1: String,
    pub cal2: String,
    pub duration: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hour {
    hour: u8,
    minute: u8,
}

impl Display for Hour {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

#[derive(Debug)]
pub enum InvalidCalendar {
    Line,
    Hour(ParseIntError),
    File(std::io::Error),
}

impl From<ParseIntError> for InvalidCalendar {
    fn from(e: ParseIntError) -> Self {
        InvalidCalendar::Hour(e)
    }
}

impl From<std::io::Error> for InvalidCalendar {
    fn from(e: std::io::Error) -> Self {
        InvalidCalendar::File(e)
    }
}

impl Hour {
    pub fn new(hour: u8, minute: u8) -> Self {
        Hour { hour, minute }
    }
    pub fn from_string(s: String) -> Result<Hour, InvalidCalendar> {
        let fragments = s.split(':').map(|s| s.trim()).collect::<Vec<_>>();
        if fragments.len() != 2 {
            return Err(InvalidCalendar::Line);
        }
        let hour = fragments[0].parse::<u8>()?;
        let minute = fragments[1].parse::<u8>()?;
        Ok(Hour { hour, minute })
    }
}

impl Hour {
    fn mindiff(&self, other: &Hour) -> usize {
        let self_minutes = self.hour as usize * 60 + self.minute as usize;
        let other_minutes = other.hour as usize * 60 + other.minute as usize;
        if self_minutes > other_minutes {
            self_minutes - other_minutes
        } else {
            other_minutes - self_minutes
        }
    }
}

#[derive(Debug)]
pub struct Calendar {
    schedule: Vec<(Hour, Hour)>,
    bounds: (Hour, Hour),
}

pub struct CalendarIterator<'a> {
    calendar: &'a Calendar,
    current_bound: Hour,
    schedule_iter: std::slice::Iter<'a, (Hour, Hour)>,
    duration: usize
}

impl<'b> Calendar {
    
    fn from_reader<T: std::io::Read>(reader: &mut BufReader<T>) -> Result<Self, InvalidCalendar> {
        let mut schedule = Vec::new();
        let mut bounds = (Hour { hour: 0, minute: 0 }, Hour { hour: 0, minute: 0 });
        // first 2 lines are the bounds
        let mut buf = String::new();
        reader.read_line(&mut buf)?;
        bounds.0 = Hour::from_string(buf)?;
        buf = String::new();
        reader.read_line(&mut buf)?;
        bounds.1 = Hour::from_string(buf)?;
        let mut start_shift = String::new();
        let mut end_shift = String::new();
        while let (Ok(_), Ok(_)) = (
            reader.read_line(&mut start_shift),
            reader.read_line(&mut end_shift),
        ) {
            if start_shift.trim().is_empty() || end_shift.trim().is_empty() {
                break;
            }
            let start = Hour::from_string(start_shift)?;
            let end = Hour::from_string(end_shift)?;
            schedule.push((start, end));
            start_shift = String::new();
            end_shift = String::new();
        }
        Ok(Calendar { schedule, bounds })
    }

    pub fn from_string(calstr: &str) -> Result<Self, InvalidCalendar> {
        let mut reader = BufReader::new(calstr.as_bytes());
        Self::from_reader(&mut reader)
    }
    pub fn from_file(filename: String) -> Result<Self, InvalidCalendar> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);
        Self::from_reader(&mut reader)
    }
    pub fn find_slots<'a>(&'b self, duration: usize) -> CalendarIterator<'a>
    where
        'b: 'a,
    {
        CalendarIterator {
            calendar: self,
            current_bound: self.bounds.0.clone(),
            schedule_iter: self.schedule.iter(),
            duration
        }
    }
}

impl<'a> Iterator for CalendarIterator<'a> {
    type Item = (Hour, Hour);

    fn next(&mut self) -> Option<Self::Item> {
        for (start, end) in self.schedule_iter.by_ref() {
            // check if we have time between current_bound and start
            if self.current_bound.mindiff(start) >= self.duration {
                let bs = std::mem::replace(&mut self.current_bound, end.clone());
                return Some((bs, start.clone()));
            } else {
                self.current_bound = end.clone();
            }
        }
        // when the iterator is done we need to check the last bound
        if self.current_bound.mindiff(&self.calendar.bounds.1) >= self.duration {
            let bs = std::mem::replace(&mut self.current_bound, self.calendar.bounds.1.clone());
            return Some((bs, self.calendar.bounds.1.clone()));
        }
        None
    }
}