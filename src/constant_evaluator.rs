use oxc_span::SPAN;
use oxc::ast::ast::*;
use oxc_ast::AstBuilder;
use oxc_ast_visit::{VisitMut, walk_mut};

pub struct JsFuckDeobfuscatorTransformer<'a> {
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
                    // ![] => false
                    (UnaryOperator::LogicalNot, Expression::ArrayExpression(arr)) if arr.elements.len() == 0 =>
                        {
                            let new_expr = 
                                self.builder.expression_boolean_literal(
                                    SPAN,
                                    false);
                            *it = new_expr;
                        },
                    // !false => true
                    // !true => false
                    (UnaryOperator::LogicalNot, Expression::BooleanLiteral(bl)) =>
                        {
                            let new_expr = 
                                self.builder.expression_boolean_literal(
                                    SPAN,
                                    !bl.value);
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
                    // true + [] = "true"
                    // false + [] = "false"
                    (BinaryOperator::Addition, Expression::BooleanLiteral(bl), Expression::ArrayExpression(arrr)) 
                        if arrr.elements.len() == 0 => {
                            if bl.value {
                                *it = self.builder.expression_string_literal(
                                    SPAN,
                                    "true",
                                    Some(Str::new_const("'true'")));
                            } else {
                                *it = self.builder.expression_string_literal(
                                    SPAN,
                                    "false",
                                    Some(Str::new_const("'false'")));
                            }
                        },
                    // undefined + [] = "undefined"
                    // TODO: Instead of assuming that undefined is not overiden, in transformation use globalThis.undefined
                    // but even that can be poisoned, if environment is not modern.
                    (BinaryOperator::Addition, Expression::Identifier(ident), Expression::ArrayExpression(arrr)) 
                        if ident.name.len() == 9 && ident.name.as_str().find("undefined") == Some(0) => {
                            *it = self.builder.expression_string_literal(
                                SPAN,
                                "undefined",
                                Some(Str::new_const("'undefined'")));
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
            Expression::ComputedMemberExpression(expr) =>
                match (&expr.object, &expr.expression) {
                    // "abc"[1] => "b"
                    (Expression::StringLiteral(sl), Expression::NumericLiteral(nl)) =>
                        {
                            let index = nl.value as usize;
                            let (_, l) = sl.value.split_at(index);
                            let (value_, _) = l.split_at(1);
                            let value = Str::from_strs_array_in([value_], self.builder.allocator);
                            let raw_value =
                                Str::from_strs_array_in(["'", value_, "'"], self.builder.allocator);
                            *it = 
                                self.builder.expression_string_literal(
                                    SPAN, 
                                    value.as_str(),
                                    Some(raw_value));
                        },
                    // "abc"["1"] => "b"
                    (Expression::StringLiteral(sl), Expression::StringLiteral(nl)) =>
                        {
                            match nl.value.parse::<usize>() {
                                Ok(index) => {
                                    let (_, l) = sl.value.split_at(index);
                                    let (value_, _) = l.split_at(1);
                                    let value = Str::from_strs_array_in([value_], self.builder.allocator);
                                    let raw_value =
                                        Str::from_strs_array_in(["'", value_, "'"], self.builder.allocator);
                                    *it = 
                                        self.builder.expression_string_literal(
                                            SPAN, 
                                            value.as_str(),
                                            Some(raw_value));
                                },
                                _ => {}
                            }
                        },
                    // [][[]] => undefined
                    // TODO: Instead of assuming that undefined is not overiden, in transformation use globalThis.undefined
                    // but even that can be poisoned, if environment is not modern.
                    (Expression::ArrayExpression(arr), Expression::ArrayExpression(arri)) 
                        if arr.elements.len() == 0 =>
                        {
                            *it = self.builder.expression_identifier(SPAN, "undefined");
                        },
                    _ => {},
                },
            _ => {},
        }
        
    }
}
