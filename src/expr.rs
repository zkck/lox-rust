use std::fmt::Display;

use crate::object;

#[derive(Clone, Copy)]
pub enum LogicalOperator {
    Or,
    And,
}

impl Display for LogicalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogicalOperator::Or => "or",
            LogicalOperator::And => "and",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy)]
pub enum UnaryOperator {
    Neg,
    Bang,
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UnaryOperator::Neg => "-",
            UnaryOperator::Bang => "!",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy)]
pub enum BinaryOperator {
    EqualEqual,
    BangEqual,

    LessThan,
    LessEqualThan,

    GreaterThan,
    GreaterEqualThan,

    Add,
    Sub,
    Mul,
    Div,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BinaryOperator::EqualEqual => "==",
            BinaryOperator::BangEqual => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::LessEqualThan => "<=",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::GreaterEqualThan => ">=",
            BinaryOperator::Add => "+",
            BinaryOperator::Sub => "-",
            BinaryOperator::Mul => "*",
            BinaryOperator::Div => "/",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub enum Expr {
    Literal(object::LoxObject),
    Unary(UnaryOperator, Box<Expr>),
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Logical(Box<Expr>, LogicalOperator, Box<Expr>),
    Call{
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Grouping(Box<Expr>),
    Variable(String),
    Assign(String, Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(obj) => write!(f, "{}", obj),
            Expr::Unary(op, expr) => write!(f, "({} {})", op, expr),
            Expr::Binary(expr1, op, expr2) => write!(f, "({} {} {})", op, expr1, expr2),
            Expr::Grouping(expr) => write!(f, "({})", expr),
            Expr::Variable(name) => write!(f, "${}", name),
            Expr::Assign(name, expr) => write!(f, "(= ${}, {})", name, expr),
            Expr::Logical(expr1, op, expr2) => write!(f, "({} {} {})", op, expr1, expr2),
            Expr::Call { callee: _, arguments: _ } => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_literal() {
        let expr = Expr::Literal(object::LoxObject::Nil);
        assert_eq!(expr.to_string(), "nil")
    }

    #[test]
    fn simple_grouping() {
        let expr = Expr::Grouping(Box::new(Expr::Literal(object::LoxObject::Nil)));
        assert_eq!(expr.to_string(), "(nil)")
    }

    #[test]
    fn simple_binary() {
        let expr = Expr::Binary(
            Box::new(Expr::Literal(object::LoxObject::True)),
            BinaryOperator::Add,
            Box::new(Expr::Literal(object::LoxObject::False)),
        );
        assert_eq!(expr.to_string(), "(+ true false)")
    }
}
