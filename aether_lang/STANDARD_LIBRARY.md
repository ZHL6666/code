# Aether 标准库设计文档

## 概述

Aether 标准库 (`aether_std`) 提供了一套完整、高效、易用的基础功能模块，遵循以下设计原则：

- **零成本抽象**: 运行时开销最小化
- **安全性优先**: 内存安全、线程安全
- **函数式风格**: 丰富的迭代器和组合子
- **异步原生**: 所有 I/O 操作天然支持 async/await
- **跨平台**: 统一接口适配多平台

## 模块结构

```
aether_std/
├── core/           # 核心类型和 trait (无依赖)
├── alloc/          # 内存分配相关
├── std/            # 标准库主体
│   ├── io/         # 输入输出
│   ├── fs/         # 文件系统
│   ├── net/        # 网络编程
│   ├── sync/       # 同步原语
│   ├── time/       # 时间处理
│   ├── collections/# 数据结构
│   ├── iter/       # 迭代器
│   ├── str/        # 字符串处理
│   ├── num/        # 数值计算
│   ├── result/     # 错误处理
│   ├── option/     # 可选值
│   └── env/        # 环境变量
└── test/           # 测试工具
```

## 核心模块详解

### 1. Core - 核心类型

```aether
// aether_std/core/src/lib.aether

/// 所有类型的根 trait
pub trait Any {
    fn type_id(self) -> TypeId
    fn as_any(self) -> AnyRef
}

/// 可复制类型标记
pub trait Copy: Clone {}

/// 克隆 trait
pub trait Clone {
    fn clone(self) -> Self
}

/// 大小固定类型
pub trait Sized {}

/// 可丢弃类型
pub trait Drop {
    fn drop(mut self)
}

/// 默认值
pub trait Default {
    fn default() -> Self
}

/// 调试格式化
pub trait Debug {
    fn fmt(self, f: Formatter) -> Result
}

/// 显示格式化
pub trait Display {
    fn fmt(self, f: Formatter) -> Result
}

/// 哈希
pub trait Hash {
    fn hash(self, state: &mut Hasher)
}

/// 相等性比较
pub trait PartialEq<Rhs = Self> {
    fn eq(self, other: Rhs) -> Bool
    fn ne(self, other: Rhs) -> Bool { !self.eq(other) }
}

/// 全序比较
pub trait PartialOrd<Rhs = Self>: PartialEq<Rhs> {
    fn partial_cmp(self, other: Rhs) -> Option<Ordering>
}

/// 全序
pub trait Ord: Eq + PartialOrd<Self> {
    fn cmp(self, other: Self) -> Ordering
}

/// 迭代器 trait
pub trait Iterator {
    type Item
    
    fn next(self) -> Option<Self::Item>
    
    // 适配器方法
    fn map<B>(self, f: Fn(Self::Item) -> B) -> Map<Self, F> where Self: Sized { ... }
    fn filter(self, predicate: Fn(Self::Item) -> Bool) -> Filter<Self, P> where Self: Sized { ... }
    fn flat_map<U, F>(self, f: F) -> FlatMap<Self, F, U> 
        where Self: Sized, F: Fn(Self::Item) -> U, U: IntoIterator 
    { ... }
    fn filter_map<B>(self, f: Fn(Self::Item) -> Option<B>) -> FilterMap<Self, F> where Self: Sized { ... }
    fn enumerate(self) -> Enumerate<Self> where Self: Sized { ... }
    fn zip<U>(self, other: U) -> Zip<Self, U::IntoIter> 
        where Self: Sized, U: IntoIterator 
    { ... }
    fn take(self, n: Int) -> Take<Self> where Self: Sized { ... }
    fn skip(self, n: Int) -> Skip<Self> where Self: Sized { ... }
    fn chain<U>(self, other: U) -> Chain<Self, U::IntoIter> 
        where Self: Sized, U: IntoIterator<Item = Self::Item> 
    { ... }
    
    // 消费方法
    fn collect<B>(self) -> B where B: FromIterator<Self::Item>, Self: Sized { ... }
    fn count(self) -> Int { ... }
    fn sum<S>(self) -> S where S: Sum<Self::Item>, Self: Sized { ... }
    fn product<P>(self) -> P where P: Product<Self::Item>, Self: Sized { ... }
    fn fold<B, F>(self, init: B, f: F) -> B 
        where F: FnMut(B, Self::Item) -> B 
    { ... }
    fn reduce<F>(self, f: F) -> Option<Self::Item> 
        where F: FnMut(Self::Item, Self::Item) -> Self::Item 
    { ... }
    fn all<F>(self, predicate: F) -> Bool 
        where F: FnMut(Self::Item) -> Bool 
    { ... }
    fn any<F>(self, predicate: F) -> Bool 
        where F: FnMut(Self::Item) -> Bool 
    { ... }
    fn find<F>(self, predicate: F) -> Option<Self::Item> 
        where F: FnMut(&Self::Item) -> Bool 
    { ... }
    fn position<F>(self, predicate: F) -> Option<Int> 
        where F: FnMut(Self::Item) -> Bool 
    { ... }
    fn max(self) -> Option<Self::Item> where Self::Item: Ord { ... }
    fn min(self) -> Option<Self::Item> where Self::Item: Ord { ... }
}

/// 可变迭代器
pub trait IteratorMut<T> {
    fn next_mut(self) -> Option<&mut T>
}

/// 可索引集合
pub trait Index<Idx> {
    type Output
    
    fn index(self, index: Idx) -> &Self::Output
}

/// 可变索引
pub trait IndexMut<Idx>: Index<Idx> {
    fn index_mut(self, index: Idx) -> &mut Self::Output
}

/// 从迭代器构建
pub trait FromIterator<A> {
    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item = A>
}

/// 转换为迭代器
pub trait IntoIterator {
    type Item
    type IntoIter: Iterator<Item = Self::Item>
    
    fn into_iter(self) -> Self::IntoIter
}

/// 求和
pub trait Sum<A> {
    fn sum<I>(iter: I) -> Self where I: Iterator<Item = A>
}

/// 乘积
pub trait Product<A> {
    fn product<I>(iter: I) -> Self where I: Iterator<Item = A>
}

/// 异步迭代器
pub trait AsyncIterator {
    type Item
    
    fn next(self) -> Future<Output = Option<Self::Item>>
}
```

### 2. Option - 可选值

```aether
// aether_std/option/src/lib.aether

/// 表示可选值的枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Option<T> {
    Some(T),
    None,
}

impl<T> Option<T> {
    /// 判断是否为 Some
    pub fn is_some(self) -> Bool {
        match self {
            Option::Some(_) => true,
            Option::None => false,
        }
    }
    
    /// 判断是否为 None
    pub fn is_none(self) -> Bool {
        !self.is_some()
    }
    
    /// 获取内部值，若为 None 则 panic
    pub fn unwrap(self) -> T {
        match self {
            Option::Some(val) => val,
            Option::None => panic!("called `Option::unwrap()` on a `None` value"),
        }
    }
    
    /// 获取内部值，若为 None 则返回默认值
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Option::Some(val) => val,
            Option::None => default,
        }
    }
    
    /// 获取内部值，若为 None 则调用函数生成默认值
    pub fn unwrap_or_else<F>(self, f: F) -> T 
        where F: FnOnce() -> T 
    {
        match self {
            Option::Some(val) => val,
            Option::None => f(),
        }
    }
    
    /// 映射 Option<T> 到 Option<U>
    pub fn map<U, F>(self, f: F) -> Option<U> 
        where F: FnOnce(T) -> U 
    {
        match self {
            Option::Some(val) => Option::Some(f(val)),
            Option::None => Option::None,
        }
    }
    
    /// 扁平化映射
    pub fn and_then<U, F>(self, f: F) -> Option<U> 
        where F: FnOnce(T) -> Option<U> 
    {
        match self {
            Option::Some(val) => f(val),
            Option::None => Option::None,
        }
    }
    
    /// 过滤
    pub fn filter<P>(self, predicate: P) -> Option<T> 
        where P: FnOnce(&T) -> Bool 
    {
        match self {
            Option::Some(val) if predicate(&val) => Option::Some(val),
            _ => Option::None,
        }
    }
    
    /// 转为 Result
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Option::Some(val) => Result::Ok(val),
            Option::None => Result::Err(err),
        }
    }
    
    /// 转为 Result，错误由函数生成
    pub fn ok_or_else<E, F>(self, err: F) -> Result<T, E> 
        where F: FnOnce() -> E 
    {
        match self {
            Option::Some(val) => Result::Ok(val),
            Option::None => Result::Err(err()),
        }
    }
    
    /// 若为 None 则执行函数
    pub fn inspect<F>(self, f: F) -> Self 
        where F: FnOnce(&T) 
    {
        if let Option::Some(ref val) = self {
            f(val)
        }
        self
    }
}

impl<T: Default> Option<T> {
    /// 若为 None 则使用默认值
    pub fn unwrap_or_default(self) -> T {
        match self {
            Option::Some(val) => val,
            Option::None => T::default(),
        }
    }
}

impl<T, E> Option<Option<T>> {
    /// 展平嵌套的 Option
    pub fn flatten(self) -> Option<T> {
        match self {
            Option::Some(inner) => inner,
            Option::None => Option::None,
        }
    }
}

// 实现 IntoIterator
impl<T> IntoIterator for Option<T> {
    type Item = T
    type IntoIter = OptionIter<T>
    
    fn into_iter(self) -> Self::IntoIter {
        OptionIter { opt: self }
    }
}

pub struct OptionIter<T> {
    opt: Option<T>,
}

impl<T> Iterator for OptionIter<T> {
    type Item = T
    
    fn next(self) -> Option<Self::Item> {
        self.opt.take()
    }
}

// 便捷函数
pub fn some<T>(val: T) -> Option<T> {
    Option::Some(val)
}

pub const fn none<T>() -> Option<T> {
    Option::None
}
```

### 3. Result - 错误处理

```aether
// aether_std/result/src/lib.aether

use crate::option::Option
use crate::fmt::{Debug, Display}

/// 表示操作结果的枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Result<T, E> {
    /// 判断是否为 Ok
    pub fn is_ok(self) -> Bool {
        match self {
            Result::Ok(_) => true,
            Result::Err(_) => false,
        }
    }
    
    /// 判断是否为 Err
    pub fn is_err(self) -> Bool {
        !self.is_ok()
    }
    
    /// 获取 Ok 中的值，若为 Err 则 panic
    pub fn unwrap(self) -> T {
        match self {
            Result::Ok(val) => val,
            Result::Err(err) => {
                panic!("called `Result::unwrap()` on an `Err` value: {:?}", err)
            }
        }
    }
    
    /// 获取 Ok 中的值，若为 Err 则返回默认值
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Result::Ok(val) => val,
            Result::Err(_) => default,
        }
    }
    
    /// 获取 Ok 中的值，若为 Err 则调用函数生成默认值
    pub fn unwrap_or_else<F>(self, op: F) -> T 
        where F: FnOnce(E) -> T 
    {
        match self {
            Result::Ok(val) => val,
            Result::Err(err) => op(err),
        }
    }
    
    /// 获取 Ok 中的引用
    pub fn as_ref(self) -> Result<&T, &E> {
        match self {
            Result::Ok(ref val) => Result::Ok(val),
            Result::Err(ref err) => Result::Err(err),
        }
    }
    
    /// 获取 Ok 中的可变引用
    pub fn as_mut(self) -> Result<&mut T, &mut E> {
        match self {
            Result::Ok(ref mut val) => Result::Ok(val),
            Result::Err(ref mut err) => Result::Err(err),
        }
    }
    
    /// 映射 Result<T, E> 到 Result<U, E>
    pub fn map<U, F>(self, op: F) -> Result<U, E> 
        where F: FnOnce(T) -> U 
    {
        match self {
            Result::Ok(val) => Result::Ok(op(val)),
            Result::Err(err) => Result::Err(err),
        }
    }
    
    /// 映射错误值
    pub fn map_err<U, O>(self, op: O) -> Result<T, U> 
        where O: FnOnce(E) -> U 
    {
        match self {
            Result::Ok(val) => Result::Ok(val),
            Result::Err(err) => Result::Err(op(err)),
        }
    }
    
    /// 扁平化映射
    pub fn and_then<U, F>(self, op: F) -> Result<U, E> 
        where F: FnOnce(T) -> Result<U, E> 
    {
        match self {
            Result::Ok(val) => op(val),
            Result::Err(err) => Result::Err(err),
        }
    }
    
    /// 若为 Err 则调用函数
    pub fn or_else<O>(self, op: O) -> Result<T, O> 
        where O: FnOnce(E) -> Result<T, O> 
    {
        match self {
            Result::Ok(val) => Result::Ok(val),
            Result::Err(err) => op(err),
        }
    }
    
    /// 检查并执行副作用
    pub fn inspect<F>(self, f: F) -> Self 
        where F: FnOnce(&T) 
    {
        if let Result::Ok(ref val) = self {
            f(val)
        }
        self
    }
    
    /// 对错误执行副作用
    pub fn inspect_err<F>(self, f: F) -> Self 
        where F: FnOnce(&E) 
    {
        if let Result::Err(ref err) = self {
            f(err)
        }
        self
    }
    
    /// 转为 Option
    pub fn ok(self) -> Option<T> {
        match self {
            Result::Ok(val) => Option::Some(val),
            Result::Err(_) => Option::None,
        }
    }
    
    /// 转为 Option，保留错误
    pub fn err(self) -> Option<E> {
        match self {
            Result::Ok(_) => Option::None,
            Result::Err(err) => Option::Some(err),
        }
    }
}

impl<T: Default, E> Result<T, E> {
    /// 若为 Err 则使用默认值
    pub fn unwrap_or_default(self) -> T {
        match self {
            Result::Ok(val) => val,
            Result::Err(_) => T::default(),
        }
    }
}

impl<T, E> Result<Result<T, E>, E> {
    /// 展平嵌套的 Result
    pub fn flatten(self) -> Result<T, E> {
        match self {
            Result::Ok(inner) => inner,
            Result::Err(err) => Result::Err(err),
        }
    }
}

// ? 运算符支持
impl<T, E, F> Try for Result<T, E> 
    where F: From<E>
{
    type Output = T
    type Residual = Result<Never, E>
    
    fn from_output(output: Self::Output) -> Self {
        Result::Ok(output)
    }
    
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Result::Ok(val) => ControlFlow::Continue(val),
            Result::Err(err) => ControlFlow::Break(Result::Err(err)),
        }
    }
}

// 便捷函数
pub fn ok<T, E>(val: T) -> Result<T, E> {
    Result::Ok(val)
}

pub fn err<T, E>(err: E) -> Result<T, E> {
    Result::Err(err)
}

// 常用别名
pub type IoResult<T> = Result<T, IoError>
pub type ParseResult<T> = Result<T, ParseError>
```

### 4. Collections - 数据结构

```aether
// aether_std/collections/src/lib.aether

mod vec
mod hashmap
mod hashset
mod btree_map
mod btree_set
mod linked_list
mod vec_deque
mod binary_heap

pub use vec::Vec
pub use hashmap::HashMap
pub use hashset::HashSet
pub use btree_map::BTreeMap
pub use btree_set::BTreeSet
pub use linked_list::LinkedList
pub use vec_deque::VecDeque
pub use binary_heap::BinaryHeap

/// 向量 - 可增长的数组
pub mod vec {
    use crate::alloc::Allocator
    use crate::iter::{Iterator, IntoIterator, FromIterator}
    
    #[derive(Debug)]
    pub struct Vec<T, A: Allocator = GlobalAlloc> {
        buf: RawVec<T, A>,
        len: Int,
    }
    
    impl<T> Vec<T> {
        /// 创建空向量
        pub fn new() -> Self {
            Self {
                buf: RawVec::new(),
                len: 0,
            }
        }
        
        /// 创建指定容量的向量
        pub fn with_capacity(capacity: Int) -> Self {
            Self {
                buf: RawVec::with_capacity(capacity),
                len: 0,
            }
        }
        
        /// 从迭代器创建
        pub fn from_iter<I>(iter: I) -> Self where I: IntoIterator<Item = T> {
            let mut vec = Vec::new()
            for item in iter {
                vec.push(item)
            }
            vec
        }
        
        /// 推入元素
        pub fn push(&mut self, value: T) {
            self.buf.grow_if_needed()
            unsafe {
                self.buf.write(self.len, value)
            }
            self.len += 1
        }
        
        /// 弹出最后一个元素
        pub fn pop(&mut self) -> Option<T> {
            if self.len == 0 {
                return None
            }
            self.len -= 1
            unsafe {
                Some(self.buf.read(self.len))
            }
        }
        
        /// 获取长度
        pub fn len(self) -> Int {
            self.len
        }
        
        /// 判断是否为空
        pub fn is_empty(self) -> Bool {
            self.len == 0
        }
        
        /// 获取容量
        pub fn capacity(self) -> Int {
            self.buf.capacity()
        }
        
        /// 预留空间
        pub fn reserve(&mut self, additional: Int) {
            self.buf.reserve(self.len, additional)
        }
        
        /// 收缩容量
        pub fn shrink_to_fit(&mut self) {
            self.buf.shrink_to_fit(self.len)
        }
        
        /// 切片
        pub fn as_slice(self) -> &[T] {
            unsafe {
                slice::from_raw_parts(self.buf.ptr(), self.len)
            }
        }
        
        /// 可变切片
        pub fn as_mut_slice(self) -> &mut [T] {
            unsafe {
                slice::from_raw_parts_mut(self.buf.ptr(), self.len)
            }
        }
        
        /// 获取元素
        pub fn get(&self, index: Int) -> Option<&T> {
            if index >= self.len {
                None
            } else {
                unsafe {
                    Some(&*self.buf.ptr().add(index))
                }
            }
        }
        
        /// 获取可变元素
        pub fn get_mut(&mut self, index: Int) -> Option<&mut T> {
            if index >= self.len {
                None
            } else {
                unsafe {
                    Some(&mut *self.buf.ptr().add(index))
                }
            }
        }
        
        /// 插入元素
        pub fn insert(&mut self, index: Int, element: T) {
            assert!(index <= self.len, "index out of bounds")
            self.reserve(1)
            unsafe {
                ptr::copy(
                    self.buf.ptr().add(index),
                    self.buf.ptr().add(index + 1),
                    self.len - index,
                )
                self.buf.write(index, element)
            }
            self.len += 1
        }
        
        /// 移除元素
        pub fn remove(&mut self, index: Int) -> T {
            assert!(index < self.len, "index out of bounds")
            unsafe {
                let result = self.buf.read(index)
                ptr::copy(
                    self.buf.ptr().add(index + 1),
                    self.buf.ptr().add(index),
                    self.len - index - 1,
                )
                self.len -= 1
                result
            }
        }
        
        /// 清空
        pub fn clear(&mut self) {
            while let Some(_) = self.pop() {}
        }
        
        /// 调整大小
        pub fn resize(&mut self, new_len: Int, value: T) where T: Clone {
            if new_len > self.len {
                let additional = new_len - self.len
                self.reserve(additional)
                for _ in 0..additional {
                    self.push(value.clone())
                }
            } else {
                while self.len > new_len {
                    self.pop()
                }
            }
        }
    }
    
    impl<T: Clone> Vec<T> {
        /// 重复创建
        pub fn repeat(element: T, n: Int) -> Self {
            let mut vec = Vec::with_capacity(n)
            for _ in 0..n {
                vec.push(element.clone())
            }
            vec
        }
    }
    
    impl<T: PartialEq> Vec<T> {
        /// 查找元素
        pub fn contains(&self, value: &T) -> Bool {
            self.iter().any(|x| x == value)
        }
        
        /// 查找索引
        pub fn index_of(&self, value: &T) -> Option<Int> {
            self.iter().position(|x| x == value)
        }
    }
    
    // 实现各种 trait
    impl<T> Index<Int> for Vec<T> {
        type Output = T
        
        fn index(self, index: Int) -> &Self::Output {
            self.get(index).expect("index out of bounds")
        }
    }
    
    impl<T> IndexMut<Int> for Vec<T> {
        fn index_mut(self, index: Int) -> &mut Self::Output {
            self.get_mut(index).expect("index out of bounds")
        }
    }
    
    impl<T> IntoIterator for Vec<T> {
        type Item = T
        type IntoIter = VecIntoIter<T>
        
        fn into_iter(self) -> Self::IntoIter {
            VecIntoIter { vec: self, index: 0 }
        }
    }
    
    pub struct VecIntoIter<T> {
        vec: Vec<T>,
        index: Int,
    }
    
    impl<T> Iterator for VecIntoIter<T> {
        type Item = T
        
        fn next(self) -> Option<Self::Item> {
            if self.index >= self.vec.len {
                None
            } else {
                let result = unsafe {
                    self.vec.buf.read(self.index)
                }
                self.index += 1
                Some(result)
            }
        }
    }
    
    impl<T> FromIterator<T> for Vec<T> {
        fn from_iter<I>(iter: I) -> Self where I: IntoIterator<Item = T> {
            Vec::from_iter(iter)
        }
    }
}

/// 哈希表
pub mod hashmap {
    use crate::hash::{Hash, Hasher}
    use crate::collections::vec::Vec
    
    pub struct HashMap<K, V, S = RandomState> {
        table: HashTable<K, V>,
        hasher: S,
    }
    
    impl<K, V> HashMap<K, V> {
        pub fn new() -> Self {
            Self {
                table: HashTable::new(),
                hasher: RandomState::new(),
            }
        }
        
        pub fn with_capacity(capacity: Int) -> Self {
            Self {
                table: HashTable::with_capacity(capacity),
                hasher: RandomState::new(),
            }
        }
        
        pub fn insert(&mut self, key: K, value: V) -> Option<V> 
            where K: Eq + Hash 
        {
            self.table.insert(key, value, &self.hasher)
        }
        
        pub fn get(&self, key: &K) -> Option<&V> 
            where K: Eq + Hash 
        {
            self.table.get(key, &self.hasher)
        }
        
        pub fn get_mut(&mut self, key: &K) -> Option<&mut V> 
            where K: Eq + Hash 
        {
            self.table.get_mut(key, &self.hasher)
        }
        
        pub fn remove(&mut self, key: &K) -> Option<V> 
            where K: Eq + Hash 
        {
            self.table.remove(key, &self.hasher)
        }
        
        pub fn contains_key(&self, key: &K) -> Bool 
            where K: Eq + Hash 
        {
            self.get(key).is_some()
        }
        
        pub fn len(&self) -> Int {
            self.table.len()
        }
        
        pub fn is_empty(&self) -> Bool {
            self.len() == 0
        }
        
        pub fn clear(&mut self) {
            self.table.clear()
        }
        
        pub fn keys(&self) -> Keys<'_, K, V> {
            Keys { inner: self.table.iter() }
        }
        
        pub fn values(&self) -> Values<'_, K, V> {
            Values { inner: self.table.iter() }
        }
        
        pub fn iter(&self) -> Iter<'_, K, V> {
            self.table.iter()
        }
        
        pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
            self.table.iter_mut()
        }
    }
    
    // 便捷宏
    #[macro_export]
    macro_rules! hashmap {
        {$($key:expr => $value:expr),+ $(,)?} => {
            {
                let mut map = HashMap::new()
                $(map.insert($key, $value);)+
                map
            }
        };
    }
}

/// 哈希集合
pub mod hashset {
    use crate::collections::hashmap::HashMap
    
    pub struct HashSet<T, S = RandomState> {
        map: HashMap<T, ()>,
    }
    
    impl<T> HashSet<T> {
        pub fn new() -> Self {
            Self {
                map: HashMap::new(),
            }
        }
        
        pub fn insert(&mut self, value: T) -> Bool 
            where T: Eq + Hash 
        {
            self.map.insert(value, ()).is_none()
        }
        
        pub fn remove(&mut self, value: &T) -> Bool 
            where T: Eq + Hash 
        {
            self.map.remove(value).is_some()
        }
        
        pub fn contains(&self, value: &T) -> Bool 
            where T: Eq + Hash 
        {
            self.map.contains_key(value)
        }
        
        pub fn len(&self) -> Int {
            self.map.len()
        }
        
        pub fn is_empty(&self) -> Bool {
            self.map.is_empty()
        }
        
        pub fn clear(&mut self) {
            self.map.clear()
        }
        
        pub fn union<'a>(&'a self, other: &'a HashSet<T>) -> Union<'a, T> {
            Union { iter: self.iter(), other, seen: self.len() }
        }
        
        pub fn intersection<'a>(&'a self, other: &'a HashSet<T>) -> Intersection<'a, T> {
            Intersection { iter: self.iter(), other }
        }
        
        pub fn difference<'a>(&'a self, other: &'a HashSet<T>) -> Difference<'a, T> {
            Difference { iter: self.iter(), other }
        }
        
        pub fn is_subset(&self, other: &HashSet<T>) -> Bool 
            where T: Eq + Hash 
        {
            self.iter().all(|v| other.contains(v))
        }
        
        pub fn is_superset(&self, other: &HashSet<T>) -> Bool 
            where T: Eq + Hash 
        {
            other.is_subset(self)
        }
        
        pub fn is_disjoint(&self, other: &HashSet<T>) -> Bool 
            where T: Eq + Hash 
        {
            self.iter().all(|v| !other.contains(v))
        }
    }
    
    #[macro_export]
    macro_rules! hashset {
        {$($value:expr),+ $(,)?} => {
            {
                let mut set = HashSet::new()
                $(set.insert($value);)+
                set
            }
        };
    }
}
```

### 5. IO - 输入输出

```aether
// aether_std/io/src/lib.aether

use crate::result::{Result, IoError}
use crate::buffer::BufRead

/// 读取字节
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<Int>
    
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => return Err(IoError::UnexpectedEof),
                Ok(n) => {
                    let tmp = buf[n..].to_vec()
                    buf = tmp.as_mut_slice()
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
    
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<Int> {
        let mut count = 0
        loop {
            let len = buf.len()
            buf.resize(len + 1024, 0)
            match self.read(&mut buf[len..]) {
                Ok(0) => {
                    buf.truncate(len)
                    return Ok(count)
                }
                Ok(n) => {
                    count += n
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
    }
    
    fn read_to_string(&mut self, buf: &mut String) -> Result<Int> {
        let mut bytes = Vec::new()
        self.read_to_end(&mut bytes)?
        *buf = String::from_utf8(bytes)
            .map_err(|_| IoError::InvalidData)?
        Ok(bytes.len())
    }
    
    fn bytes(self) -> Bytes<Self> where Self: Sized {
        Bytes { reader: self }
    }
    
    fn chain<R>(self, next: R) -> Chain<Self, R> 
        where Self: Sized, R: Read 
    {
        Chain { first: self, second: next, done_first: false }
    }
    
    fn take(self, limit: u64) -> Take<Self> where Self: Sized {
        Take { reader: self, limit }
    }
}

/// 写入字节
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<Int>
    
    fn flush(&mut self) -> Result<()>
    
    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(IoError::WriteZero),
                Ok(n) => buf = &buf[n..],
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
    
    fn write_fmt(&mut self, args: FormatArgs) -> Result<()> {
        let buf = format!("{}", args)
        self.write_all(buf.as_bytes())
    }
    
    fn by_ref(&mut self) -> &mut Self {
        self
    }
}

/// 缓冲读取
pub trait BufRead: Read {
    fn fill_buf(&mut self) -> Result<&[u8]>
    
    fn consume(&mut self, amt: Int)
    
    fn read_line(&mut self, buf: &mut String) -> Result<Int> {
        let mut bytes_read = 0
        loop {
            let available = self.fill_buf()?
            if let Some(pos) = available.iter().position(|&b| b == b'\n') {
                buf.push_str(&String::from_utf8_lossy(&available[..pos + 1]))
                self.consume(pos + 1)
                bytes_read += pos + 1
                break
            }
            bytes_read += available.len()
            buf.push_str(&String::from_utf8_lossy(available))
            self.consume(available.len())
            if available.is_empty() {
                break
            }
        }
        Ok(bytes_read)
    }
    
    fn lines(self) -> Lines<Self> where Self: Sized {
        Lines { buf: self }
    }
}

/// 查找
pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>
}

pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

/// 错误类型
#[derive(Debug)]
pub struct IoError {
    kind: ErrorKind,
    message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    Interrupted,
    UnexpectedEof,
    Other,
}

impl IoError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: Some(message.into()),
        }
    }
    
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
    
    pub fn raw_os_error(&self) -> Option<Int> {
        None  // 平台相关
    }
}

// 标准流
pub fn stdin() -> Stdin {
    Stdin { inner: platform::stdin() }
}

pub fn stdout() -> Stdout {
    Stdout { inner: platform::stdout() }
}

pub fn stderr() -> Stderr {
    Stderr { inner: platform::stderr() }
}

pub struct Stdin {
    inner: platform::StdinInner,
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<Int> {
        self.inner.read(buf)
    }
}

pub struct Stdout {
    inner: platform::StdoutInner,
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<Int> {
        self.inner.write(buf)
    }
    
    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

pub struct Stderr {
    inner: platform::StderrInner,
}

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> Result<Int> {
        self.inner.write(buf)
    }
    
    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

// 文件
pub struct File {
    inner: platform::FileInner,
}

impl File {
    pub fn open(path: &str) -> Result<Self> {
        Ok(Self {
            inner: platform::FileInner::open(path)?,
        })
    }
    
    pub fn create(path: &str) -> Result<Self> {
        Ok(Self {
            inner: platform::FileInner::create(path)?,
        })
    }
    
    pub fn metadata(&self) -> Result<Metadata> {
        self.inner.metadata()
    }
    
    pub fn set_permissions(&mut self, perms: Permissions) -> Result<()> {
        self.inner.set_permissions(perms)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<Int> {
        self.inner.read(buf)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<Int> {
        self.inner.write(buf)
    }
    
    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.inner.seek(pos)
    }
}

pub struct Metadata {
    len: u64,
    is_dir: Bool,
    is_file: Bool,
    permissions: Permissions,
    modified: Option<Time>,
    accessed: Option<Time>,
    created: Option<Time>,
}

impl Metadata {
    pub fn len(&self) -> u64 { self.len }
    pub fn is_dir(&self) -> Bool { self.is_dir }
    pub fn is_file(&self) -> Bool { self.is_file }
    pub fn permissions(&self) -> &Permissions { &self.permissions }
    pub fn modified(&self) -> Result<Time> {
        self.modified.ok_or(IoError::new(ErrorKind::Other, "modified time not available"))
    }
    pub fn accessed(&self) -> Result<Time> {
        self.accessed.ok_or(IoError::new(ErrorKind::Other, "accessed time not available"))
    }
    pub fn created(&self) -> Result<Time> {
        self.created.ok_or(IoError::new(ErrorKind::Other, "created time not available"))
    }
}

pub struct Permissions {
    readonly: Bool,
    mode: u32,  // Unix 权限位
}

impl Permissions {
    pub fn readonly(&self) -> Bool { self.readonly }
    pub fn set_readonly(&mut self, readonly: Bool) { self.readonly = readonly }
}

// 辅助函数
pub fn copy<R, W>(reader: &mut R, writer: &mut W) -> Result<u64>
    where R: Read, W: Write 
{
    let mut buf = [0; 8192]
    let mut written = 0
    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
        writer.write_all(&buf[..len])?
        written += len as u64
    }
}

pub fn read_to_string(path: &str) -> Result<String> {
    let mut file = File::open(path)?
    let mut contents = String::new()
    file.read_to_string(&mut contents)?
    Ok(contents)
}

pub fn write_all(path: &str, contents: &str) -> Result<()> {
    let mut file = File::create(path)?
    file.write_all(contents.as_bytes())
}
```

## 性能优化策略

1. **内联小函数**: 使用 `#[inline]` 标记热点函数
2. **避免分配**: 提供 `with_capacity` 系列 API
3. **迭代器惰性求值**: 链式调用不产生中间结果
4. **SIMD 加速**: 对数值运算自动向量化
5. **缓存友好**: 数据结构布局优化 CPU 缓存命中率

## 扩展机制

```aether
// 用户可扩展标准库
trait MyExtension {
    fn custom_method(self) -> Int
}

impl MyExtension for Int {
    fn custom_method(self) -> Int {
        self * 2
    }
}

// 使用
let x = 5.custom_method()  // 10
```

---

本文档定义了 Aether 标准库的核心接口和实现细节，为标准库开发者和使用者提供参考。
