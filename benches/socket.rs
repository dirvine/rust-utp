#![feature(test)]

extern crate test;
extern crate utp;
extern crate rand;

use test::Bencher;
use utp::UtpSocket;
use std::sync::Arc;
use std::thread;
use std::cmp::min;

macro_rules! iotry {
    ($e:expr) => (match $e { Ok(e) => e, Err(e) => panic!("{}", e) })
}

fn next_test_port() -> u16 {
    use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
    static NEXT_OFFSET: AtomicUsize = ATOMIC_USIZE_INIT;
    const BASE_PORT: u16 = 9600;
    BASE_PORT + NEXT_OFFSET.fetch_add(1, Ordering::Relaxed) as u16
}

fn next_test_ip4<'a>() -> (&'a str, u16) {
    ("127.0.0.1", next_test_port())
}

#[bench]
fn bench_connection_setup_and_teardown(b: &mut Bencher) {
    let server_addr = next_test_ip4();
    let mut buf = [0; 1500];

    b.iter(|| {
        let mut server = iotry!(UtpSocket::bind(server_addr));

        thread::spawn(move || {
            let mut client = iotry!(UtpSocket::connect(server_addr));
            iotry!(client.close());
        });

        loop {
            match server.recv_from(&mut buf) {
                Ok((0, _src)) => break,
                Ok(_) => (),
                Err(e) => panic!("{}", e)
            }
        }
        iotry!(server.close());
    });
}

#[bench]
fn bench_transfer_one_packet(b: &mut Bencher) {
    let len = 1024;
    let server_addr = next_test_ip4();
    let mut buf = [0; 1500];
    let data = (0..len).map(|x| x as u8).collect::<Vec<u8>>();
    let data_arc = Arc::new(data);

    b.iter(|| {
        let data = data_arc.clone();
        let mut server = iotry!(UtpSocket::bind(server_addr));

        thread::spawn(move || {
            let mut client = iotry!(UtpSocket::connect(server_addr));
            iotry!(client.send_to(&data[..]));
            iotry!(client.close());
        });

        loop {
            match server.recv_from(&mut buf) {
                Ok((0, _src)) => break,
                Ok(_) => (),
                Err(e) => panic!("{}", e)
            }
        }
        iotry!(server.close());
    });
    b.bytes = len as u64;
}

#[bench]
fn bench_transfer_one_megabyte(b: &mut Bencher) {
    let len = 1024 * 1024;
    let server_addr = next_test_ip4();
    let mut buf = [0; 1500];
    let data = (0..len).map(|x| x as u8).collect::<Vec<u8>>();
    let data_arc = Arc::new(data);

    b.iter(|| {
        let data = data_arc.clone();
        let mut server = iotry!(UtpSocket::bind(server_addr));

        thread::spawn(move || {
            let mut client = iotry!(UtpSocket::connect(server_addr));
            iotry!(client.send_to(&data[..]));
            iotry!(client.close());
        });

        loop {
            match server.recv_from(&mut buf) {
                Ok((0, _src)) => break,
                Ok(_) => (),
                Err(e) => panic!("{}", e)
            }
        }
        iotry!(server.close());
    });
    b.bytes = len as u64;
}

#[bench]
fn bench_transfer_one_megabyte_random_chunks(b: &mut Bencher) {
    let len = 1024 * 1024;
    let server_addr = next_test_ip4();
    let data = (0..len).map(|x| x as u8).collect::<Vec<u8>>();
    let data_arc = Arc::new(data);
    let mut received: Vec<u8> = Vec::with_capacity(len);

    b.iter(|| {
        let data = data_arc.clone();
        let mut server = iotry!(UtpSocket::bind(server_addr));

        thread::spawn(move || {
            let mut written_total = 0;
            let mut client = iotry!(UtpSocket::connect(server_addr));
            while written_total < len {
                let chunk_size = rand::random::<u16>() as usize + 1;
                let chunk_size = min(chunk_size, len - written_total);
                //println!("written_total == {}; chunk_size == {}", written_total, chunk_size);
                let written = iotry!(client.send_to(&data[written_total..(written_total + chunk_size)]));
                written_total += written;
            }
            iotry!(client.close());
        });

        let mut read_total: usize = 0;
        let mut chunk = [0u8; 65536];
        while read_total < len {
            let chunk_size = rand::random::<u16>() as usize + 1;
            //println!("read_total == {}; chunk_size == {}", read_total, chunk_size);
            let (read, _) = iotry!(server.recv_from(&mut chunk[..chunk_size]));
            read_total += read;
            received.extend(&chunk[..read]);
        }

        iotry!(server.close());
    });
    b.bytes = len as u64;
}

