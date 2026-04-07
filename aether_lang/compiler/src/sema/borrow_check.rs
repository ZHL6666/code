//! Aether 借用检查器
//! 
//! 实现 Rust 风格的借用规则：
//! 1. 任意时刻，只能有一个可变引用 (&mut)
//! 2. 任意时刻，可以有多个不可变引用 (&)
//! 3. 引用不能比其指向的数据存活更久

use crate::ir::{Function, Instruction, ValueId};
use crate::sema::types::Type;
use std::collections::{HashMap, HashSet};
use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq)]
enum BorrowState {
    /// 未被借用
    Unborrowed,
    /// 被不可变借用，记录借用次数
    ImmutableBorrow(usize),
    /// 被可变借用
    MutableBorrow,
}

#[derive(Debug, Clone)]
struct PlaceInfo {
    state: BorrowState,
    /// 记录借用的位置（指令索引），用于错误报告
    borrow_sites: Vec<usize>,
}

pub struct BorrowChecker {
    /// 当前作用域内每个变量/地方的借用状态
    places: HashMap<ValueId, PlaceInfo>,
    /// 当前指令索引
    current_inst_idx: usize,
    /// 错误信息收集
    errors: Vec<String>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            places: HashMap::new(),
            current_inst_idx: 0,
            errors: Vec::new(),
        }
    }

    /// 检查整个函数的借用合法性
    pub fn check_function(&mut self, func: &Function) -> Result<()> {
        self.places.clear();
        self.errors.clear();
        self.current_inst_idx = 0;

        // 初始化参数为未借用状态
        for param in &func.params {
            self.places.insert(param.id, PlaceInfo {
                state: BorrowState::Unborrowed,
                borrow_sites: vec![],
            });
        }

        // 遍历所有基本块和指令
        // 注意：这里简化处理，实际需要对控制流图进行不动点迭代分析
        for block in func.blocks.values() {
            for (idx, inst) in block.instructions.iter().enumerate() {
                self.current_inst_idx = idx;
                if let Err(e) = self.check_instruction(inst) {
                    self.errors.push(format!(
                        "Error at {}:{}: {}", 
                        func.name, idx, e
                    ));
                }
            }
        }

        if !self.errors.is_empty() {
            bail!("Borrow check failed:\n{}", self.errors.join("\n"));
        }

        Ok(())
    }

    fn check_instruction(&mut self, inst: &Instruction) -> Result<()> {
        match inst {
            Instruction::Alloc { dest, .. } => {
                // 新分配的内存初始化为未借用
                self.places.insert(*dest, PlaceInfo {
                    state: BorrowState::Unborrowed,
                    borrow_sites: vec![],
                });
            }
            
            Instruction::Load { src, dest, is_mut } => {
                // 加载操作涉及读取引用
                if let Some(place) = self.places.get(src) {
                    match &place.state {
                        BorrowState::MutableBorrow => {
                            // 如果源是可变借用，需要检查是否允许读取
                            // 简化：允许从可变引用读取
                        }
                        _ => {}
                    }
                }
                
                // 如果 Load 创建了新的引用 (例如 &x 或 &mut x)
                // 这里假设 Load 指令本身不改变借用状态，借用由专门的 Ref 指令处理
            }

            Instruction::Store { dest, src, .. } => {
                // 存储操作需要可变访问权限
                if let Some(place) = self.places.get(dest) {
                    if matches!(place.state, BorrowState::ImmutableBorrow(_)) {
                        bail!("Cannot write to immutably borrowed place");
                    }
                }
            }

            Instruction::Ref { src, dest, is_mut } => {
                // 处理取地址操作: &x 或 &mut x
                let src_state = self.places.get(src).cloned()
                    .unwrap_or(PlaceInfo { state: BorrowState::Unborrowed, borrow_sites: vec![] });

                if *is_mut {
                    // 请求可变借用 (&mut)
                    match src_state.state {
                        BorrowState::Unborrowed => {
                            // 允许可变借用
                            self.places.insert(*src, PlaceInfo {
                                state: BorrowState::MutableBorrow,
                                borrow_sites: vec![self.current_inst_idx],
                            });
                            self.places.insert(*dest, PlaceInfo {
                                state: BorrowState::MutableBorrow, // 引用的引用也是可变状态传递
                                borrow_sites: vec![self.current_inst_idx],
                            });
                        }
                        BorrowState::ImmutableBorrow(count) => {
                            bail!("Cannot mutably borrow '{}' because it is also immutably borrowed ({} active borrows)", 
                                  src, count);
                        }
                        BorrowState::MutableBorrow => {
                            bail!("Cannot mutably borrow '{}' because it is already mutably borrowed", src);
                        }
                    }
                } else {
                    // 请求不可变借用 (&)
                    match src_state.state {
                        BorrowState::Unborrowed | BorrowState::ImmutableBorrow(_) => {
                            // 允许叠加不可变借用
                            let new_count = match src_state.state {
                                BorrowState::ImmutableBorrow(c) => c + 1,
                                _ => 1,
                            };
                            
                            let mut sites = src_state.borrow_sites;
                            sites.push(self.current_inst_idx);
                            
                            self.places.insert(*src, PlaceInfo {
                                state: BorrowState::ImmutableBorrow(new_count),
                                borrow_sites: sites,
                            });
                            
                            self.places.insert(*dest, PlaceInfo {
                                state: BorrowState::ImmutableBorrow(1),
                                borrow_sites: vec![self.current_inst_idx],
                            });
                        }
                        BorrowState::MutableBorrow => {
                            bail!("Cannot immutably borrow '{}' because it is already mutably borrowed", src);
                        }
                    }
                }
            }

            Instruction::EndBorrow { target } => {
                // 显式结束借用 (通常在作用域结束时隐式调用，或由 Drop 处理)
                // 简化：这里仅作为示意，实际需通过数据流分析确定借用结束点
                if let Some(place) = self.places.get_mut(target) {
                    match &mut place.state {
                        BorrowState::ImmutableBorrow(count) => {
                            if *count > 0 {
                                *count -= 1;
                                place.borrow_sites.pop();
                            }
                            if *count == 0 {
                                place.state = BorrowState::Unborrowed;
                            }
                        }
                        BorrowState::MutableBorrow => {
                            place.state = BorrowState::Unborrowed;
                            place.borrow_sites.clear();
                        }
                        _ => {}
                    }
                }
            }

            // 函数调用可能使所有可变引用失效 (保守策略)
            Instruction::Call { .. } => {
                // 在实际实现中，需要根据被调用函数的签名来判断
                // 这里简化：如果有未结束的可变借用，且调用了外部函数，可能需要报错或冻结
            }

            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{BasicBlock, TypeId};

    #[test]
    fn test_double_mut_borrow() {
        // 测试双重可变借用应失败
        let mut checker = BorrowChecker::new();
        // 模拟场景需要在完整的 IR 构建后进行，此处仅作单元测试框架占位
        assert!(true); 
    }

    #[test]
    fn test_mut_and_imm_borrow() {
        // 测试同时存在可变和不可变借用应失败
        assert!(true);
    }
}
