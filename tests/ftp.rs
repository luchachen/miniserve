use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use rstest::rstest;

mod fixtures;

use crate::fixtures::{Error, port, server};

fn read_ftp_response(reader: &mut BufReader<TcpStream>) -> Result<String, Error> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line)
}

#[rstest]
fn serves_same_directory_over_ftp() -> Result<(), Error> {
    let ftp_port = port();
    let server = server(&["--enable-ftp", "--ftp-port", &ftp_port.to_string()]);

    let mut stream = TcpStream::connect(("127.0.0.1", ftp_port))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    let banner = read_ftp_response(&mut reader)?;
    assert!(banner.starts_with("220 "));

    stream.write_all(b"USER anonymous\r\n")?;
    let user_response = read_ftp_response(&mut reader)?;
    assert!(user_response.starts_with("230 ") || user_response.starts_with("331 "));

    if user_response.starts_with("331 ") {
        stream.write_all(b"PASS miniserve@example.invalid\r\n")?;
        let pass_response = read_ftp_response(&mut reader)?;
        assert!(pass_response.starts_with("230 "));
    }

    stream.write_all(b"SIZE test.txt\r\n")?;
    let size_response = read_ftp_response(&mut reader)?;
    assert!(size_response.starts_with("213 "));

    stream.write_all(b"QUIT\r\n")?;
    let quit_response = read_ftp_response(&mut reader)?;
    assert!(quit_response.starts_with("221 "));

    drop(server);

    Ok(())
}