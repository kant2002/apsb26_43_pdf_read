use lopdf::{Document};
use base64::prelude::*;
use std::fs;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];
    println!("Extracting payload from {}", file_name);
    // Load existing PDF
    let doc = Document::load(file_name)?;

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
                                            let base64_payload = value.as_name()?;
                                            fs::write("payload1.base64", base64_payload)?;
                                            let decoded_code = BASE64_STANDARD.decode(base64_payload)?;
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
                "Dictionary" => {
                    let second_payload_dict =val.as_dict()?;
                    let names = second_payload_dict.iter().nth(0).unwrap().1;
                    let dict_with_code = names.as_array()?.iter().nth_back(0).unwrap().as_dict()?;
                    fs::write("payload2.js", dict_with_code.get(b"JS")?.as_str()?)?;
                }
                _ => match val.type_name() {
                    Ok(r) => println!("type_name: {:?}", str::from_utf8(r)),
                    Err(err) => println!("type_name: {}", err)
                }
            }
        }
        
    }
    Ok(())
}
