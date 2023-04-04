use std::{
    fmt::Debug,
    fmt::Display,
    fmt::Formatter,
    fs::File,
    io::Read,
    os::raw::{c_char, c_float, c_int, c_long},
};
use clap::Parser;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct ValueStruct {
    value_type: c_int,
    val: c_float,
    timestamp: c_long,
}

impl Display for ValueStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:.6}, timestamp: {}", self.val, self.timestamp))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct MValueStruct {
    value_type: c_int,
    val: [c_float; 10],
    mval: c_long,
}

impl Display for MValueStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut values = String::new();
        for i in 0..10 {
            values.push_str(&format!("{:.6} ", self.val[i]));
        }
        f.write_fmt(format_args!("{}, timestamp: {}", values, self.mval))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct SValueStruct {
    value_type: c_int,
    message: [c_char; 21],
}

impl Display for SValueStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let message = unsafe { std::ffi::CStr::from_ptr(self.message.as_ptr()) };
        match message.to_str() {
            Ok(s) => f.write_fmt(format_args!("{}", s)),
            Err(_) => Err(std::fmt::Error),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
union ValueUnion {
    value: ValueStruct,
    mvalue: MValueStruct,
    svalue: SValueStruct,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct CData {
    value_type: c_int,
    value: ValueUnion,
}

impl CData {
    fn from_file(reader: &mut std::io::BufReader<File>) -> Result<Self, Box<dyn std::error::Error>> {
        const SIZE: usize = std::mem::size_of::<CData>();
        let mut buf: [u8; SIZE] = [0; SIZE];
        reader.read_exact(&mut buf)?;
        Ok(unsafe { std::mem::transmute::<[u8; SIZE], CData>(buf) })
    }
}

impl Display for CData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            match self.value_type {
                1 => f.write_fmt(format_args!("Value: {}", self.value.value)),
                2 => f.write_fmt(format_args!("MValue: {}", self.value.mvalue)),
                3 => f.write_fmt(format_args!("Message: {}", self.value.svalue)),
                _ => f.write_str("Unknown"),
            }
        }
    }
}

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    file: String,
}

fn main() {
    let args = Cli::parse();
    let f = match File::open(args.file) {
        Ok(f) => f,
        Err(e) => panic!("Error opening file: {}", e),
    };
    let mut reader = std::io::BufReader::new(f);
    let mut imported_data = vec![];
    while let Ok(data) = CData::from_file(&mut reader) {
        imported_data.push(data);
    }
    for data in imported_data.iter() {
        println!("{}", data);
    }
}
