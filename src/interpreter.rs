use crate::environment;
use crate::expr;
use crate::object;
use crate::stmt;

#[derive(Debug)]
pub struct EvaluateError(pub &'static str);

pub trait Interpret<T> {
    fn evaluate(&self, environment: &mut environment::Environment) -> Result<T, EvaluateError>;
}

impl Interpret<object::LoxObject> for expr::Expr {
    fn evaluate(
        &self,
        environment: &mut environment::Environment,
    ) -> Result<object::LoxObject, EvaluateError> {
        match self {
            expr::Expr::Literal(obj) => Ok(obj.clone()),
            expr::Expr::Unary(op, val) => {
                let val = val.evaluate(environment)?;
                match op {
                    expr::UnaryOperator::Neg => {
                        if let object::LoxObject::Number(n) = val {
                            Ok(object::LoxObject::Number(-n))
                        } else {
                            Err(EvaluateError("cannot negate a non-number"))
                        }
                    }
                    expr::UnaryOperator::Bang => Ok(object::LoxObject::from(!is_truthy(&val))),
                }
            }
            expr::Expr::Binary(expr1, op, expr2) => match op {
                expr::BinaryOperator::EqualEqual => Ok(object::LoxObject::from(
                    expr1.evaluate(environment)? == expr2.evaluate(environment)?,
                )),
                expr::BinaryOperator::BangEqual => Ok(object::LoxObject::from(
                    expr1.evaluate(environment)? != expr2.evaluate(environment)?,
                )),
                expr::BinaryOperator::LessThan => {
                    compare_numbers(expr1, expr2, environment, |n1, n2| n1 < n2)
                }
                expr::BinaryOperator::LessEqualThan => {
                    compare_numbers(expr1, expr2, environment, |n1, n2| n1 <= n2)
                }
                expr::BinaryOperator::GreaterThan => {
                    compare_numbers(expr1, expr2, environment, |n1, n2| n1 > n2)
                }
                expr::BinaryOperator::GreaterEqualThan => {
                    compare_numbers(expr1, expr2, environment, |n1, n2| n1 >= n2)
                }
                expr::BinaryOperator::Add => match expr1.evaluate(environment)? {
                    object::LoxObject::Number(n1) => {
                        if let object::LoxObject::Number(n2) = expr2.evaluate(environment)? {
                            Ok(object::LoxObject::from(n1 + n2))
                        } else {
                            Err(EvaluateError(
                                "number value cannot be added with non-number operand",
                            ))
                        }
                    }
                    object::LoxObject::String(s1) => {
                        if let object::LoxObject::String(s2) = expr2.evaluate(environment)? {
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
                expr::BinaryOperator::Sub => match expr1.evaluate(environment)? {
                    object::LoxObject::Number(n1) => {
                        if let object::LoxObject::Number(n2) = expr2.evaluate(environment)? {
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
                expr::BinaryOperator::Mul => match expr1.evaluate(environment)? {
                    object::LoxObject::Number(n1) => {
                        if let object::LoxObject::Number(n2) = expr2.evaluate(environment)? {
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
                expr::BinaryOperator::Div => match expr1.evaluate(environment)? {
                    object::LoxObject::Number(n1) => {
                        if let object::LoxObject::Number(n2) = expr2.evaluate(environment)? {
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
            expr::Expr::Grouping(g) => g.evaluate(environment),
            expr::Expr::Variable(name) => environment
                .get(name)
                .ok_or(EvaluateError("Undefined variable")),
            expr::Expr::Assign(name, expr) => {
                let new_value = expr.evaluate(environment)?;
                if environment.assign(name, new_value.clone()) {
                    Ok(new_value)
                } else {
                    Err(EvaluateError("Undefined variable."))
                }
            }
            expr::Expr::Logical(expr1, op, expr2) => {
                let evaluated = expr1.evaluate(environment)?;
                match op {
                    expr::LogicalOperator::Or => {
                        if is_truthy(&evaluated) {
                            // first statement was true, short-circuit
                            return Ok(evaluated);
                        }
                    }
                    expr::LogicalOperator::And => {
                        if !is_truthy(&evaluated) {
                            // first statement was false, short-circuit
                            return Ok(evaluated);
                        }
                    }
                };
                expr2.evaluate(environment)
            }
        }
    }
}

fn is_truthy(val: &object::LoxObject) -> bool {
    match val {
        object::LoxObject::Number(n) => *n != 0.0,
        object::LoxObject::String(s) => s != "",
        object::LoxObject::True => true,
        object::LoxObject::False => false,
        object::LoxObject::Nil => false,
    }
}

fn compare_numbers<F>(
    expr1: &expr::Expr,
    expr2: &expr::Expr,
    environment: &mut environment::Environment,
    compare_fn: F,
) -> Result<object::LoxObject, EvaluateError>
where
    F: Fn(f32, f32) -> bool,
{
    match (expr1.evaluate(environment)?, expr2.evaluate(environment)?) {
        (object::LoxObject::Number(n1), object::LoxObject::Number(n2)) => {
            Ok(object::LoxObject::from(compare_fn(n1, n2)))
        }
        _ => Err(EvaluateError("comparison can only between two numbers")),
    }
}

impl Interpret<()> for stmt::Stmt {
    fn evaluate(&self, environment: &mut environment::Environment) -> Result<(), EvaluateError> {
        match self {
            stmt::Stmt::Expression(expr1) => {
                expr1.evaluate(environment)?;
            }
            stmt::Stmt::Print(expr1) => {
                println!("{}", expr1.evaluate(environment)?);
            }
            stmt::Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => expr.evaluate(environment)?,
                    None => object::LoxObject::Nil,
                };
                environment.define(name.to_string(), value)
            }
            stmt::Stmt::Block(statements) => {
                environment.new_scope();
                for statement in statements {
                    statement.evaluate(environment)?;
                }
                environment.pop_scope();
            }
            stmt::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&condition.evaluate(environment)?) {
                    then_branch.evaluate(environment)?;
                } else {
                    if let Some(statement) = else_branch {
                        statement.evaluate(environment)?;
                    }
                }
            }
            stmt::Stmt::While(condition, body) => {
                while is_truthy(&condition.evaluate(environment)?) {
                    body.evaluate(environment)?;
                }
            },
        }
        Ok(())
    }
}
