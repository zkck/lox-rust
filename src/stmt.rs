use crate::expr;

pub enum Stmt {
    Expression(expr::Expr),
    Print(expr::Expr),
    Block(Vec<Stmt>),
    Var {
        name: String,
        initializer: Option<expr::Expr>,
    },
    If {
        condition: expr::Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    }
}
