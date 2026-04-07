# Aether 语言规范 v1.0

## 词法结构

### 标识符

```ebnf
identifier = letter { letter | digit | "_" } ;
letter = "a".."z" | "A".."Z" ;
digit = "0".."9" ;
```

- 必须以字母或下划线开头
- 区分大小写
- 支持 Unicode 字符

### 关键字

```
let, var, fn, return, if, else, match, while, for, in
struct, enum, trait, impl, type, where
async, await, parallel, actor, message
use, pub, mod, super, self, Self
true, false, nil
try, catch, finally, throw, raise
macro, derive, test, bench
```

### 字面量

```ebnf
integer = ["+" | "-"] digit { digit } ;
float = integer "." integer [exponent] ;
exponent = ("e" | "E") ["+" | "-"] digit { digit } ;
string = "\"" { character | escape_sequence } "\"" ;
character = any UTF-8 character ;
escape_sequence = "\" | "n" | "t" | "r" | "0" ;
boolean = "true" | "false" ;
nil = "nil" ;
```

### 运算符

```
算术运算符：+ - * / % **
比较运算符：== != < > <= >=
逻辑运算符：&& || !
位运算符：& | ^ << >>
赋值运算符：= += -= *= /= %= &= |= ^= <<= >>=
其他运算符：?. ?? |> & * -> :: ..
```

## 语法结构

### 程序结构

```ebnf
program = { module_item } ;
module_item = use_declaration 
            | function_definition 
            | struct_definition 
            | enum_definition 
            | trait_definition 
            | impl_block 
            | macro_definition ;
```

### 变量声明

```ebnf
variable_declaration = "let" identifier [":" type] "=" expression ";" ;
mutable_declaration = "var" identifier ":" type "=" expression ";" ;
```

示例：
```aether
let x = 42                    // 类型推断为 Int
let y: Float = 3.14           // 显式类型
var counter: Int = 0          // 可变变量
let (a, b) = (1, 2)           // 解构绑定
```

### 函数定义

```ebnf
function_definition = ["async"] "fn" identifier generics? "(" [parameters] ")" ["->" type] ["where" where_clause] function_body ;
parameters = parameter { "," parameter } ;
parameter = identifier ":" type ;
function_body = block | "=" expression ";" ;
generics = "<" type_parameter { "," type_parameter } ">" ;
type_parameter = identifier [":" type_bound] ;
where_clause = where_bound { "," where_bound } ;
where_bound = type ":" type_bound ;
type_bound = identifier { "+" identifier } ;
```

示例：
```aether
// 普通函数
fn add(a: Int, b: Int) -> Int {
    a + b
}

// 表达式体函数
fn multiply(a: Int, b: Int) -> Int = a * b

// 泛型函数
fn identity<T>(x: T) -> T = x

// 带约束的泛型
fn print_show<T>(x: T) -> String 
where T: Show {
    x.show()
}

// 异步函数
async fn fetch(url: String) -> Result<String, Error> {
    await http.get(url)
}
```

### 控制流

#### If 表达式

```ebnf
if_expression = "if" condition block ["else" (block | if_expression)] ;
condition = expression ;
```

示例：
```aether
let result = if x > 0 {
    "positive"
} else if x < 0 {
    "negative"
} else {
    "zero"
}
```

#### Match 表达式

```ebnf
match_expression = "match" expression "{" { match_arm } "}" ;
match_arm = pattern "=>" (expression | block) "," ;
pattern = literal_pattern | identifier_pattern | wildcard_pattern 
        | range_pattern | struct_pattern | enum_pattern ;
```

示例：
```aether
match value {
    0 => println("zero"),
    1..10 => println("small"),
    n if n < 100 => println("medium"),  // 守卫条件
    Point(x: 0, y: 0) => println("origin"),
    Some(x) => println(f"got {x}"),
    _ => println("other")
}
```

#### 循环

```ebnf
while_loop = "while" condition block ;
for_loop = "for" pattern "in" expression block ;
loop_expression = "loop" block ;
```

示例：
```aether
// While 循环
while count < 10 {
    count += 1
}

// For 循环
for i in 0..10 {
    println(i)
}

// 迭代集合
for item in collection {
    process(item)
}

// 无限循环
let result = loop {
    if condition {
        break value
    }
}
```

### 类型系统

#### 基础类型

```
Int, Int8, Int16, Int32, Int64
UInt, UInt8, UInt16, UInt32, UInt64
Float, Float32, Float64
Bool
String, Char
Unit (())
Never (!)
```

#### 复合类型

```ebnf
type = identifier [generic_args]
     | "(" [type { "," type }] ")"      // 元组
     | "[" type "]"                     // 数组
     | "&" ["lifetime"] type            // 引用
     | "->" type                        // 函数返回
     | identifier generics?             // 泛型实例化
     ;
generic_args = "<" type { "," type } ">" ;
lifetime = "'" identifier ;
```

#### 结构体

```ebnf
struct_definition = "struct" identifier generics? "{" [field_definitions] "}" ;
field_definitions = field_definition { "," field_definition } ;
field_definition = identifier ":" type ;
```

示例：
```aether
struct Point {
    x: Float,
    y: Float
}

struct TupleStruct(Int, String);

struct UnitStruct;
```

#### 枚举

```ebnf
enum_definition = "enum" identifier generics? "{" [variant_definitions] "}" ;
variant_definitions = variant_definition { "," variant_definition } ;
variant_definition = identifier ["(" [type { "," type }] ")"] ["{" field_definitions "}"] ;
```

示例：
```aether
enum Option<T> {
    Some(T),
    None
}

enum Message {
    Quit,
    Move { x: Int, y: Int },
    Write(String),
    ChangeColor(Int, Int, Int)
}
```

#### Trait

```ebnf
trait_definition = "trait" identifier generics? [":" type_bound_list] "{" [trait_items] "}" ;
trait_items = trait_item { trait_item } ;
trait_item = function_signature | const_declaration | type_alias ;
function_signature = "fn" identifier "(" [parameters] ")" ["->" type] ";" ;
```

示例：
```aether
trait Drawable {
    fn draw(self) -> Canvas;
    fn area(self) -> Float;
}

trait Comparable<T> {
    fn compare(self, other: T) -> Int;
}
```

#### Impl 块

```ebnf
impl_block = "impl" generics? type ["for" type] ["where" where_clause] "{" [impl_items] "}" ;
impl_items = impl_item { impl_item } ;
impl_item = function_definition | const_declaration | type_alias ;
```

示例：
```aether
impl Point {
    fn new(x: Float, y: Float) -> Point {
        Point { x, y }
    }
    
    fn distance(&self, other: Point) -> Float {
        ((self.x - other.x) ** 2 + (self.y - other.y) ** 2).sqrt()
    }
}

impl Drawable for Circle {
    fn draw(self) -> Canvas {
        // 实现细节
    }
}
```

### 表达式

```ebnf
expression = literal 
           | identifier 
           | binary_expression 
           | unary_expression 
           | call_expression 
           | index_expression 
           | field_access 
           | lambda_expression 
           | block 
           | if_expression 
           | match_expression 
           | loop_expression
           ;
binary_expression = expression operator expression ;
unary_expression = operator expression ;
call_expression = expression "(" [arguments] ")" ;
arguments = argument { "," argument } ;
argument = [identifier ":"] expression ;
index_expression = expression "[" expression "]" ;
field_access = expression "." identifier ;
lambda_expression = "fn" "(" [parameters] ")" ["->" type] block ;
```

### 模式匹配

```ebnf
pattern = literal 
        | identifier 
        | "_" 
        | range_pattern 
        | struct_pattern 
        | enum_pattern 
        | tuple_pattern 
        | slice_pattern 
        | reference_pattern 
        ;
range_pattern = expression ".." [expression] ;
struct_pattern = identifier "{" [field_pattern { "," field_pattern }] "}" ;
field_pattern = identifier [":" pattern] ;
enum_pattern = identifier ["(" [pattern { "," pattern }] ")"] ;
tuple_pattern = "(" [pattern { "," pattern }] ")" ;
slice_pattern = "[" [pattern { "," pattern }] "]" ;
reference_pattern = "&" pattern ;
```

### 错误处理

```ebnf
try_block = "try" block ["catch" catch_clauses] ["finally" block] ;
catch_clauses = catch_clause { catch_clause } ;
catch_clause = "catch" [pattern] block ;
throw_expression = "throw" expression ;
raise_expression = "raise" expression ;
option_operator = expression "?" ;
```

示例：
```aether
// Try-catch-finally
try {
    risky_operation()
} catch NetworkError(e) {
    handle_network_error(e)
} catch e {
    handle_generic_error(e)
} finally {
    cleanup()
}

// 错误传播
fn read_file(path: String) -> Result<String, Error> {
    let content = fs.read(path)?  // ? 传播错误
    Ok(content)
}
```

### 并发原语

#### Async/Await

```ebnf
async_function = "async" "fn" identifier "(" [parameters] ")" ["->" type] block ;
await_expression = "await" expression ;
```

#### Parallel

```ebnf
parallel_block = "parallel" "[" [expressions] "]" ;
```

#### Actor

```ebnf
actor_definition = "actor" identifier "{" [actor_items] "}" ;
actor_items = actor_item { actor_item } ;
actor_item = variable_declaration | message_handler ;
message_handler = "message" identifier "(" [parameters] ")" block ;
```

示例：
```aether
actor BankAccount {
    var balance: Float = 0.0
    
    message Deposit(amount: Float) {
        balance += amount
    }
    
    message Withdraw(amount: Float, reply_to: Channel<Result<(), Error>>) {
        if amount <= balance {
            balance -= amount
            reply_to.send(Ok(()))
        } else {
            reply_to.send(Err("Insufficient funds"))
        }
    }
}
```

### 宏系统

```ebnf
macro_definition = "macro" identifier "{" [macro_rules] "}" ;
macro_rules = macro_rule { macro_rule } ;
macro_rule = "(" [pattern] ")" "=>" "(" [template] ")" ;
```

示例：
```aether
macro unless {
    ($cond:expr, $body:block) => {
        if !$cond { $body }
    }
}

// 使用
unless(is_valid(), {
    println("Invalid!")
})
```

### 属性

```ebnf
attribute = "#" "[" identifier ["(" [arguments] ")"] "]" ;
```

常用属性：
```aether
@derive(Debug, Clone, Serialize, Deserialize)
@test
@bench
@inline
@no_mangle
@export
@deprecated("Use new_function instead")
```

## 内存模型

### 所有权规则

1. 每个值都有一个所有者
2. 一次只能有一个所有者
3. 当所有者离开作用域，值被丢弃

### 借用规则

1. 任意时刻，要么有一个可变引用，要么有任意数量的不可变引用
2. 引用必须始终有效

### 生命周期

```ebnf
lifetime_annotation = "'" identifier ;
lifetime_parameter = lifetime_annotation [":" lifetime_bound] ;
lifetime_bound = lifetime_annotation { "+" lifetime_annotation } ;
```

## 模块系统

```ebnf
module_declaration = "mod" identifier [":" block] ;
use_declaration = "use" path ["as" identifier] ";" ;
path = identifier {"::" identifier} ;
visibility = "pub" ["(" visibility_scope ")"] ;
visibility_scope = "crate" | "module" | "super" | "self" ;
```

示例：
```aether
mod math {
    pub fn add(a: Int, b: Int) -> Int = a + b
    
    fn internal_helper() { }  // 私有函数
}

use math::add
use std::collections::HashMap as Map
```

## 标准库概览

### 核心模块

- `std::core` - 基本类型和原语
- `std::collections` - 数据结构
- `std::io` - 输入输出
- `std::fs` - 文件系统
- `std::net` - 网络编程
- `std::thread` - 线程管理
- `std::sync` - 同步原语
- `std::time` - 时间相关
- `std::fmt` - 格式化
- `std::result` - Result 类型
- `std::option` - Option 类型

## 版本信息

- 规范版本：1.0
- 最后更新：2024
- 状态：草案

---

*Aether Language Specification - Precision in Every Character*
