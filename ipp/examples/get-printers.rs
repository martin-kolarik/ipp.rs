use std::{env, error::Error, process::exit};

use ipp::{prelude::*, proto::operation::cups::CupsGetPrinters};

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} uri", args[0]);
        exit(1);
    }

    let client = IppClientBuilder::new(&args[1]).build();
    let operation = CupsGetPrinters::new();

    let attrs = futures::executor::block_on(client.send(operation))?;

    for group in attrs.groups_of(DelimiterTag::PrinterAttributes) {
        let name = group.attributes()["printer-name"].value();
        let uri = group.attributes()["device-uri"].value();
        let state = group.attributes()["printer-state"]
            .value()
            .as_enum()
            .and_then(|v| PrinterState::from_i32(*v))
            .ok_or(IppError::InvalidAttributeType)?;

        println!("{}: {} {:?}", name, uri, state);
    }

    Ok(())
}
