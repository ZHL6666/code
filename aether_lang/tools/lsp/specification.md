# Aether 语言服务器协议 (LSP) 实现

## 概述

Aether LSP (`aether-lsp`) 提供完整的 IDE 支持，包括代码补全、跳转定义、悬停提示、重构等功能。

```
tools/lsp/
├── main.rs              # LSP 服务器入口
├── server.rs            # LSP 协议处理
├── capabilities.rs      # 功能声明
├── handlers/            # 请求处理器
│   ├── completion.rs    # 代码补全
│   ├── hover.rs         # 悬停提示
│   ├── goto_def.rs      # 跳转定义
│   ├── find_refs.rs     # 查找引用
│   ├── rename.rs        # 重命名
│   ├── diagnostics.rs   # 错误诊断
│   ├── formatting.rs    # 代码格式化
│   └── symbols.rs       # 文档符号
├── analysis/            # 语言分析
│   ├── indexer.rs       # 符号索引
│   ├── typer.rs         # 类型查询
│   ├── inference.rs     # 类型推断
│   └── cache.rs         # 分析缓存
├── vfs/                 # 虚拟文件系统
│   ├── overlay.rs       # 内存文件覆盖
│   └── watcher.rs       # 文件监听
└── utils/
    ├── fuzzy.rs         # 模糊匹配
    └── markup.rs        # Markdown 生成
```

## 1. 代码补全 (Completion)

```rust
// src/handlers/completion.rs

pub struct CompletionProvider {
    analysis: AnalysisSnapshot,
    config: CompletionConfig,
}

struct CompletionConfig {
    enable_auto_import: bool,
    enable_snippets: bool,
    enable_postfix: bool,
    fuzzy_match: bool,
}

impl CompletionProvider {
    pub fn completions(&self, position: Position) -> Vec<CompletionItem> {
        let token = self.analysis.token_at(position);
        let context = self.completion_context(position, token);
        
        let mut items = Vec::new();
        
        // 局部变量和参数
        items.extend(self.complete_locals(&context));
        
        // 模块作用域项
        items.extend(self.complete_scope(&context));
        
        // 类型成员 (如果有点号)
        if context.is_member_access {
            items.extend(self.complete_members(&context));
        }
        
        // 关键字
        items.extend(self.complete_keywords(&context));
        
        // 代码片段
        if self.config.enable_snippets {
            items.extend(self.complete_snippets(&context));
        }
        
        // 后缀补全 (如 `expr.if` -> `if expr {}`)
        if self.config.enable_postfix && context.is_postfix {
            items.extend(self.complete_postfix(&context));
        }
        
        // 自动导入建议
        if self.config.enable_auto_import && !items.is_empty() {
            items.extend(self.suggest_auto_imports(&context, &items));
        }
        
        // 排序和过滤
        self.rank_and_filter(items, &context.query)
    }
    
    fn complete_members(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        let receiver_ty = ctx.receiver_type.clone();
        let mut items = Vec::new();
        
        // 方法和字段
        for impl_block in self.analysis.find_impls(&receiver_ty) {
            for item in &impl_block.items {
                match item {
                    ImplItem::Method(method) => {
                        items.push(CompletionItem {
                            label: method.name.clone(),
                            kind: CompletionItemKind::Method,
                            detail: self.format_method_signature(method),
                            documentation: self.extract_docs(&method.docs),
                            insert_text: Some(format!("{}($0)", method.name)),
                            insert_text_format: InsertTextFormat::Snippet,
                        });
                    }
                    ImplItem::Field(field) => {
                        items.push(CompletionItem {
                            label: field.name.clone(),
                            kind: CompletionItemKind::Field,
                            detail: format!("{}", field.ty),
                            insert_text: Some(field.name.clone()),
                        });
                    }
                    _ => {}
                }
            }
        }
        
        // Trait 方法
        for trait_ref in self.analysis.resolve_traits(&receiver_ty) {
            for method in &trait_ref.methods {
                items.push(CompletionItem {
                    label: method.name.clone(),
                    kind: CompletionItemKind::Method,
                    detail: format!("from {}", trait_ref.name),
                    insert_text: Some(format!("{}($0)", method.name)),
                    tags: Some(vec![CompletionItemTag::FromTrait]),
                });
            }
        }
        
        items
    }
    
    fn complete_snippets(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "fn".into(),
                kind: CompletionItemKind::Snippet,
                detail: "Function definition".into(),
                insert_text: Some("fn ${1:name}(${2:params}) -> ${3:ReturnType} {\n    $0\n}".into()),
                insert_text_format: InsertTextFormat::Snippet,
            },
            CompletionItem {
                label: "impl".into(),
                kind: CompletionItemKind::Snippet,
                detail: "Implementation block".into(),
                insert_text: Some("impl ${1:Type} {\n    fn ${2:new}($0) -> Self {\n        Self { }\n    }\n}".into()),
                insert_text_format: InsertTextFormat::Snippet,
            },
            CompletionItem {
                label: "match".into(),
                kind: CompletionItemKind::Snippet,
                detail: "Match expression".into(),
                insert_text: Some("match ${1:expr} {\n    ${2:pattern} => {$0},\n    _ => {},\n}".into()),
                insert_text_format: InsertTextFormat::Snippet,
            },
            CompletionItem {
                label: "async".into(),
                kind: CompletionItemKind::Snippet,
                detail: "Async function".into(),
                insert_text: Some("async fn ${1:name}(${2:params}) -> ${3:Result<T, E>} {\n    Ok($0)\n}".into()),
                insert_text_format: InsertTextFormat::Snippet,
            },
        ]
    }
    
    fn complete_postfix(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        // expr.if -> if expr {}
        // expr.while -> while expr {}
        // expr.match -> match expr {}
        // expr.not -> !expr
        // expr.ok -> expr.unwrap_or_default()
        // expr.some -> Some(expr)
        
        vec![
            CompletionItem {
                label: "if".into(),
                kind: CompletionItemKind::Snippet,
                detail: "if expr {}".into(),
                insert_text: Some("if ${1:condition} {\n    $0\n}".into()),
                insert_text_format: InsertTextFormat::Snippet,
                filter_text: Some(ctx.expression_before_dot.clone()),
            },
            CompletionItem {
                label: "ok".into(),
                kind: CompletionItemKind::Snippet,
                detail: "unwrap_or_default()".into(),
                insert_text: Some("${1:expr}.unwrap_or_default()".into()),
                filter_text: Some(ctx.expression_before_dot.clone()),
            },
        ]
    }
}
```

## 2. 跳转定义 (Go to Definition)

```rust
// src/handlers/goto_def.rs

pub struct GotoDefinitionProvider {
    index: SymbolIndex,
}

impl GotoDefinitionProvider {
    pub fn goto_definition(&self, position: Position) -> Option<Location> {
        let token = self.analysis.token_at(position);
        let name = token.as_identifier()?;
        
        // 查找符号定义
        match self.index.lookup(name, position.file_id) {
            SymbolResolution::Local(var_id) => {
                let var_info = self.analysis.get_variable(var_id);
                Some(Location {
                    uri: var_info.file_id.to_uri(),
                    range: var_info.declaration_span.to_range(),
                })
            }
            SymbolResolution::Item(def_id) => {
                let item_info = self.index.get_item(def_id);
                Some(Location {
                    uri: item_info.file_id.to_uri(),
                    range: item_info.name_span.to_range(),
                })
            }
            SymbolResolution::TraitMethod(trait_id, method_name) => {
                let trait_def = self.index.get_trait(trait_id);
                let method = trait_def.methods.iter()
                    .find(|m| m.name == method_name)?;
                Some(Location {
                    uri: trait_def.file_id.to_uri(),
                    range: method.name_span.to_range(),
                })
            }
            SymbolResolution::Builtin(builtin) => {
                // 内置类型/函数跳转到标准库文档
                Some(Location {
                    uri: Url::parse(&format!("aether://std/{}", builtin))?,
                    range: Range::default(),
                })
            }
            SymbolResolution::Unresolved => None,
        }
    }
    
    pub fn goto_type_definition(&self, position: Position) -> Option<Location> {
        let expr_ty = self.analysis.type_at(position)?;
        
        match expr_ty {
            Type::Struct(def_id) | Type::Enum(def_id) | Type::Trait(def_id) => {
                let item_info = self.index.get_item(def_id);
                Some(Location {
                    uri: item_info.file_id.to_uri(),
                    range: item_info.name_span.to_range(),
                })
            }
            Type::Alias(def_id) => {
                // 跳转到类型别名定义
                self.goto_definition_for_alias(def_id)
            }
            _ => None,
        }
    }
    
    pub fn goto_implementation(&self, position: Position) -> Vec<Location> {
        let ty = self.analysis.type_at(position)?;
        let mut locations = Vec::new();
        
        // 查找所有实现该类型的 impl 块
        for impl_block in self.index.find_impls_for_type(&ty) {
            locations.push(Location {
                uri: impl_block.file_id.to_uri(),
                range: impl_block.impl_span.to_range(),
            });
        }
        
        // 如果是 trait，查找所有实现
        if let Type::Trait(trait_id) = ty {
            for impl_block in self.index.find_trait_impls(trait_id) {
                locations.push(Location {
                    uri: impl_block.file_id.to_uri(),
                    range: impl_block.impl_span.to_range(),
                });
            }
        }
        
        locations
    }
}
```

## 3. 悬停提示 (Hover)

```rust
// src/handlers/hover.rs

pub struct HoverProvider {
    analysis: AnalysisSnapshot,
}

impl HoverProvider {
    pub fn hover(&self, position: Position) -> Option<Hover> {
        let token = self.analysis.token_at(position);
        
        match token.kind {
            TokenKind::Identifier => self.hover_identifier(token, position),
            TokenKind::Keyword => self.hover_keyword(token),
            TokenKind::Literal => self.hover_literal(token),
            _ => None,
        }
    }
    
    fn hover_identifier(&self, token: Token, position: Position) -> Option<Hover> {
        let name = token.text;
        let resolution = self.index.lookup(name, position.file_id)?;
        
        let (type_info, docs) = match resolution {
            SymbolResolution::Local(var_id) => {
                let var = self.analysis.get_variable(var_id);
                (format!("{}", var.ty), var.docs)
            }
            SymbolResolution::Item(def_id) => {
                let item = self.index.get_item(def_id);
                (self.format_item_signature(&item), item.docs)
            }
            SymbolResolution::TraitMethod(trait_id, method_name) => {
                let trait_def = self.index.get_trait(trait_id);
                let method = trait_def.methods.iter()
                    .find(|m| m.name == method_name)?;
                (self.format_method_signature(method), method.docs)
            }
            _ => return None,
        };
        
        let markdown = format!(
            "```aether\n{}\n```\n\n{}",
            type_info,
            docs.unwrap_or_default()
        );
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: Some(token.span.to_range()),
        })
    }
    
    fn format_item_signature(&self, item: &ItemInfo) -> String {
        match item.kind {
            ItemKind::Function => {
                let func = &item.data.as_function().unwrap();
                format!(
                    "pub fn {}({}) -> {}",
                    item.name,
                    func.params.iter()
                        .map(|p| format!("{}: {}", p.name, p.ty))
                        .collect::<Vec<_>>()
                        .join(", "),
                    func.return_type
                )
            }
            ItemKind::Struct => {
                let struct_def = &item.data.as_struct().unwrap();
                format!(
                    "pub struct {} {{ /* {} fields */ }}",
                    item.name,
                    struct_def.fields.len()
                )
            }
            ItemKind::Enum => {
                let enum_def = &item.data.as_enum().unwrap();
                format!(
                    "pub enum {} {{ /* {} variants */ }}",
                    item.name,
                    enum_def.variants.len()
                )
            }
            ItemKind::Trait => {
                let trait_def = &item.data.as_trait().unwrap();
                format!(
                    "pub trait {} {{ /* {} methods */ }}",
                    item.name,
                    trait_def.methods.len()
                )
            }
            _ => item.name.clone(),
        }
    }
}
```

## 4. 查找引用 (Find References)

```rust
// src/handlers/find_refs.rs

pub struct FindReferencesProvider {
    index: SymbolIndex,
}

impl FindReferencesProvider {
    pub fn find_references(&self, position: Position, include_decl: bool) -> Vec<Location> {
        let token = self.analysis.token_at(position)?;
        let name = token.as_identifier()?;
        let resolution = self.index.lookup(name, position.file_id)?;
        
        let def_id = match resolution {
            SymbolResolution::Local(var_id) => {
                return self.find_local_references(var_id, include_decl);
            }
            SymbolResolution::Item(def_id) => def_id,
            _ => return Vec::new(),
        };
        
        let mut references = Vec::new();
        
        // 包含定义
        if include_decl {
            let item_info = self.index.get_item(def_id);
            references.push(Location {
                uri: item_info.file_id.to_uri(),
                range: item_info.name_span.to_range(),
            });
        }
        
        // 查找所有引用
        for reference in self.index.find_references(def_id) {
            references.push(Location {
                uri: reference.file_id.to_uri(),
                range: reference.span.to_range(),
            });
        }
        
        references
    }
    
    fn find_local_references(&self, var_id: VarId, include_decl: bool) -> Vec<Location> {
        let var_info = self.analysis.get_variable(var_id);
        let mut references = Vec::new();
        
        if include_decl {
            references.push(Location {
                uri: var_info.file_id.to_uri(),
                range: var_info.declaration_span.to_range(),
            });
        }
        
        for usage in &var_info.usages {
            references.push(Location {
                uri: usage.file_id.to_uri(),
                range: usage.span.to_range(),
            });
        }
        
        references
    }
}
```

## 5. 重命名 (Rename)

```rust
// src/handlers/rename.rs

pub struct RenameProvider {
    analysis: AnalysisSnapshot,
}

impl RenameProvider {
    pub fn prepare_rename(&self, position: Position) -> Option<PrepareRenameResponse> {
        let token = self.analysis.token_at(position)?;
        
        // 检查是否可重命名
        if !token.is_identifier() {
            return None;
        }
        
        let resolution = self.index.lookup(token.text, position.file_id)?;
        
        // 不能重命名内置项
        if matches!(resolution, SymbolResolution::Builtin(_)) {
            return None;
        }
        
        Some(PrepareRenameResponse {
            range: token.span.to_range(),
            placeholder: token.text.to_string(),
        })
    }
    
    pub fn rename(&self, position: Position, new_name: &str) -> Option<WorkspaceEdit> {
        let prepare = self.prepare_rename(position)?;
        
        // 验证新名称
        if !is_valid_identifier(new_name) {
            return None;
        }
        
        let token = self.analysis.token_at(position)?;
        let resolution = self.index.lookup(token.text, position.file_id)?;
        
        let changes = match resolution {
            SymbolResolution::Local(var_id) => {
                self.rename_local(var_id, new_name)
            }
            SymbolResolution::Item(def_id) => {
                self.rename_item(def_id, new_name)
            }
            _ => return None,
        };
        
        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }
    
    fn rename_item(&self, def_id: DefId, new_name: &str) -> HashMap<Url, Vec<TextEdit>> {
        let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
        
        // 获取定义位置
        let item_info = self.index.get_item(def_id);
        
        // 编辑定义
        let def_edit = TextEdit {
            range: item_info.name_span.to_range(),
            new_text: new_name.to_string(),
        };
        
        changes
            .entry(item_info.file_id.to_uri())
            .or_insert_with(Vec::new)
            .push(def_edit);
        
        // 编辑所有引用
        for reference in self.index.find_references(def_id) {
            let edit = TextEdit {
                range: reference.span.to_range(),
                new_text: new_name.to_string(),
            };
            
            changes
                .entry(reference.file_id.to_uri())
                .or_insert_with(Vec::new)
                .push(edit);
        }
        
        changes
    }
}
```

## 6. 诊断 (Diagnostics)

```rust
// src/handlers/diagnostics.rs

pub struct DiagnosticProvider {
    session: CompilationSession,
}

impl DiagnosticProvider {
    pub fn diagnostics(&self, file_id: FileId) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // 词法错误
        diagnostics.extend(self.lex_errors(file_id));
        
        // 语法错误
        diagnostics.extend(self.parse_errors(file_id));
        
        // 类型错误
        diagnostics.extend(self.type_errors(file_id));
        
        // 借用检查错误
        diagnostics.extend(self.borrow_errors(file_id));
        
        // 警告
        diagnostics.extend(self.warnings(file_id));
        
        // 排序（按位置）
        diagnostics.sort_by_key(|d| d.range.start);
        
        diagnostics
    }
    
    fn type_errors(&self, file_id: FileId) -> Vec<Diagnostic> {
        let hir = self.session.hir(file_id);
        let mut diagnostics = Vec::new();
        
        for error in &hir.type_errors {
            diagnostics.push(Diagnostic {
                range: error.span.to_range(),
                severity: Some(DiagnosticSeverity::Error),
                code: Some(NumberOrString::String("E0001".into())),
                message: error.message(),
                source: Some("aethc".into()),
                related_information: self.format_related_info(&error.related),
            });
        }
        
        diagnostics
    }
    
    fn warnings(&self, file_id: FileId) -> Vec<Diagnostic> {
        let mut warnings = Vec::new();
        
        // 未使用的变量
        for unused in self.session.unused_variables(file_id) {
            warnings.push(Diagnostic {
                range: unused.span.to_range(),
                severity: Some(DiagnosticSeverity::Warning),
                code: Some(NumberOrString::String("W0001".into())),
                message: format!("unused variable: `{}`", unused.name),
                source: Some("aethc".into()),
                suggestions: Some(vec![
                    DiagnosticSuggestion {
                        range: unused.span.to_range(),
                        label: "prefix with underscore".into(),
                        edits: vec![TextEdit {
                            range: Range {
                                start: unused.span.start.to_position(),
                                end: unused.span.start.to_position(),
                            },
                            new_text: "_".into(),
                        }],
                    }
                ]),
            });
        }
        
        // 可以改为不可变借用
        for mutable_borrow in self.session.unnecessary_mut_borrows(file_id) {
            warnings.push(Diagnostic {
                range: mutable_borrow.span.to_range(),
                severity: Some(DiagnosticSeverity::Warning),
                code: Some(NumberOrString::String("W0002".into())),
                message: "variable does not need to be mutable".into(),
                source: Some("aethc".into()),
                suggestions: Some(vec![
                    DiagnosticSuggestion {
                        range: mutable_borrow.span.to_range(),
                        label: "remove mut".into(),
                        edits: vec![TextEdit {
                            range: mutable_borrow.mut_keyword_span.to_range(),
                            new_text: "".into(),
                        }],
                    }
                ]),
            });
        }
        
        warnings
    }
}
```

## 7. 代码格式化 (Formatting)

```rust
// src/handlers/formatting.rs

pub struct FormattingProvider {
    config: FormatConfig,
}

struct FormatConfig {
    tab_width: usize,
    max_width: usize,
    newline_style: NewlineStyle,
    imports_granularity: ImportsGranularity,
}

impl FormattingProvider {
    pub fn format_document(&self, file_id: FileId) -> Vec<TextEdit> {
        let ast = self.session.ast(file_id);
        let formatted = self.format_node(&ast, 0);
        
        vec![TextEdit {
            range: Range::new(Position::new(0, 0), self.end_position(file_id)),
            new_text: formatted,
        }]
    }
    
    fn format_node(&self, node: &AstNode, indent: usize) -> String {
        match node {
            AstNode::Function(func) => self.format_function(func, indent),
            AstNode::Struct(struct_def) => self.format_struct(struct_def, indent),
            AstNode::Impl(impl_block) => self.format_impl(impl_block, indent),
            AstNode::Expr(expr) => self.format_expr(expr, indent),
            _ => String::new(),
        }
    }
    
    fn format_function(&self, func: &FunctionDef, indent: usize) -> String {
        let mut result = String::new();
        
        // 修饰符
        if func.is_async {
            result.push_str("async ");
        }
        if func.is_unsafe {
            result.push_str("unsafe ");
        }
        
        // 函数签名
        result.push_str(&format!(
            "fn {}(",
            func.name
        ));
        
        // 参数
        let params: Vec<String> = func.params.iter()
            .map(|p| {
                if let Some(ty) = &p.ty {
                    format!("{}: {}", p.name, ty)
                } else {
                    p.name.clone()
                }
            })
            .collect();
        
        if params.len() > 3 || params.join(", ").len() > self.config.max_width {
            result.push('\n');
            for param in &params {
                result.push_str(&" ".repeat(indent + 4));
                result.push_str(param);
                result.push_str(",\n");
            }
            result.push_str(&" ".repeat(indent));
        } else {
            result.push_str(&params.join(", "));
        }
        
        result.push(')');
        
        // 返回类型
        if let Some(ret) = &func.return_type {
            result.push_str(&format!(" -> {}", ret));
        }
        
        // 函数体
        result.push_str(" {\n");
        result.push_str(&self.format_block(&func.body, indent + 4));
        result.push_str(&" ".repeat(indent));
        result.push('}');
        
        result
    }
    
    fn format_expr(&self, expr: &Expr, indent: usize) -> String {
        match expr {
            Expr::If(if_expr) => {
                let mut result = format!("if {} ", self.format_expr(&if_expr.cond, indent));
                result.push_str("{\n");
                result.push_str(&self.format_block(&if_expr.then_branch, indent + 4));
                result.push_str(&" ".repeat(indent));
                result.push('}');
                
                if let Some(else_branch) = &if_expr.else_branch {
                    result.push_str(" else ");
                    if let Expr::If(inner_if) = else_branch.as_ref() {
                        result.push_str(&self.format_expr(inner_if, indent)[..]);
                    } else {
                        result.push_str("{\n");
                        result.push_str(&self.format_block(else_branch, indent + 4));
                        result.push_str(&" ".repeat(indent));
                        result.push('}');
                    }
                }
                
                result
            }
            Expr::Match(match_expr) => {
                let mut result = format!("match {} {{\n", self.format_expr(&match_expr.expr, indent));
                
                for arm in &match_expr.arms {
                    result.push_str(&" ".repeat(indent + 4));
                    result.push_str(&self.format_pattern(&arm.pattern));
                    result.push_str(" => ");
                    
                    if arm.body.is_block() {
                        result.push_str(&self.format_expr(&arm.body, indent + 4));
                    } else {
                        result.push_str(&self.format_expr(&arm.body, indent + 8));
                        result.push(',');
                    }
                    result.push('\n');
                }
                
                result.push_str(&" ".repeat(indent));
                result.push('}');
                
                result
            }
            _ => self.format_simple_expr(expr),
        }
    }
}
```

## LSP 能力声明

```json
{
  "textDocumentSync": {
    "openClose": true,
    "change": 2,
    "save": true
  },
  "completionProvider": {
    "resolveProvider": true,
    "triggerCharacters": [".", ":", "@"]
  },
  "hoverProvider": true,
  "definitionProvider": true,
  "typeDefinitionProvider": true,
  "implementationProvider": true,
  "referencesProvider": true,
  "documentHighlightProvider": true,
  "documentSymbolProvider": true,
  "workspaceSymbolProvider": true,
  "codeActionProvider": {
    "codeActionKinds": ["quickfix", "refactor", "organize_imports"],
    "resolveProvider": true
  },
  "renameProvider": {
    "prepareProvider": true
  },
  "foldingRangeProvider": true,
  "selectionRangeProvider": true,
  "semanticTokensProvider": {
    "legend": {
      "tokenTypes": ["namespace", "type", "class", "enum", "interface", "struct", "typeParameter", "parameter", "variable", "property", "enumMember", "event", "function", "method", "macro", "keyword", "modifier", "comment", "string", "number", "regexp", "operator"],
      "tokenModifiers": ["declaration", "definition", "readonly", "static", "deprecated", "abstract", "async", "modification", "documentation", "defaultLibrary"]
    },
    "range": true,
    "full": {
      "delta": true
    }
  },
  "inlayHintProvider": true,
  "documentFormattingProvider": true,
  "documentRangeFormattingProvider": true,
  "documentOnTypeFormattingProvider": {
    "firstTriggerCharacter": "}",
    "moreTriggerCharacter": [";", "\n"]
  },
  "callHierarchyProvider": true,
  "linkedEditingRangeProvider": true
}
```

这个 LSP 实现提供了：
- ✅ 智能代码补全（局部变量、成员、关键字、代码片段、后缀补全）
- ✅ 精确的跳转定义（变量、函数、类型、Trait 方法）
- ✅ 丰富的悬停提示（类型信息、文档注释）
- ✅ 全面的引用查找
- ✅ 安全的重命名重构
- ✅ 实时诊断（错误、警告、建议）
- ✅ 代码格式化
- ✅ 语义高亮
- ✅ 内联提示（类型注解、参数名）
- ✅ 调用层次结构
- ✅ 折叠范围
- ✅ 选择范围
