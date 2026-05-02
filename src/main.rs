use lopdf::{Document};
use base64::prelude::*;
use std::fs;
use std::env;

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = env::args().collect();

    // Load existing PDF
    let mut doc = Document::load(args[0])?;
    

    for (key, val) in doc.objects.iter() {
        if key.0 == 7 {
            println!("stream key: {} - {}", key.0, key.1);
            println!("enum_variant: {}", val.enum_variant());
            match val.enum_variant() {
                "Array" => 
                    match val.as_array() {
                        Ok(arr) => 
                            {
                                let stream7_items = &arr[0];
                                match stream7_items.enum_variant() {
                                    "Dictionary" => 
                                        {
                                            let value = stream7_items.as_dict()?.get(b"V")?;
                                            let name = value.as_name()?;
                                            let decoded_code = BASE64_STANDARD.decode(name)?;
                                            fs::write("payload1.js", decoded_code)?;
                                        },
                                    _ => match stream7_items.type_name() {
                                        Ok(r) => println!("type_name: {:?} {}", str::from_utf8(r), stream7_items.enum_variant()),
                                        Err(err) => println!("type_name err: {}", err)
                                    }
                                }
                            },
                        Err(err) => println!("stream 7 is not an array: {}", err)
                    }
                _ => match val.type_name() {
                    Ok(r) => println!("Stream 7 expected Array type by have type: {:?}", str::from_utf8(r)),
                    Err(err) => println!("Unknown error in stream 7: {}", err)
                }
            }
        }
        if key.0 == 9 {
            println!("stream key: {} - {}", key.0, key.1);
            println!("enum_variant: {}", val.enum_variant());
            match val.enum_variant() {
                "Array" => println!("enum_variant: {:?}", val.as_array()),
                _ => match val.type_name() {
                    Ok(r) => println!("type_name: {:?}", str::from_utf8(r)),
                    Err(err) => println!("type_name: {}", err)
                }
            }
        }
        
    }
    Ok(())
}

#[cfg(feature = "async")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For async feature, you need to use tokio runtime
    println!("This example requires the async feature to be disabled");
    Ok(())
}