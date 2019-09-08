extern crate env_logger;
extern crate async_ssh;
extern crate thrussh;
extern crate thrussh_keys;
extern crate futures;
extern crate tokio;
extern crate tokio_core;
use async_ssh::Session;
use futures::Future;
use std::error;
use std::path::PathBuf;
use structopt::StructOpt;
use std::time::Instant;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "russh")]
struct Opt {
    /// Key file
    #[structopt(short, long, parse(from_os_str))]
    key: PathBuf,

    /// Key password
    #[structopt(short, long)]
    password: String,

    /// User
    #[structopt(short, long)]
    user: String,

    /// Command
    #[structopt(short, long)]
    command: String,

    /// Hosts
    #[structopt()]
    hosts: Vec<String>,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opt = Opt::from_args();
    for host in &opt.hosts {
        let key_loading_instant = Instant::now();
        let key = thrussh_keys::load_secret_key(&opt.key, Some(opt.password.as_bytes()))?;
        println!("Key loading time: {:?}", key_loading_instant.elapsed());

        let command_instant = Instant::now();
        let mut core = tokio_core::reactor::Core::new()?;
        let handle = core.handle();

        let ls_out = tokio_core::net::TcpStream::connect(&host.parse()? , &handle)
            .map_err(thrussh::Error::IO)
            .map_err(thrussh::HandlerError::Error)
            .and_then(|c| Session::new(c, &handle))
            .and_then(|session| session.authenticate_key(&opt.user, key))
            .and_then(|mut session| session.open_exec(&opt.command));

        let channel = core.run(ls_out).unwrap();
        let (channel, data) = core.run(tokio_io::io::read_to_end(channel, Vec::new()))?;
        let status = core.run(channel.exit_status()).unwrap();

        println!("Command execution time: {:?}", command_instant.elapsed());
        println!("{}", ::std::str::from_utf8(&data[..])?);
        println!("exited with: {}", status);
    }
    Ok(())
}
