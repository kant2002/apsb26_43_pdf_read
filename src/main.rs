use lopdf::{Document,Object};
use base64::prelude::*;
use std::fs;
use std::env;
use std::path::Path;
use itertools::Itertools;
use oxc::allocator::Allocator;
use oxc::span::SourceType;
use oxc_parser::{Parser,ParseOptions};
use oxc_ast::AstBuilder;
use oxc_ast_visit::{VisitMut};
use oxc_codegen::{Codegen};
mod constant_evaluator;

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

pub fn get_parse_options() -> ParseOptions {
    ParseOptions {
        // Do not need to parse regexp
        parse_regular_expression: false,
        // Enable all syntax features
        allow_return_outside_function: true,
        allow_v8_intrinsics: true,
        // `oxc_formatter` expects this to be `false`, otherwise panics
        preserve_parens: false,
    }
}


fn reformat_js(file_name: &'static str, output_filename: &'static str, source_text: String) {
    let path = Path::new(file_name);
    let source_type = SourceType::from_path(path).unwrap();

    let allocator = Allocator::new();

    // Parse the source code
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(get_parse_options())
        .parse();

    // Report any parsing errors
    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
        println!("Parsed with Errors.");
    }

    let mut program = ret.program;
    let mut deobfuscator = constant_evaluator::JsFuckDeobfuscatorTransformer {
        builder: &AstBuilder::new(&allocator),
    };
    deobfuscator.visit_program(&mut program);
    let js = Codegen::new().build(&program);
    fs::write(output_filename, js.code).unwrap();
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
    
    reformat_js("payload1.js", "payload1.formatted.js", source_text);
    reformat_js("payload2.js", "payload2.formatted.js", second_payload_source_code);
    Ok(())
}
