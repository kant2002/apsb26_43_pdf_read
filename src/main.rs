use lopdf::{Document,Object};
use base64::prelude::*;
use std::fs;
use std::env;
use itertools::Itertools;

use apsb26_43_pdf_read::reformat_js;

fn extract_base64_payload(val: &Object) -> Result<&[u8], Box<dyn std::error::Error>> {
    let arr = val.as_array()?;
    let stream7_items = &arr[0];
    let value = stream7_items.as_dict()?.get(b"V")?;
    let base64_payload = value.as_name()?;
    fs::write("payload1.base64", base64_payload)?;
    Ok(base64_payload)
}

fn extract_second_payload(val: &Object) -> Result<&[u8], Box<dyn std::error::Error>> {
    let second_payload_dict =val.as_dict()?;
    let names = second_payload_dict.iter().nth(0).unwrap().1;
    let dict_with_code = names.as_array()?.iter().nth_back(0).unwrap().as_dict()?;
    Ok(dict_with_code.get(b"JS")?.as_str()?)
}

fn extract_payload(doc: Document) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    //let doc = Document::load(file_name)?;
    let binding = doc.objects.clone();
    let stream_data = 
        binding.iter()
        .filter(|kv| kv.0.0 == 7 || kv.0.0 == 9)
        .collect_tuple();

    match stream_data {
        Some((p1, p2)) => {
            let pp1 = Vec::from(extract_base64_payload(p1.1)?);
            let pp2 = Vec::from(extract_second_payload(p2.1)?);
            Ok((pp1, pp2))
        }
        None => {
            Err(Into::into("The streams 7 or 9 missing"))
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];
    println!("Extracting payload from {}", file_name);
    // Load existing PDF
    let doc = Document::load(file_name)?;
    let (base64_payload, second_payload) = extract_payload(doc)?;
    fs::write("payload1.base64", &base64_payload)?;
    let decoded_code = BASE64_STANDARD.decode(&base64_payload)?;
    fs::write("payload1.js", &decoded_code)?;
    fs::write("payload2.js", &second_payload)?;

    let source_text = String::from_utf8(decoded_code)?;
    let second_payload_source_code = String::from_utf8(second_payload)?;

    fs::write("payload1.formatted.js", reformat_js("payload1.js", source_text))?;
    fs::write("payload2.formatted.js", reformat_js("payload2.js", second_payload_source_code))?;
    Ok(())
}
