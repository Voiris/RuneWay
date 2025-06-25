use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Types
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Box<Expr>>),
    FString(Vec<FStringExpr>),

    Iterator {
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
    },
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
            Expr::FString(fs) => {
                let mut string = String::new();

                for fs_expr in fs {
                    match fs_expr {
                        FStringExpr::Expr(expr) => string.push_str(&expr.evaluate(env.clone()).to_string()),
                        FStringExpr::String(s) => string.push_str(&s),
                    }
                }

                Value::String(string)
            }
            Expr::Iterator { start, end, step } => {
                Value::Iterator {
                    start: Box::new(start.evaluate(Rc::clone(&env))),
                    end: Box::new(end.evaluate(Rc::clone(&env))),
                    step: if let Some(step) = step {
                        Box::new(step.evaluate(Rc::clone(&env)))
                    } else {
                        Box::new(Value::Integer(1))
                    },
                }
            }
            Expr::Null => Value::Null,
            Expr::List(l) => {
                let mut list = Vec::new();

                for e in l {
                    list.push(Box::new(e.evaluate(Rc::clone(&env))));
                }

                Value::List(list)
            }
            Expr::Expr(expr) => expr.evaluate(Rc::clone(&env)),
            Expr::Variable(name) => env
                .borrow()
                .get(name).unwrap_or_else(|| panic!("Переменная '{}' не найдена", name)),

            Expr::UnaryOperation { operator, operand } => {
                let val = operand.evaluate(env);
                match (operator, val) {
                    // Unary negative
                    (UnaryOperator::Neg, Value::Integer(i)) => Value::Integer(-i),
                    (UnaryOperator::Neg, Value::Float(f)) => Value::Float(-f),

                    // Unary not
                    (UnaryOperator::Not, Value::Boolean(b)) => Value::Boolean(!b),
                    (UnaryOperator::Not, Value::Null) => Value::Boolean(true),
                    (UnaryOperator::Not, Value::Integer(i)) => Value::Boolean(!(i != 0)),
                    (UnaryOperator::Not, Value::Float(i)) => Value::Boolean(!(i != 0.0)),

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

                    // String concatenation and equalisation
                    (Value::String(a), Value::String(b), BinaryOperator::Add) => Value::String(a + &b),
                    (Value::String(a), b, BinaryOperator::Add) => Value::String(a + &(b.to_string())),

                    (Value::String(a), Value::String(b), BinaryOperator::Eq) => Value::Boolean(a == b),
                    (Value::String(a), Value::String(b), BinaryOperator::NotEq) => Value::Boolean(a != b),
                    (Value::String(a), Value::String(b), BinaryOperator::Gt) => Value::Boolean(a > b),
                    (Value::String(a), Value::String(b), BinaryOperator::GtEq) => Value::Boolean(a >= b),
                    (Value::String(a), Value::String(b), BinaryOperator::Lt) => Value::Boolean(a < b),
                    (Value::String(a), Value::String(b), BinaryOperator::LtEq) => Value::Boolean(a <= b),

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
                    "bool" => {
                        Value::Boolean(args.first().unwrap().to_bool())
                    }
                    "list" => {
                        Value::List(args.first().unwrap().to_list())
                    }

                    _ => {
                        if env.borrow().contains(act) {
                            let parent_env = Rc::clone(&env);
                            let local_env =
                                Rc::new(RefCell::new(Environment::new_enclosed(parent_env)));
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
                                                match stmt.execute(Rc::clone(&local_env)) {
                                                    ControlFlow::Return(value) => {
                                                        result = value;
                                                    }
                                                    _ => {}
                                                }
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

#[derive(Debug, Clone, PartialEq)]
pub enum FStringExpr {
    String(String),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Box<Value>>),
    Iterator {
        start: Box<Value>,
        end: Box<Value>,
        step: Box<Value>,
    },
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
            Value::List(l) =>
                format!(
                    "[{}]",
                    l.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(", ")
                ),
            Value::Iterator { start, end, step } => {
                format!("Iterator({}..{}::{})", start.to_string(), end.to_string(), step.to_string())
            }
            Value::Null => "null".to_string(),

            Value::Action { parameters: _, body: _ } => format!("{:?}", self),

            Value::Unit => "<Unit>".to_string(),
        }
    }

    fn to_bool(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Null => false,

            Value::Iterator { start: _, end: _, step: _ } => true,
            Value::Action { parameters: _, body: _ } => true,

            Value::Unit => false,
        }
    }

    fn to_list(&self) -> Vec<Box<Value>> {
        match self {
            Value::String(s) => {
                let mut list = Vec::new();

                for c in s.chars() {
                    list.push(Box::new(Value::String(c.to_string())));
                }

                list
            },
            Value::Iterator { start, end, step } => {
                match (*start.clone(), *end.clone(), *step.clone()) {
                    (Value::Integer(start), Value::Integer(end), Value::Integer(step)) => {
                        if step <= 0 {
                            return Vec::new()
                        }

                        (start..end)
                            .step_by(step as usize)
                            .map(|i| Box::new(Value::Integer(i)) )
                            .collect()
                    }
                    _ => panic!("Неправильные переменные для итерации")
                }
            },
            t => panic!("Невозможно преобразовать {:?} в список", t),
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Neg,  // -a
    Not,  // !a (not a)
}

#[derive(Debug, Clone, PartialEq)]
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
        then_branch: Vec<Box<Statement>>,
        else_branch: Option<Vec<Box<Statement>>>,
    },
    While {
        condition: Expr,
        body: Vec<Box<Statement>>,
    },
    For {
        variable: String,
        iterable: Expr,
        body: Vec<Box<Statement>>,
    },
    Break,
    Act {
        name: String,
        parameters: Vec<String>,
        body: Vec<Box<Statement>>,
    }
}

impl Statement {
    //noinspection DuplicatedCode
    fn execute(&self, env: Rc<RefCell<Environment>>) -> ControlFlow {
        match self {
            Statement::Let { name, value } => {
                let val = value.evaluate(Rc::clone(&env));
                env.borrow_mut().set(name.clone(), val);
                ControlFlow::Nothing
            }
            Statement::Act { name, parameters, body } => {
                env.borrow_mut().set(name.clone(), Value::Action { parameters: parameters.clone(), body: body.clone() });
                ControlFlow::Nothing
            }
            Statement::Assign { name, value } => {
                let value = value.evaluate(Rc::clone(&env));
                env.borrow_mut().set(name.clone(), value);
                ControlFlow::Nothing
            }
            Statement::If { condition, then_branch, else_branch } => {
                let mut result = ControlFlow::Nothing;
                if condition.evaluate(Rc::clone(&env)).to_bool() {
                    for stmt in then_branch {
                        if stmt.is(&Statement::Break) {
                            result = ControlFlow::Break;
                            break;
                        }
                        match stmt.execute(Rc::clone(&env)) {
                            ControlFlow::Return(value) => result = ControlFlow::Return(value),
                            _ => {}
                        }
                    }
                } else {
                    if else_branch.is_some() {
                        for stmt in else_branch.clone().unwrap() {
                            if stmt.is(&Statement::Break) {
                                result = ControlFlow::Break;
                                break;
                            }
                            match stmt.execute(Rc::clone(&env)) {
                                ControlFlow::Return(value) => result = ControlFlow::Return(value),
                                _ => {}
                            }
                        }
                    }
                }

                result
            }
            Statement::While { condition, body } => {
                let mut result = ControlFlow::Nothing;
                'outer: while condition.evaluate(Rc::clone(&env)).to_bool()  {
                    for stmt in body.iter() {
                        let cf = stmt.execute(Rc::clone(&env));
                        match cf {
                            ControlFlow::Break => break 'outer,
                            ControlFlow::Return(value) => {
                                result = ControlFlow::Return(value);
                                break 'outer;
                            }
                            _ => {}
                        }
                    }
                }

                result
            }
            Statement::For { variable, iterable, body } => {
                let local_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(&env))));

                let iterable = iterable.evaluate(Rc::clone(&local_env));

                let mut result = ControlFlow::Nothing;

                let list = match iterable {
                    Value::List(list) => {
                        let list = list.clone();
                        list
                    }
                    Value::Iterator { start, end, step } => {
                        match (*start, *end, *step) {
                            (Value::Integer(start), Value::Integer(end), Value::Integer(step)) => {
                                (start..end).step_by(usize::try_from(step).unwrap())
                                    .map(|i| Box::new(Value::Integer(i))).collect::<Vec<_>>()
                            }
                            _ => panic!("Cannot iterate over non-integer or not-float values"),
                        }
                    }
                    _ => panic!("Value {:?} is not iterable", iterable),
                };

                'outer: for value in list.iter() {
                    local_env.borrow_mut().set(variable.clone(), *value.clone());

                    for stmt in body.iter() {
                        let cf = stmt.execute(Rc::clone(&local_env));
                        match cf {
                            ControlFlow::Break => break 'outer,
                            ControlFlow::Return(value) => {
                                result = ControlFlow::Return(value);
                                break 'outer;
                            }
                            _ => {}
                        }
                    }
                }

                result
            }
            Statement::Expr(expr) => {
                expr.evaluate(env);
                ControlFlow::Nothing
            }
            Statement::Return(expr) => {
                ControlFlow::Return(expr.evaluate(Rc::clone(&env)))
            }

            _ => panic!("Cannot execute statement: {:?}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ControlFlow {
    Break,
    Continue,
    Nothing,
    Return(Value),
}

impl ControlFlow {
    fn is(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
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
