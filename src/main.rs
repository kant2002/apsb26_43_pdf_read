use lopdf::{Document,Object};
use base64::prelude::*;
use oxc_span::SPAN;
use std::fs;
use std::env;
use std::path::Path;
use itertools::Itertools;
use oxc::allocator::Allocator;
use oxc::span::SourceType;
use oxc_parser::{Parser,ParseOptions};
use oxc::ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_codegen::{Codegen};

struct JsFuckDeobfuscatorTransformer<'a> {
    pub builder: &'a AstBuilder<'a>,
}

impl<'a> VisitMut<'a> for JsFuckDeobfuscatorTransformer<'a> {
    fn visit_expression(&mut self, it: &mut Expression<'a>) {
        walk_mut::walk_expression(self, it);
        match it {
            Expression::UnaryExpression(expr) =>
                match (expr.operator, &expr.argument) {
                    // +{} => "NaN"
                    (UnaryOperator::UnaryPlus, Expression::ObjectExpression(obje)) if obje.properties.len() == 0 =>
                        {
                            let new_expr = self.builder.expression_string_literal(SPAN, "NaN", Some(Str::new_const("'NaN'")));
                            *it = new_expr;
                        },
                    // +[] => 0
                    (UnaryOperator::UnaryPlus, Expression::ArrayExpression(obje)) if obje.elements.len() == 0 =>
                        {
                            let new_expr = self.builder.expression_numeric_literal(SPAN, 0.0, Some(Str::new_const("0")), NumberBase::Decimal);
                            *it = new_expr;
                        },
                    // +x => x
                    (UnaryOperator::UnaryPlus, Expression::NumericLiteral(lit)) =>
                        {
                            *it = 
                                self.builder.expression_numeric_literal(
                                    SPAN,
                                    lit.value,
                                    lit.raw,
                                    lit.base);
                        },
                    // !0 => 1
                    (UnaryOperator::LogicalNot, Expression::NumericLiteral(lit)) if lit.value == 0.0 =>
                        {
                            let new_expr = 
                                self.builder.expression_numeric_literal(
                                    SPAN,
                                    1.0,
                                    Some(Str::new_const("1")),
                                    NumberBase::Decimal);
                            *it = new_expr;
                        },
                    // !1 => 0
                    (UnaryOperator::LogicalNot, Expression::NumericLiteral(lit)) if lit.value == 1.0 =>
                        {
                            let new_expr = 
                                self.builder.expression_numeric_literal(
                                    SPAN,
                                    0.0,
                                    Some(Str::new_const("0")),
                                    NumberBase::Decimal);
                            *it = new_expr;
                        },
                    _ => {},
                },
            Expression::BinaryExpression(expr) =>
                match (expr.operator, &expr.left, &expr.right) {
                    // {}+[] => "[object Object]"
                    (BinaryOperator::Addition, Expression::ObjectExpression(objl), Expression::ArrayExpression(arrr)) if objl.properties.len() == 0 && arrr.elements.len() == 0 =>
                        {
                            *it = 
                                self.builder.expression_string_literal(
                                    SPAN, 
                                    "[object Object]",
                                    Some(Str::new_const("'[object Object]'")));
                        },
                    // [a]+[b] => "ab"
                    (BinaryOperator::Addition, Expression::ArrayExpression(arrl), Expression::ArrayExpression(arrr)) 
                        if arrl.elements.len() == 1 && arrr.elements.len() == 1 => {
                            match(&arrl.elements[0], &arrr.elements[0]) {
                                (ArrayExpressionElement::NumericLiteral(l), ArrayExpressionElement::NumericLiteral(r)) => {
                                    let value = Str::from_strs_array_in([l.raw.unwrap().as_str(), r.raw.unwrap().as_str()], self.builder.allocator);
                                    let raw_value =
                                        Str::from_strs_array_in(["'", l.raw.unwrap().as_str(), r.raw.unwrap().as_str(), "'"], self.builder.allocator);
                                    *it = 
                                        self.builder.expression_string_literal(
                                            SPAN, 
                                            value,
                                            Some(raw_value));
                                },
                                _ => {},
                            }
                        },
                    (BinaryOperator::Addition, Expression::NumericLiteral(nl), Expression::NumericLiteral(nr)) => {
                            let value = nl.value + nr.value;
                            let raw_value = format!("{}", value);
                            let new_expr = 
                                self.builder.expression_numeric_literal(
                                    SPAN,
                                    value,
                                    Some(Str::from_strs_array_in(
                                        [raw_value.as_str()], self.builder.allocator)),
                                    NumberBase::Decimal);
                            *it = new_expr;
                        },
                    // a + [] = "a"
                    (BinaryOperator::Addition, Expression::NumericLiteral(nl), Expression::ArrayExpression(arrr)) 
                        if arrr.elements.len() == 0 => {
                            let value = nl.value;
                            let raw_value = format!("{}", value);
                            let new_expr = 
                                self.builder.expression_string_literal(
                                    SPAN,
                                    Str::from_strs_array_in(
                                        [raw_value.as_str()], self.builder.allocator),
                                    Some(Str::from_strs_array_in(
                                        ["'", raw_value.as_str(), "'"], self.builder.allocator)),);
                            *it = new_expr;
                        },
                    // "a" + [] = "a"
                    (BinaryOperator::Addition, Expression::StringLiteral(sl), Expression::ArrayExpression(arrr)) 
                        if arrr.elements.len() == 0 => {
                            let new_expr = 
                                self.builder.expression_string_literal(
                                    SPAN,
                                    sl.value,
                                    sl.raw);
                            *it = new_expr;
                        },
                    // "a"+"b" => "ab"
                    (BinaryOperator::Addition, Expression::StringLiteral(sl), Expression::StringLiteral(sr)) => {
                            let value = Str::from_strs_array_in([sl.value.as_str(), sr.value.as_str()], self.builder.allocator);
                            let raw_value =
                                Str::from_strs_array_in(["'", sl.value.as_str(), sr.value.as_str(), "'"], self.builder.allocator);
                            *it = 
                                self.builder.expression_string_literal(
                                    SPAN, 
                                    value,
                                    Some(raw_value));
                        },
                    (BinaryOperator::Division, Expression::NumericLiteral(nl), Expression::NumericLiteral(nr)) 
                        if nr.value == 0.0 && nl.value > 0.0 => {
                            let new_expr = 
                                self.builder.expression_string_literal(
                                    SPAN,
                                    "Infinity",
                                    Some(Str::new_const("'Infinity'")));
                            *it = new_expr;
                        },
                    _ => {},
                },
            _ => {},
        }
        
    }
}


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
    let mut deobfuscator = JsFuckDeobfuscatorTransformer {
        builder: &AstBuilder::new(&allocator),
    };
    deobfuscator.visit_program(&mut program);
    //deobfuscator.visit_program(&mut program);
    // deobfuscator.visit_program(&mut program);
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
