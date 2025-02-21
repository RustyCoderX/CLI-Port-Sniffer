use std::net::TcpStream;
use std::io::{self, Write};
use std::sync::mpsc::{channel, Sender};
use std::{env, net::IpAddr}; // Help to pull our arguments out of the command line
use std::str::FromStr; // Allow converting a string to an IP address
use std::process;
use std::thread;

const MAX: u16 = 65535;

struct Arguments {
    flag: String,
    ipaddr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("Not enough Arguments");
        } else if args.len() > 4 {
            return Err("Too many Arguments");
        }

        let f = args[1].clone();
        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            return Ok(Arguments {
                flag: String::from(""),
                ipaddr,
                threads: 4,
            });
        } else {
            let flags = args[1].clone();
            if (flags.contains("-h") || flags.contains("-help")) && args.len() == 2 {
                println!("Usage: -j to select how many threads you want \r\n -h or -help to show this message");
                return Err("Help message displayed");
            } else if flags.contains("-h") || flags.contains("-help") {
                return Err("Too many Arguments");
            } else if flags.contains("-j") {
                if args.len() < 4 {
                    return Err("Not enough arguments for -j flag");
                }

                let threads = match args[2].parse::<u16>() {
                    Ok(s) => s,
                    Err(_) => return Err("Failed to parse thread count"),
                };

                let ipaddr = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("Not a valid IP Address"),
                };

                return Ok(Arguments {
                    threads,
                    flag: flags,
                    ipaddr,
                });
            } else {
                return Err("Invalid Syntax");
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_thread: u16) {
    let mut port = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap(); // Fixed missing send call to notify open ports
            }
            Err(_) => {}
        }

        if (MAX - port) <= num_thread {
            break;
        }
        port += num_thread;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            println!("{}: Error parsing arguments: {}", program, err);
            process::exit(1);
        }
    });

    let num_thread = arguments.threads;
    let addr = arguments.ipaddr;

    let (tx, rx) = channel();

    for i in 0..num_thread {
        let tx = tx.clone();
        let addr = addr.clone(); // Fix: Ensure `addr` is cloned for use in each thread

        thread::spawn(move || {
            scan(tx, i, addr, num_thread);
        });
    }

    let mut out = vec![];

    drop(tx); // Ensure all threads finish sending before collecting results

    for p in rx {
        out.push(p);
    }

    println!(); // Print newline for formatting

    out.sort();
    for v in out {
        println!("Port {} is open", v);
    }
}
