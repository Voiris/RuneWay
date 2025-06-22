use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Expr {
    // Types
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,

    // Operations
    BinaryOperation {
        left_operand: Box<Expr>,
        operator: BinaryOperator,
        right_operand: Box<Expr>
    },
    UnaryOperation {
        operator: UnaryOperator,
        operand: Box<Expr>
    },

    Expr(Box<Expr>),
    Variable(String),
    Call {
        act: String,
        arguments: Vec<Expr>,
    },
}

impl Expr {
    pub fn evaluate(&self, env: Rc<RefCell<Environment>>) -> Value {
        match self {
            Expr::Integer(i) => Value::Integer(*i),
            Expr::Float(f) => Value::Float(*f),
            Expr::String(s) => Value::String(s.clone()),
            Expr::Boolean(b) => Value::Boolean(*b),
            Expr::Null => Value::Null,

            Expr::Expr(expr) => expr.evaluate(Rc::clone(&env)),
            Expr::Variable(name) => env
                .borrow()
                .get(name).unwrap_or_else(|| panic!("Переменная '{}' не найдена", name)),

            Expr::UnaryOperation { operator, operand } => {
                let val = operand.evaluate(env);
                match (operator, val) {
                    (UnaryOperator::Neg, Value::Integer(i)) => Value::Integer(-i),
                    (UnaryOperator::Neg, Value::Float(f)) => Value::Float(-f),
                    (UnaryOperator::Not, Value::Boolean(b)) => Value::Boolean(!b),
                    _ => panic!("Неверная унарная операция"),
                }
            }

            Expr::BinaryOperation { left_operand, operator, right_operand } => {
                let left = left_operand.evaluate(Rc::clone(&env));
                let right = right_operand.evaluate(Rc::clone(&env));

                match (left, right, operator) {
                    // i64 arithmetic
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::Add) => Value::Integer(a + b),
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::Sub) => Value::Integer(a - b),
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::Mul) => Value::Integer(a * b),
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::Div) => Value::Integer(a / b),
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::Mod) => Value::Integer(a % b),

                    // i64 equals
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::Eq) => Value::Boolean(a == b),
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::NotEq) => Value::Boolean(a != b),
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::Gt) => Value::Boolean(a > b),
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::GtEq) => Value::Boolean(a >= b),
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::Lt) => Value::Boolean(a < b),
                    (Value::Integer(a), Value::Integer(b), BinaryOperator::LtEq) => Value::Boolean(a <= b),

                    // f64 arithmetic
                    (Value::Float(a), Value::Float(b), BinaryOperator::Add) => Value::Float(a + b),
                    (Value::Float(a), Value::Float(b), BinaryOperator::Sub) => Value::Float(a - b),
                    (Value::Float(a), Value::Float(b), BinaryOperator::Mul) => Value::Float(a * b),
                    (Value::Float(a), Value::Float(b), BinaryOperator::Div) => Value::Float(a / b),

                    // f64 equals
                    (Value::Float(a), Value::Float(b), BinaryOperator::Eq) => Value::Boolean(a == b),
                    (Value::Float(a), Value::Float(b), BinaryOperator::NotEq) => Value::Boolean(a != b),
                    (Value::Float(a), Value::Float(b), BinaryOperator::Gt) => Value::Boolean(a > b),
                    (Value::Float(a), Value::Float(b), BinaryOperator::GtEq) => Value::Boolean(a >= b),
                    (Value::Float(a), Value::Float(b), BinaryOperator::Lt) => Value::Boolean(a < b),
                    (Value::Float(a), Value::Float(b), BinaryOperator::LtEq) => Value::Boolean(a <= b),

                    // bool equals
                    (Value::Boolean(a), Value::Boolean(b), BinaryOperator::And) => Value::Boolean(a && b),
                    (Value::Boolean(a), Value::Boolean(b), BinaryOperator::Or) => Value::Boolean(a || b),

                    // String concatenation
                    (Value::String(a), Value::String(b), BinaryOperator::Add) => Value::String(a + &b),

                    _ => panic!("Недопустимая бинарная операция"),
                }
            }

            Expr::Call { act, arguments } => {
                let args: Vec<Value> = arguments.iter().map(|p| p.evaluate(Rc::clone(&env))).collect();

                match act.as_str() {
                    "print" => {
                        for arg in args {
                            print!("{} ", arg);
                        }
                        print!("\n");
                        Value::Unit
                    }
                    "str" => {
                        Value::String(args.first().unwrap().to_string())
                    }

                    _ => {
                        if env.borrow().contains(act) {
                            let parent_env = Rc::clone(&env);
                            let local_env = Rc::new(RefCell::new(Environment::new_enclosed(parent_env)));
                            let action = env.borrow().get(&act).unwrap();

                            match action {
                                Value::Action { parameters, body } => {
                                    for (param, arg) in parameters.iter().zip(args.iter()) {
                                        local_env.borrow_mut().set(param.clone(), arg.clone());
                                    }

                                    let mut result: Value = Value::Null;

                                    for stmt in body {
                                        match *stmt {
                                            Statement::Return(expr) => {
                                                result = expr.evaluate(Rc::clone(&local_env));
                                            }
                                            _ => {
                                                stmt.execute(Rc::clone(&local_env));
                                            }
                                        }
                                    }

                                    result
                                }
                                _ => panic!("Нельзя вызвать выражение {}", act)
                            }
                        } else {
                            panic!("Неизвестная функция: {}", act)
                        }
                    },
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Action {
        parameters: Vec<String>,
        body: Vec<Box<Statement>>,
    },

    Unit
}

impl Value {
    fn to_string(&self) -> String {
        match self {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Action { parameters: _, body: _ } => format!("{:?}", self),

            Value::Unit => "<Unit>".to_string(),
        }
    }

    fn is(&self, other: &Value) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    // Arithmetic
    Add, // +
    Sub, // -
    Mul, // *
    Pow, // **
    Div, // /
    Mod, // %
    // FloorDiv, // ~/

    // Equalising
    Eq,     // ==
    NotEq,  // !=
    Lt,     // <
    LtEq,   // <=
    Gt,     // >
    GtEq,   // >=

    // Logic
    And,
    Or,
}

impl BinaryOperator {
    pub fn get_precedence(&self) -> u8 {
        match self {
            BinaryOperator::Pow => 5,
            BinaryOperator::Mod => 4,
            BinaryOperator::Mul | BinaryOperator::Div => 3,
            BinaryOperator::Add | BinaryOperator::Sub => 2,
            BinaryOperator::Eq | BinaryOperator::NotEq | BinaryOperator::Lt |
            BinaryOperator::LtEq | BinaryOperator::Gt | BinaryOperator::GtEq => 1,
            BinaryOperator::And | BinaryOperator::Or => 0,
            _ => 255,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,  // -a
    Not,  // !a (not a)
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    Expr(Expr),
    Return(Expr),
    If {
        condition: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Statement>,
    },
    Act {
        name: String,
        parameters: Vec<String>,
        body: Vec<Box<Statement>>,
    }
}

impl Statement {
    fn execute(&self, env: Rc<RefCell<Environment>>) {
        match self {
            Statement::Let { name, value } => {
                let val = value.evaluate(Rc::clone(&env));
                env.borrow_mut().set(name.clone(), val);
            }
            Statement::Act { name, parameters, body } => {
                env.borrow_mut().set(name.clone(), Value::Action { parameters: parameters.clone(), body: body.clone() })
            }
            Statement::Assign { name, value } => {
                env.borrow_mut().set(name.clone(), value.evaluate(Rc::clone(&env)));
            }
            Statement::Expr(expr) => {
                expr.evaluate(env);
            }

            _ => panic!("Cannot execute statement: {:?}", self),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub statements: Vec<Statement>,
}

impl Module {
    pub fn run(&self) {
        let env = Rc::new(RefCell::new(Environment::new()));
        for stmt in &self.statements {
            stmt.execute(Rc::clone(&env));
        }
    }
}

#[derive(Debug, Clone)]
struct Environment {
    variables: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_enclosed(parent: Rc<RefCell<Self>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.variables.get(name) {
            Some(value.clone())
        } else if let Some(ref parent) = self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn contains(&self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            true
        } else if let Some(ref parent) = self.parent {
            parent.borrow().contains(name)
        } else {
            false
        }
    }
}
