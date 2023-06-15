use futures::{future, StreamExt};
extern crate serde;
extern crate serde_json;
use std::{fs::File};
use serde::Deserialize;
use std::io::BufReader;
use std::net::IpAddr;
use std::str::FromStr;
use futures::future::join_all;
use clap::{App,Arg};
#[derive(Debug,Deserialize)]
struct  Targest {
      targets: Vec<String>,
}


#[tokio::main]
async fn main() {
    let args = App::new("grep-lite")
  .version("0.1")
  .about("searches for patterns")
  .arg(Arg::with_name("path")
    .help("ips json file")
    .takes_value(true)
    .required(true))
  .get_matches();
let path = args.value_of("path").unwrap();

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let ips: Targest = serde_json::from_reader(reader).unwrap();
    let pinger = tokio_icmp_echo::Pinger::new().await.unwrap();
    
    let handles = ips.targets.into_iter().map(|ip| {
        let addr = IpAddr::from_str(&ip).unwrap();
        let stream = pinger.chain(addr).stream();
        
        tokio::spawn(async move {
            stream.take(3).for_each(|mb_time| {
                match mb_time {
                    Ok(Some(time)) => println!("ip : {:?} => time={:?}",addr, time),
                    Ok(None) => println!(" ip : {:?}  => timeout",addr),
                    Err(err) => println!("error: {:?}", err)
                }
                
                future::ready(())
            }).await;
        })
    }).collect::<Vec<_>>();
    
    // Wait for all tasks to complete
    join_all(handles).await;
}
