use crate::expr;
use crate::object;
use crate::stmt;

#[derive(Debug)]
pub struct EvaluateError(&'static str);

pub trait Interpret {
    fn evaluate(&self) -> Result<object::LoxObject, EvaluateError>;
}

impl Interpret for expr::Expr {
    fn evaluate(&self) -> Result<object::LoxObject, EvaluateError> {
        match self {
            expr::Expr::Literal(obj) => Ok(obj.clone()),
            expr::Expr::Unary(op, val) => {
                let val = val.evaluate()?;
                match op {
                    expr::UnaryOperator::Neg => {
                        if let object::LoxObject::Number(n) = val {
                            Ok(object::LoxObject::Number(-n))
                        } else {
                            Err(EvaluateError("cannot negate a non-number"))
                        }
                    }
                    expr::UnaryOperator::Bang => Ok(match val {
                        object::LoxObject::Number(n) => {
                            if n == 0.0 {
                                object::LoxObject::True
                            } else {
                                object::LoxObject::False
                            }
                        }
                        object::LoxObject::String(s) => {
                            if s == "" {
                                object::LoxObject::True
                            } else {
                                object::LoxObject::False
                            }
                        }
                        object::LoxObject::True => object::LoxObject::False,
                        object::LoxObject::False => object::LoxObject::True,
                        object::LoxObject::Nil => object::LoxObject::True,
                    }),
                }
            }
            expr::Expr::Binary(expr1, op, expr2) => match op {
                expr::BinaryOperator::EqualEqual => Ok(object::LoxObject::from(
                    expr1.evaluate()? == expr2.evaluate()?,
                )),
                expr::BinaryOperator::BangEqual => Ok(object::LoxObject::from(
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
                    object::LoxObject::Number(n1) => {
                        if let object::LoxObject::Number(n2) = expr2.evaluate()? {
                            Ok(object::LoxObject::from(n1 + n2))
                        } else {
                            Err(EvaluateError(
                                "number value cannot be added with non-number operand",
                            ))
                        }
                    }
                    object::LoxObject::String(s1) => {
                        if let object::LoxObject::String(s2) = expr2.evaluate()? {
                            Ok(object::LoxObject::from([s1, s2].concat()))
                        } else {
                            Err(EvaluateError(
                                "string value cannot be added to non-string value",
                            ))
                        }
                    }
                    object::LoxObject::True | object::LoxObject::False => {
                        Err(EvaluateError("boolean cannot be an operand to addition"))
                    }
                    object::LoxObject::Nil => {
                        Err(EvaluateError("nil cannot be an operand to addition"))
                    }
                },
                expr::BinaryOperator::Sub => match expr1.evaluate()? {
                    object::LoxObject::Number(n1) => {
                        if let object::LoxObject::Number(n2) = expr2.evaluate()? {
                            Ok(object::LoxObject::from(n1 - n2))
                        } else {
                            Err(EvaluateError(
                                "number value cannot be added with non-number operand",
                            ))
                        }
                    }
                    object::LoxObject::String(_)
                    | object::LoxObject::True
                    | object::LoxObject::False
                    | object::LoxObject::Nil => {
                        Err(EvaluateError("subtraction operand cannot be non-number"))
                    }
                },
                expr::BinaryOperator::Mul => match expr1.evaluate()? {
                    object::LoxObject::Number(n1) => {
                        if let object::LoxObject::Number(n2) = expr2.evaluate()? {
                            Ok(object::LoxObject::from(n1 * n2))
                        } else {
                            Err(EvaluateError(
                                "number value cannot be multiplied with non-number operand",
                            ))
                        }
                    }
                    object::LoxObject::String(_)
                    | object::LoxObject::True
                    | object::LoxObject::False
                    | object::LoxObject::Nil => {
                        Err(EvaluateError("multiplication operand cannot be non-number"))
                    }
                },
                expr::BinaryOperator::Div => match expr1.evaluate()? {
                    object::LoxObject::Number(n1) => {
                        if let object::LoxObject::Number(n2) = expr2.evaluate()? {
                            Ok(object::LoxObject::from(n1 / n2))
                        } else {
                            Err(EvaluateError(
                                "number value cannot be divided by non-number operand",
                            ))
                        }
                    }
                    object::LoxObject::String(_)
                    | object::LoxObject::True
                    | object::LoxObject::False
                    | object::LoxObject::Nil => {
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
) -> Result<object::LoxObject, EvaluateError>
where
    F: Fn(f32, f32) -> bool,
{
    match (expr1.evaluate()?, expr2.evaluate()?) {
        (object::LoxObject::Number(n1), object::LoxObject::Number(n2)) => {
            Ok(object::LoxObject::from(compare_fn(n1, n2)))
        }
        _ => Err(EvaluateError("comparison can only between two numbers")),
    }
}


impl Interpret for stmt::Stmt {
    fn evaluate(&self) -> Result<object::LoxObject, EvaluateError> {
        match self {
            stmt::Stmt::Expression(expr1) => {
                expr1.evaluate()?;
                Ok(object::LoxObject::Nil)
            },
            stmt::Stmt::Print(expr1) => {
                println!("{}", expr1.evaluate()?);
                Ok(object::LoxObject::Nil)
            },
        }
    }
}
