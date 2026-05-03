use std::path::Path;

use oxc::allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_codegen::Codegen;
use oxc_parser::{ParseOptions, Parser};
use oxc_ast_visit::{VisitMut};
use oxc_span::SourceType;

mod constant_evaluator;

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


pub fn reformat_js(file_name: &'static str, source_text: String) -> String {
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
    js.code
}