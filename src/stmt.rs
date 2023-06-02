use crate::expr;

pub enum Stmt {
    Expression(expr::Expr),
    Print(expr::Expr),
    Var {
        name: String,
        initializer: Option<expr::Expr>,
    },
}
