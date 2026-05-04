use std::collections::HashMap;

use oxc_span::SPAN;
use oxc::{ast::ast::*, syntax::scope::ScopeId};
use oxc_ast::AstBuilder;
use oxc_ast_visit::{Visit, VisitMut, walk::*};

pub struct UniqueIdentifierCollector {
    // Count of times indentifiers was bound. 
    // It is't bound only once, then it's effectively unique
    // within this translation unit.
    pub names: HashMap<String, u32>,
}

impl<'a> Visit<'a> for UniqueIdentifierCollector {
    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        let name = String::from(it.name.as_str());
        let existing = self.names.get_mut(&name);
        if existing.is_some() {
            let p = existing.unwrap();
            *p = *p + 1;
        } else {
            self.names.insert(name, 1);
        }
    }
}

#[derive(Clone)]
pub struct RemapScope {
    // Remapping of the identifiers.
    // Once whole map is build, we can found final target idenfier,
    // which is hidden.
    pub remap: HashMap<String, String>,
}

pub struct RemapVistor<'a> {
    // Rules for remapping
    pub remap_scopes: HashMap<oxc::syntax::scope::ScopeId, RemapScope>,
    pub scope_stack: Vec<oxc::syntax::scope::ScopeId>,
    pub builder: &'a AstBuilder<'a>,
}

impl RemapVistor<'_> {
    fn get_identifier(&mut self, name: &String) -> Option<String> {
        let mut current = name;
        for stack in self.scope_stack.iter().rev() {
            let current_map = self.remap_scopes.get(stack).unwrap();
            let remapping = current_map.remap.get(current);
            if remapping.is_some() {
                current = remapping.unwrap();
            }
        }
        
        if current == name {
            None
         } else {
            Some(current.to_string())
         }
    }
}

impl<'a> VisitMut<'a> for RemapVistor<'a> {
    fn enter_scope(&mut self, _flags: oxc::syntax::scope::ScopeFlags, scope_id: &std::cell::Cell<Option<ScopeId>>) {
        let scope_id = scope_id.get().unwrap();
        self.scope_stack.push(scope_id);
    }
    fn leave_scope(&mut self) {
        self.scope_stack.pop();
    }
    fn visit_identifier_reference(&mut self, it: &mut IdentifierReference<'a>) {
        let current_name = String::from(it.name.as_str());
        let new_name = self.get_identifier(&current_name);
        if new_name.is_some() {
            let name_str: String = new_name.unwrap().clone();
            *it = self.builder.identifier_reference(
                SPAN,
                Str::from_strs_array_in([name_str.as_str()], self.builder.allocator));
        }
    }
}

pub struct RemapCollector {
    // Unique identifiers within translation unit.
    pub names: Vec<String>,
    // Stack of scopes
    pub scope_stack: Vec<oxc::syntax::scope::ScopeId>,
    pub remap_scopes: HashMap<oxc::syntax::scope::ScopeId, RemapScope>,

    pub last_scope: usize,
}

impl<'a> Visit<'a> for RemapCollector {
    fn enter_scope(&mut self, _flags: oxc::syntax::scope::ScopeFlags, scope_id: &std::cell::Cell<Option<ScopeId>>) {
        let new_scope = RemapScope {
            remap: HashMap::new()
        };
        let scope_id = 
            if scope_id.get().is_none() {
                self.last_scope = self.last_scope + 1;
                let id = ScopeId::from_usize(self.last_scope);
                scope_id.set(Some(id));
                id
            } else {
                scope_id.get().unwrap()
            };
        self.remap_scopes.insert(scope_id, new_scope);
        self.scope_stack.push(scope_id);
    }
    fn leave_scope(&mut self) {
        self.scope_stack.pop();
    }
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
       match (&it.id, &it.init) {
        (BindingPattern::BindingIdentifier(target), Some(Expression::Identifier(src))) => {
            let target_name = String::from(target.name.as_str());
            let src_name = String::from(src.name.as_str());
            if self.names.contains(&src_name) {
                let current_stack_id = self.scope_stack.last().unwrap();
                //println!("Register remap - {} as {}", target_name, src_name);
                let current_stack = self.remap_scopes.get_mut(current_stack_id).unwrap();
                current_stack.remap.insert(target_name, src_name);
            }
        },
        _ => {}
       }
       walk_variable_declarator(self, it);
    }
}