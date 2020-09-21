fn main() {
    let arg = std::env::args().nth(1);
    let port = match arg {
        Some(filename) => {
            println!("{}", filename);
            serial::open(&filename)
        }
        None => {
            eprintln!("usage: macos_example <TTY path>");
            std::process::exit(1);
        }
    }
    .unwrap();
    let mut can = slcan::CanSocket::<serial::SystemPort>::open(port);

    loop {
        match can.read() {
            Ok(frame) => println!("{}", frame),
            Err(error) => match error.kind() {
                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => (),
                _ => eprintln!("{:?}", error),
            },
        }
    }
}
