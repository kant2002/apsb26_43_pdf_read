use std::collections::HashMap;
use std::path::Path;

use oxc::allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_codegen::Codegen;
use oxc_parser::{ParseOptions, Parser};
use oxc_ast_visit::{VisitMut,Visit};
use oxc_span::SourceType;

mod constant_evaluator;
mod unique_identifiers;

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
    let mut const_expr = unique_identifiers::UniqueIdentifierCollector {
        names: HashMap::new(),
    };
    const_expr.visit_program(&program);
    //println!("{:?}", const_expr.names);
    let mut unique_names = Vec::new();
    for (k, &v) in const_expr.names.iter() {
        if v == 1 { 
            unique_names.push(k.clone());
        }
    }
    let mut remap_collector = unique_identifiers::RemapCollector {
        remap_scopes: HashMap::new(),
        scope_stack: Vec::new(),
        names: unique_names,
        last_scope: 0,
    };
    remap_collector.visit_program(&program);
    let mut remap = unique_identifiers::RemapVistor {
        builder: &AstBuilder::new(&allocator),
        remap_scopes: remap_collector.remap_scopes,
        scope_stack: Vec::new(),
    };
    remap.visit_program(&mut program);
    //println!("{:?}", remap_collector.names);
    //println!("{:?}", remap_collector.scope_stack);
    // for (&k,v) in remap_collector.remap_scopes.iter() {
    //     println!("{} {:?}", k.index(), v.remap);
    // }
    let js = Codegen::new().build(&program);
    js.code
}