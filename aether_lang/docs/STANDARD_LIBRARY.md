# Aether 标准库文档

## 模块概览

| 模块 | 描述 | 稳定性 |
|------|------|--------|
| `std::core` | 核心原语（无依赖） | Stable |
| `std::mem` | 内存管理 | Stable |
| `std::ops` | 运算符重载 | Stable |
| `std::result` | Result 和 Option | Stable |
| `std::iter` | 迭代器 | Stable |
| `std::collections` | 数据结构 | Stable |
| `std::io` | 输入输出 | Stable |
| `std::fs` | 文件系统 | Stable |
| `std::net` | 网络编程 | Stable |
| `std::async` | 异步运行时 | Stable |
| `std::sync` | 并发原语 | Stable |
| `std::time` | 时间处理 | Stable |

---

## std::core

基础类型和 trait，无需标准库即可使用。

```ae
// 基本类型
let x: i32 = 42;
let y: f64 = 3.14;
let z: bool = true;
let s: str = "hello";

// Trait 定义
trait Display {
    fn fmt(&self, f: &mut Formatter) -> Result;
}

// 宏
assert!(x > 0);
debug_assert!(y > 3.0);
println!("Value: {}", x);
```

### 关键类型
- `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- `f32`, `f64`
- `bool`, `char`, `str`
- `()`, `!`, `never`

### 关键 Trait
- `Copy`, `Clone`, `Drop`
- `Sized`, `Unsize`
- `Display`, `Debug`
- `Eq`, `PartialEq`, `Ord`, `PartialOrd`
- `Add`, `Sub`, `Mul`, `Div`, `Rem`
- `Iterator`, `IntoIterator`
- `From`, `Into`, `TryFrom`, `TryInto`

---

## std::mem

内存操作工具。

```ae
use std::mem;

// 大小和对齐
let size = mem::size_of::<i32>();      // 4
let align = mem::align_of::<i64>();    // 8

// 交换
let mut a = 1;
let mut b = 2;
mem::swap(&mut a, &mut b);

// 替换
let old = mem::replace(&mut a, 42);

// 遗忘（阻止 drop）
mem::forget(expensive_resource);

// 未初始化（不安全）
unsafe {
    let mut buf: [u8; 1024] = mem::uninitialized();
}

// 判别式
enum MyEnum { A(i32), B(String) }
let tag = mem::discriminant(&MyEnum::A(42));
```

---

## std::result & std::option

错误处理和可选值。

```ae
use std::result::Result::{self, Ok, Err};
use std::option::Option::{self, Some, None};

// Result 使用
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// 模式匹配
match divide(10.0, 2.0) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => eprintln!("Error: {}", e),
}

// 组合子
let value = divide(10.0, 0.0)
    .unwrap_or(1.0)
    .map(|x| x * 2.0)
    .unwrap_err();

// ? 运算符
fn complex_calculation() -> Result<f64, String> {
    let x = divide(10.0, 2.0)?;
    let y = divide(x, 5.0)?;
    Ok(y)
}

// Option 使用
fn find_user(id: u32) -> Option<User> {
    // ...
    None
}

let name = find_user(42).map(|u| u.name).unwrap_or("Anonymous".to_string());
```

---

## std::iter

迭代器和函数式编程。

```ae
use std::iter;

// 创建迭代器
let nums = vec![1, 2, 3, 4, 5];
let iter = nums.iter();

// 适配器
let sum: i32 = nums.iter()
    .filter(|&x| x % 2 == 0)
    .map(|x| x * 2)
    .sum();

// collect
let doubled: Vec<i32> = nums.iter()
    .map(|x| x * 2)
    .collect();

// 链式
let result = nums.iter()
    .enumerate()
    .filter(|(_, &x)| x > 2)
    .map(|(i, &x)| (i, x * 10))
    .fold(0, |acc, (_, x)| acc + x);

// 自定义迭代器
struct Counter { count: u32 }

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 10 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}
```

---

## std::collections

数据结构。

```ae
use std::collections::{Vec, HashMap, HashSet, BTreeMap, LinkedList, VecDeque};

// Vec - 动态数组
let mut vec: Vec<i32> = Vec::new();
vec.push(1);
vec.push(2);
vec.extend([3, 4, 5]);
let val = vec[2];

// HashMap - 哈希表
let mut map: HashMap<String, i32> = HashMap::new();
map.insert("one".to_string(), 1);
map.insert("two".to_string(), 2);

if let Some(val) = map.get("one") {
    println!("Found: {}", val);
}

// entry API
map.entry("three".to_string()).or_insert(3);

// HashSet - 集合
let set: HashSet<i32> = [1, 2, 3, 4].iter().cloned().collect();
if set.contains(&2) {
    println!("Contains 2");
}

// BTreeMap - 有序映射
let mut btree: BTreeMap<i32, String> = BTreeMap::new();
btree.insert(3, "three".to_string());
btree.insert(1, "one".to_string());

for (k, v) in btree.iter() {
    println!("{}: {}", k, v);  // 有序输出
}

// VecDeque - 双端队列
let mut deque: VecDeque<i32> = VecDeque::new();
deque.push_front(1);
deque.push_back(2);
let front = deque.pop_front();
```

---

## std::io

输入输出。

```ae
use std::io::{self, Read, Write, BufRead, BufReader, BufWriter};
use std::fs::File;

// 读取文件
let mut file = File::open("data.txt")?;
let mut contents = String::new();
file.read_to_string(&mut contents)?;

// 缓冲读取
let file = File::open("large.txt")?;
let reader = BufReader::new(file);

for line in reader.lines() {
    let line = line?;
    println!("{}", line);
}

// 写入文件
let mut file = File::create("output.txt")?;
file.write_all(b"Hello, World!")?;

// 缓冲写入
let file = File::create("output.txt")?;
let mut writer = BufWriter::new(file);
writer.write_all(b"Buffered content")?;
writer.flush()?;

// stdin/stdout
let mut input = String::new();
io::stdin().read_line(&mut input)?;
print!("Enter name: ");
io::stdout().flush()?;

// 自定义 Reader/Writer
struct MyReader;

impl Read for MyReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // 实现读取逻辑
        Ok(0)
    }
}
```

---

## std::fs

文件系统操作。

```ae
use std::fs::{self, File, OpenOptions, DirEntry};
use std::path::{Path, PathBuf};
use std::io::Write;

// 读取整个文件
let contents = fs::read_to_string("config.toml")?;

// 写入文件
fs::write("output.txt", "Hello!")?;

// 元数据
let metadata = fs::metadata("file.txt")?;
if metadata.is_file() {
    println!("Size: {} bytes", metadata.len());
}

// 创建目录
fs::create_dir("new_dir")?;
fs::create_dir_all("nested/deep/dir")?;

// 复制/移动/删除
fs::copy("src.txt", "dst.txt")?;
fs::rename("old.txt", "new.txt")?;
fs::remove_file("temp.txt")?;
fs::remove_dir("empty_dir")?;
fs::remove_dir_all("dir_with_content")?;

// 遍历目录
for entry in fs::read_dir(".")? {
    let entry = entry?;
    let path = entry.path();
    println!("Found: {:?}", path);
}

// 路径操作
let path = Path::new("/home/user/file.txt");
println!("Parent: {:?}", path.parent());
println!("Extension: {:?}", path.extension());

let mut path_buf = PathBuf::from("/home");
path_buf.push("user");
path_buf.push("docs");
```

---

## std::net

网络编程。

```ae
use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddr, IpAddr};
use std::io::{Read, Write};

// TCP 服务器
let listener = TcpListener::bind("127.0.0.1:8080")?;

for stream in listener.incoming() {
    let mut stream = stream?;
    
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer)?;
        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\nHello")?;
        Ok::<_, io::Error>(())
    });
}

// TCP 客户端
let mut stream = TcpStream::connect("127.0.0.1:8080")?;
stream.write_all(b"GET / HTTP/1.1\r\n\r\n")?;

let mut response = String::new();
stream.read_to_string(&mut response)?;

// UDP
let socket = UdpSocket::bind("127.0.0.1:0")?;
socket.send_to(b"Hello", "127.0.0.1:8080")?;

let mut buf = [0; 1024];
let (len, addr) = socket.recv_from(&mut buf)?;

// IP 地址
let ip: IpAddr = "192.168.1.1".parse()?;
let addr: SocketAddr = "127.0.0.1:8080".parse()?;
```

---

## std::async

异步编程。

```ae
use std::async::{Future, spawn, sleep, join, select};
use std::time::Duration;

// async 函数
async fn fetch_data(url: &str) -> Result<String, Error> {
    // 模拟网络请求
    sleep(Duration::from_millis(100)).await;
    Ok("data".to_string())
}

// spawn 任务
let handle = spawn(async {
    let result = fetch_data("https://api.example.com").await;
    result.unwrap()
});

let data = handle.await?;

// join 多个任务
let (a, b) = join(
    fetch_data("url1"),
    fetch_data("url2")
).await;

// select 第一个完成
select! {
    result = fetch_data("fast") => println!("Fast: {}", result?),
    result = fetch_data("slow") => println!("Slow: {}", result?),
}

// 超时
use std::async::timeout;

match timeout(Duration::from_secs(5), fetch_data("slow")).await {
    Ok(result) => println!("Completed: {}", result?),
    Err(_) => println!("Timed out"),
}

// Stream
use std::async::stream::{Stream, StreamExt};

async fn number_stream() -> impl Stream<Item = i32> {
    stream::iter(1..=5)
}

let mut stream = number_stream().await;
while let Some(num) = stream.next().await {
    println!("{}", num);
}
```

---

## std::sync

并发同步原语。

```ae
use std::sync::{Arc, Mutex, RwLock, Condvar, atomic::{AtomicUsize, Ordering}};
use std::thread;

// Arc - 原子引用计数
let data = Arc::new(vec![1, 2, 3]);
let mut handles = vec![];

for i in 0..3 {
    let data_clone = Arc::clone(&data);
    handles.push(thread::spawn(move || {
        println!("Thread {}: {:?}", i, data_clone);
    }));
}

// Mutex - 互斥锁
let counter = Mutex::new(0);

{
    let mut num = counter.lock()?;
    *num += 1;
}

// RwLock - 读写锁
let cache = RwLock::new(HashMap::new());

// 读（多个）
{
    let data = cache.read()?;
    println!("{:?}", data);
}

// 写（独占）
{
    let mut data = cache.write()?;
    data.insert("key", "value");
}

// 条件变量
let pair = Arc::new((Mutex::new(false), Condvar::new()));
let pair_clone = Arc::clone(&pair);

thread::spawn(move || {
    let (lock, cvar) = &*pair_clone;
    let mut started = lock.lock()?;
    *started = true;
    cvar.notify_one();
});

let (lock, cvar) = &*pair;
let mut started = lock.lock()?;
while !*started {
    started = cvar.wait(started)?;
}

// 原子操作
let counter = AtomicUsize::new(0);
counter.fetch_add(1, Ordering::SeqCst);
let val = counter.load(Ordering::Relaxed);
```

---

## std::time

时间处理。

```ae
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// Duration
let duration = Duration::new(5, 0);  // 5 秒
let millis = Duration::from_millis(500);
let total = duration + millis;

// Instant - 单调时钟
let start = Instant::now();

// 执行某些操作
thread::sleep(Duration::from_millis(100));

let elapsed = start.elapsed();
println!("Elapsed: {:?}", elapsed);

// SystemTime - 系统时间
let now = SystemTime::now();

let since_epoch = now.duration_since(UNIX_EPOCH)?;
println!("Timestamp: {}", since_epoch.as_secs());

// 日期计算
let future = now + Duration::from_secs(3600);
match future.duration_since(now) {
    Ok(diff) => println!("1 hour from now"),
    Err(_) => println!("Something went wrong"),
}
```

---

## 错误处理最佳实践

```ae
// 自定义错误类型
#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Parse(ParseError),
    NotFound(String),
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

// 使用 Result 别名
type Result<T> = std::result::Result<T, AppError>;

fn process_file(path: &str) -> Result<Data> {
    let contents = fs::read_to_string(path)?;  // 自动转换
    parse_data(&contents)
}

// anyhow 风格（快速原型）
use anyhow::{Result, Context, bail};

fn risky_operation() -> Result<()> {
    let file = fs::File::open("config.json")
        .context("Failed to open config file")?;
    
    if some_condition {
        bail!("Invalid configuration");
    }
    
    Ok(())
}
```

---

## 性能提示

1. **使用 `Box<T>`** 用于大型数据结构堆分配
2. **使用 `Rc<T>`/`Arc<T>`** 用于共享所有权
3. **使用 `Cow<T>`** 避免不必要的克隆
4. **使用 `SmallVec`** 优化小数组
5. **预分配容量**：`Vec::with_capacity(n)`
6. **使用迭代器** 而非中间集合
7. **使用 `mem::swap`** 而非手动交换
