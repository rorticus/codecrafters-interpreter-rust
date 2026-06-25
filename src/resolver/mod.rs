use crate::lexer::Token;
use crate::parser::expr::{Expr, ExprKind};
use crate::parser::stmt::Stmt;
use std::collections::HashMap;
use std::fmt::Display;

pub enum ResolveError {
    SelfReference(usize),
    AlreadyDefined(usize, String),
    InvalidReturn(usize),
}

#[derive(Clone)]
enum FunctionState {
    None,
    Function,
    Method,
}

impl Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::SelfReference(line) => {
                write!(f, "[Line {}] variable references itself", line)
            }
            ResolveError::AlreadyDefined(line, name) => write!(
                f,
                "[Line {}] Error at '{}': Already a variable with this name in this scope.",
                line, name
            ),
            ResolveError::InvalidReturn(line) => {
                write!(f, "[Line {}] Invalid return statement.", line)
            }
        }
    }
}

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionState,
    pub depths: HashMap<usize, usize>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            scopes: Vec::new(),
            current_function: FunctionState::None,
            depths: HashMap::new(),
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, line: usize) -> Result<(), ResolveError> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name) {
                return Err(ResolveError::AlreadyDefined(line, name.to_string()));
            }
            scope.insert(name.to_string(), false);
        }

        Ok(())
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), true);
        }
    }

    fn resolve_local(&mut self, expr_id: usize, name: &str) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.depths.insert(expr_id, i);
                return;
            }
        }
    }

    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), ResolveError> {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();
                for s in stmts {
                    self.resolve_stmt(s)?;
                }
                self.end_scope();
            }
            Stmt::Declaration(name_token, initializer) => {
                self.declare(&name_token.lexeme, name_token.line)?;
                if let Some(init) = initializer {
                    self.resolve_expr(init)?;
                }
                self.define(&name_token.lexeme);
            }
            Stmt::Function { name, params, body } => {
                self.declare(&name.lexeme, name.line)?;
                self.define(&name.lexeme);
                self.resolve_function(params, body, FunctionState::Function)?;
            }
            Stmt::Expression(e) | Stmt::Print(e) => {
                self.resolve_expr(e)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then_branch)?;

                if let Some(else_b) = else_branch {
                    self.resolve_stmt(else_b)?;
                }
            }
            Stmt::While { condition, block } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(block)?;
            }
            Stmt::Return(expr) => {
                if let FunctionState::None = self.current_function {
                    return Err(ResolveError::InvalidReturn(0));
                }

                if let Some(e) = expr {
                    self.resolve_expr(e)?;
                }
            }
            Stmt::For {
                initializer,
                condition,
                increment,
                block,
            } => {
                self.begin_scope();

                if let Some(stmt) = initializer {
                    self.resolve_stmt(stmt)?;
                }

                if let Some(expr) = condition {
                    self.resolve_expr(expr)?;
                }

                if let Some(expr) = increment {
                    self.resolve_expr(expr)?;
                }

                self.resolve_stmt(block)?;

                self.end_scope();
            }
            Stmt::Break(t) | Stmt::Continue(t) => {
                // do nothing
            }
            Stmt::Class { name, methods } => {
                self.declare(&name.lexeme, name.line)?;
                self.define(&name.lexeme);

                for stmt in methods {
                    match stmt {
                        Stmt::Function { params, body, .. } => {
                            self.resolve_function(params, body, FunctionState::Method)?;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn resolve_function(
        &mut self,
        params: &[Token],
        body: &Stmt,
        fn_type: FunctionState,
    ) -> Result<(), ResolveError> {
        let enclosing_function = self.current_function.clone();
        self.current_function = fn_type;

        self.begin_scope();

        for param in params {
            self.declare(&param.lexeme, param.line)?;
            self.define(&param.lexeme);
        }

        if let Stmt::Block(stmts) = body {
            for stmt in stmts {
                self.resolve_stmt(stmt)?;
            }
        }

        self.current_function = enclosing_function;

        self.end_scope();

        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), ResolveError> {
        match &expr.kind {
            ExprKind::Identifier(token) => {
                if let Some(scope) = self.scopes.last() {
                    if scope.get(&token.lexeme) == Some(&false) {
                        return Err(ResolveError::SelfReference(token.line));
                    }
                }

                self.resolve_local(expr.id, &token.lexeme);
            }
            ExprKind::Assign { name, value } => {
                self.resolve_expr(value)?;
                self.resolve_local(expr.id, &name.lexeme);
            }
            ExprKind::Binary { left, right, .. } | ExprKind::Logical { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            ExprKind::Unary { right, .. } => {
                self.resolve_expr(right)?;
            }
            ExprKind::Call {
                expr: callee,
                arguments,
            } => {
                self.resolve_expr(callee)?;
                for arg in arguments {
                    self.resolve_expr(arg)?;
                }
            }
            ExprKind::Grouping(expr) => {
                self.resolve_expr(expr)?;
            }
            ExprKind::Literal(_) => {}
            ExprKind::Get { object, .. } => {
                self.resolve_expr(object)?;
            }
            ExprKind::Set { object, value, .. } => {
                self.resolve_expr(object)?;
                self.resolve_expr(value)?;
            }
        }

        Ok(())
    }
}
