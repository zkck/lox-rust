use crate::expr;

#[derive(Debug)]
pub struct EvaluateError(&'static str);

pub trait Interpret {
    fn evaluate(&self) -> Result<expr::LiteralObject, EvaluateError>;
}

impl Interpret for expr::Expr {
    fn evaluate(&self) -> Result<expr::LiteralObject, EvaluateError> {
        match self {
            expr::Expr::Literal(obj) => Ok(obj.clone()),
            expr::Expr::Unary(op, val) => {
                let val = val.evaluate()?;
                match op {
                    expr::UnaryOperator::Neg => {
                        if let expr::LiteralObject::Number(n) = val {
                            Ok(expr::LiteralObject::Number(-n))
                        } else {
                            Err(EvaluateError("cannot negate a non-number"))
                        }
                    }
                    expr::UnaryOperator::Bang => Ok(match val {
                        expr::LiteralObject::Number(n) => {
                            if n == 0.0 {
                                expr::LiteralObject::True
                            } else {
                                expr::LiteralObject::False
                            }
                        }
                        expr::LiteralObject::String(s) => {
                            if s == "" {
                                expr::LiteralObject::True
                            } else {
                                expr::LiteralObject::False
                            }
                        }
                        expr::LiteralObject::True => expr::LiteralObject::False,
                        expr::LiteralObject::False => expr::LiteralObject::True,
                        expr::LiteralObject::Nil => expr::LiteralObject::True,
                    }),
                }
            }
            expr::Expr::Binary(expr1, op, expr2) => match op {
                expr::BinaryOperator::EqualEqual => Ok(expr::LiteralObject::from(
                    expr1.evaluate()? == expr2.evaluate()?,
                )),
                expr::BinaryOperator::BangEqual => Ok(expr::LiteralObject::from(
                    expr1.evaluate()? != expr2.evaluate()?,
                )),
                expr::BinaryOperator::LessThan => compare_numbers(expr1, expr2, |n1, n2| n1 < n2),
                expr::BinaryOperator::LessEqualThan => {
                    compare_numbers(expr1, expr2, |n1, n2| n1 <= n2)
                }
                expr::BinaryOperator::GreaterThan => {
                    compare_numbers(expr1, expr2, |n1, n2| n1 > n2)
                }
                expr::BinaryOperator::GreaterEqualThan => {
                    compare_numbers(expr1, expr2, |n1, n2| n1 >= n2)
                }
                expr::BinaryOperator::Add => match expr1.evaluate()? {
                    expr::LiteralObject::Number(n1) => {
                        if let expr::LiteralObject::Number(n2) = expr2.evaluate()? {
                            Ok(expr::LiteralObject::from(n1 + n2))
                        } else {
                            Err(EvaluateError(
                                "number value cannot be added with non-number operand",
                            ))
                        }
                    }
                    expr::LiteralObject::String(s1) => {
                        if let expr::LiteralObject::String(s2) = expr2.evaluate()? {
                            Ok(expr::LiteralObject::from([s1, s2].concat()))
                        } else {
                            Err(EvaluateError(
                                "string value cannot be added to non-string value",
                            ))
                        }
                    }
                    expr::LiteralObject::True | expr::LiteralObject::False => {
                        Err(EvaluateError("boolean cannot be an operand to addition"))
                    }
                    expr::LiteralObject::Nil => {
                        Err(EvaluateError("nil cannot be an operand to addition"))
                    }
                },
                expr::BinaryOperator::Sub => match expr1.evaluate()? {
                    expr::LiteralObject::Number(n1) => {
                        if let expr::LiteralObject::Number(n2) = expr2.evaluate()? {
                            Ok(expr::LiteralObject::from(n1 - n2))
                        } else {
                            Err(EvaluateError(
                                "number value cannot be added with non-number operand",
                            ))
                        }
                    }
                    expr::LiteralObject::String(_)
                    | expr::LiteralObject::True
                    | expr::LiteralObject::False
                    | expr::LiteralObject::Nil => {
                        Err(EvaluateError("subtraction operand cannot be non-number"))
                    }
                },
                expr::BinaryOperator::Mul => match expr1.evaluate()? {
                    expr::LiteralObject::Number(n1) => {
                        if let expr::LiteralObject::Number(n2) = expr2.evaluate()? {
                            Ok(expr::LiteralObject::from(n1 * n2))
                        } else {
                            Err(EvaluateError(
                                "number value cannot be multiplied with non-number operand",
                            ))
                        }
                    }
                    expr::LiteralObject::String(_)
                    | expr::LiteralObject::True
                    | expr::LiteralObject::False
                    | expr::LiteralObject::Nil => {
                        Err(EvaluateError("multiplication operand cannot be non-number"))
                    }
                },
                expr::BinaryOperator::Div => match expr1.evaluate()? {
                    expr::LiteralObject::Number(n1) => {
                        if let expr::LiteralObject::Number(n2) = expr2.evaluate()? {
                            Ok(expr::LiteralObject::from(n1 / n2))
                        } else {
                            Err(EvaluateError(
                                "number value cannot be divided by non-number operand",
                            ))
                        }
                    }
                    expr::LiteralObject::String(_)
                    | expr::LiteralObject::True
                    | expr::LiteralObject::False
                    | expr::LiteralObject::Nil => {
                        Err(EvaluateError("division operand cannot be non-number"))
                    }
                },
            },
            expr::Expr::Grouping(g) => g.evaluate(),
        }
    }
}

fn compare_numbers<F>(
    expr1: &expr::Expr,
    expr2: &expr::Expr,
    compare_fn: F,
) -> Result<expr::LiteralObject, EvaluateError>
where
    F: Fn(f32, f32) -> bool,
{
    match (expr1.evaluate()?, expr2.evaluate()?) {
        (expr::LiteralObject::Number(n1), expr::LiteralObject::Number(n2)) => {
            Ok(expr::LiteralObject::from(compare_fn(n1, n2)))
        }
        _ => Err(EvaluateError("comparison can only between two numbers")),
    }
}
