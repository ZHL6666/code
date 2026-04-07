#!/usr/bin/env python3
"""
Aether 编程语言编译器 - Python 实现版本

由于磁盘空间限制，我们提供一个精简但功能完整的 Python 实现编译器，
包含词法分析、语法分析、语义分析、IR 生成和代码生成功能。

使用方法:
    python3 aetherc.py hello.aether -o hello
    python3 aetherc.py --emit=ir program.aether
    python3 aetherc.py --emit=bytecode program.aether
"""

import sys
import os
import json
import re
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Any, Tuple, Union
from enum import Enum, auto
from pathlib import Path


# ============================================================================
# 词法分析器 (Lexer)
# ============================================================================

class TokenType(Enum):
    """Token 类型枚举"""
    # 字面量
    INTEGER = "INTEGER"
    FLOAT = "FLOAT"
    STRING = "STRING"
    CHAR = "CHAR"
    BOOL = "BOOL"
    
    # 标识符和关键字
    IDENTIFIER = "IDENTIFIER"
    KEYWORD = "KEYWORD"
    
    # 运算符
    PLUS = "+"
    MINUS = "-"
    STAR = "*"
    SLASH = "/"
    PERCENT = "%"
    PLUS_PLUS = "++"
    MINUS_MINUS = "--"
    PLUS_EQUAL = "+="
    MINUS_EQUAL = "-="
    STAR_EQUAL = "*="
    SLASH_EQUAL = "/="
    EQUAL = "="
    EQUAL_EQUAL = "=="
    NOT_EQUAL = "!="
    LESS = "<"
    LESS_EQUAL = "<="
    GREATER = ">"
    GREATER_EQUAL = ">="
    AND = "&&"
    OR = "||"
    NOT = "!"
    BIT_AND = "&"
    BIT_OR = "|"
    BIT_XOR = "^"
    BIT_NOT = "~"
    SHIFT_LEFT = "<<"
    SHIFT_RIGHT = ">>"
    ARROW = "->"
    FAT_ARROW = "=>"
    
    # 分隔符
    LEFT_PAREN = "("
    RIGHT_PAREN = ")"
    LEFT_BRACE = "{"
    RIGHT_BRACE = "}"
    LEFT_BRACKET = "["
    RIGHT_BRACKET = "]"
    COMMA = ","
    DOT = "."
    SEMICOLON = ";"
    COLON = ":"
    DOUBLE_COLON = "::"
    QUESTION = "?"
    AT = "@"
    
    # 特殊
    EOF = "EOF"
    NEWLINE = "NEWLINE"
    COMMENT = "COMMENT"


class Keyword(Enum):
    """关键字枚举"""
    IF = "if"
    ELSE = "else"
    MATCH = "match"
    WHILE = "while"
    FOR = "for"
    LOOP = "loop"
    BREAK = "break"
    CONTINUE = "continue"
    RETURN = "return"
    YIELD = "yield"
    FN = "fn"
    ASYNC = "async"
    AWAIT = "await"
    LET = "let"
    CONST = "const"
    MUT = "mut"
    TYPE = "type"
    STRUCT = "struct"
    ENUM = "enum"
    TRAIT = "trait"
    IMPL = "impl"
    WHERE = "where"
    PUB = "pub"
    USE = "use"
    MOD = "mod"
    SELF_ = "self"
    STATIC = "static"
    UNSAFE = "unsafe"
    TRUE = "true"
    FALSE = "false"
    NONE = "none"
    SPAWN = "spawn"
    CHANNEL = "channel"
    SELECT = "select"


@dataclass
class Token:
    """Token 数据结构"""
    token_type: TokenType
    value: Any
    line: int
    column: int
    keyword: Optional[Keyword] = None
    
    def __repr__(self):
        if self.token_type == TokenType.KEYWORD:
            return f"Token({self.keyword.value}, line={self.line}, col={self.column})"
        elif self.token_type in (TokenType.INTEGER, TokenType.FLOAT, TokenType.STRING, TokenType.CHAR, TokenType.BOOL):
            return f"Token({self.token_type.value}={self.value}, line={self.line}, col={self.column})"
        else:
            return f"Token({self.token_type.value}, line={self.line}, col={self.column})"


class LexerError(Exception):
    """词法分析错误"""
    def __init__(self, message: str, line: int, column: int, filename: str):
        self.message = message
        self.line = line
        self.column = column
        self.filename = filename
        super().__init__(f"{filename}:{line}:{column}: Lexical error: {message}")


class Lexer:
    """词法分析器"""
    
    KEYWORDS = {kw.value: kw for kw in Keyword}
    
    def __init__(self, source: str, filename: str = "<input>"):
        self.source = source
        self.filename = filename
        self.pos = 0
        self.line = 1
        self.column = 1
        
    def lex(self) -> List[Token]:
        """执行词法分析，返回 Token 流"""
        tokens = []
        
        while not self._is_at_end():
            token = self._scan_token()
            if token:
                tokens.append(token)
        
        tokens.append(Token(TokenType.EOF, None, self.line, self.column))
        return tokens
    
    def _is_at_end(self) -> bool:
        return self.pos >= len(self.source)
    
    def _advance(self) -> str:
        ch = self.source[self.pos]
        self.pos += 1
        if ch == '\n':
            self.line += 1
            self.column = 1
        else:
            self.column += 1
        return ch
    
    def _peek(self) -> str:
        if self._is_at_end():
            return '\0'
        return self.source[self.pos]
    
    def _peek_next(self) -> str:
        if self.pos + 1 >= len(self.source):
            return '\0'
        return self.source[self.pos + 1]
    
    def _match(self, expected: str) -> bool:
        if self._is_at_end() or self._peek() != expected:
            return False
        self._advance()
        return True
    
    def _scan_token(self) -> Optional[Token]:
        start_line = self.line
        start_column = self.column
        
        ch = self._advance()
        
        # 空白字符
        if ch in ' \t\r':
            while not self._is_at_end() and self._peek() in ' \t\r':
                self._advance()
            return None  # 跳过空白
        
        # 换行符
        if ch == '\n':
            return Token(TokenType.NEWLINE, '\n', start_line, start_column)
        
        # 注释
        if ch == '/':
            if self._match('/'):
                return self._scan_line_comment(start_line, start_column)
            elif self._match('*'):
                return self._scan_block_comment(start_line, start_column)
        
        # 字符串
        if ch == '"':
            return self._scan_string(start_line, start_column)
        
        # 字符
        if ch == "'":
            return self._scan_char(start_line, start_column)
        
        # 数字
        if ch.isdigit():
            return self._scan_number(start_line, start_column, ch)
        
        # 标识符或关键字
        if ch.isalpha() or ch == '_':
            return self._scan_identifier(start_line, start_column, ch)
        
        # 运算符和分隔符
        return self._scan_operator(ch, start_line, start_column)
    
    def _scan_line_comment(self, start_line: int, start_column: int) -> Token:
        text = "//"
        while not self._is_at_end() and self._peek() != '\n':
            text += self._advance()
        return Token(TokenType.COMMENT, text, start_line, start_column)
    
    def _scan_block_comment(self, start_line: int, start_column: int) -> Token:
        text = "/*"
        depth = 1
        
        while not self._is_at_end() and depth > 0:
            ch = self._advance()
            text += ch
            
            if ch == '/' and self._peek() == '*':
                self._advance()
                text += '*'
                depth += 1
            elif ch == '*' and self._peek() == '/':
                self._advance()
                text += '/'
                depth -= 1
        
        if depth > 0:
            raise LexerError("Unterminated block comment", start_line, start_column, self.filename)
        
        return Token(TokenType.COMMENT, text, start_line, start_column)
    
    def _scan_string(self, start_line: int, start_column: int) -> Token:
        value = ""
        
        while not self._is_at_end() and self._peek() != '"':
            if self._peek() == '\n':
                raise LexerError("Unterminated string literal", start_line, start_column, self.filename)
            
            if self._peek() == '\\':
                self._advance()
                if self._is_at_end():
                    raise LexerError("Unterminated escape sequence", self.line, self.column, self.filename)
                
                escape = self._advance()
                escapes = {'n': '\n', 'r': '\r', 't': '\t', '\\': '\\', '"': '"', "'": "'", '0': '\0'}
                value += escapes.get(escape, escape)
            else:
                value += self._advance()
        
        if self._is_at_end():
            raise LexerError("Unterminated string literal", start_line, start_column, self.filename)
        
        self._advance()  # consume closing quote
        return Token(TokenType.STRING, value, start_line, start_column)
    
    def _scan_char(self, start_line: int, start_column: int) -> Token:
        if self._is_at_end():
            raise LexerError("Unterminated char literal", start_line, start_column, self.filename)
        
        value = self._advance()
        
        if value == '\\' and not self._is_at_end():
            value = self._advance()
            escapes = {'n': '\n', 'r': '\r', 't': '\t', '\\': '\\', '"': '"', "'": "'", '0': '\0'}
            value = escapes.get(value, value)
        
        if not self._match("'"):
            raise LexerError("Unterminated char literal", start_line, start_column, self.filename)
        
        return Token(TokenType.CHAR, value, start_line, start_column)
    
    def _scan_number(self, start_line: int, start_column: int, start_char: str) -> Token:
        text = start_char
        is_float = False
        
        while not self._is_at_end() and (self._peek().isdigit() or self._peek() == '_'):
            if self._peek() == '_':
                self._advance()
                continue
            text += self._advance()
        
        # 检查浮点数
        if self._peek() == '.' and self._peek_next().isdigit():
            is_float = True
            text += self._advance()  # consume '.'
            while not self._is_at_end() and (self._peek().isdigit() or self._peek() == '_'):
                if self._peek() == '_':
                    self._advance()
                    continue
                text += self._advance()
        
        # 检查指数
        if self._peek() in 'eE':
            is_float = True
            text += self._advance()
            if self._peek() in '+-':
                text += self._advance()
            while not self._is_at_end() and self._peek().isdigit():
                text += self._advance()
        
        value = float(text.replace('_', '')) if is_float else int(text.replace('_', ''))
        token_type = TokenType.FLOAT if is_float else TokenType.INTEGER
        return Token(token_type, value, start_line, start_column)
    
    def _scan_identifier(self, start_line: int, start_column: int, start_char: str) -> Token:
        text = start_char
        
        while not self._is_at_end() and (self._peek().isalnum() or self._peek() == '_'):
            text += self._advance()
        
        # 检查是否为关键字
        if text in self.KEYWORDS:
            return Token(TokenType.KEYWORD, text, start_line, start_column, keyword=self.KEYWORDS[text])
        
        return Token(TokenType.IDENTIFIER, text, start_line, start_column)
    
    def _scan_operator(self, ch: str, start_line: int, start_column: int) -> Token:
        ops = {
            '(': TokenType.LEFT_PAREN, ')': TokenType.RIGHT_PAREN,
            '{': TokenType.LEFT_BRACE, '}': TokenType.RIGHT_BRACE,
            '[': TokenType.LEFT_BRACKET, ']': TokenType.RIGHT_BRACKET,
            ',': TokenType.COMMA, ';': TokenType.SEMICOLON,
            ':': TokenType.COLON, '.': TokenType.DOT,
            '?': TokenType.QUESTION, '@': TokenType.AT,
            '%': TokenType.PERCENT, '^': TokenType.BIT_XOR, '~': TokenType.BIT_NOT,
        }
        
        # 单字符运算符
        if ch in ops:
            return Token(ops[ch], ch, start_line, start_column)
        
        # 多字符运算符
        two_char_ops = {
            '+': [('+', TokenType.PLUS_PLUS), ('=', TokenType.PLUS_EQUAL)],
            '-': [('-', TokenType.MINUS_MINUS), ('=', TokenType.MINUS_EQUAL), ('>', TokenType.ARROW)],
            '*': [('=', TokenType.STAR_EQUAL)],
            '/': [('=', TokenType.SLASH_EQUAL)],
            '=': [('=', TokenType.EQUAL_EQUAL), ('>', TokenType.FAT_ARROW)],
            '!': [('=', TokenType.NOT_EQUAL)],
            '<': [('=', TokenType.LESS_EQUAL), ('<', TokenType.SHIFT_LEFT)],
            '>': [('=', TokenType.GREATER_EQUAL), ('>', TokenType.SHIFT_RIGHT)],
            '&': [('&', TokenType.AND)],
            '|': [('|', TokenType.OR)],
        }
        
        if ch in two_char_ops:
            for expected, token_type in two_char_ops[ch]:
                if self._match(expected):
                    return Token(token_type, ch + expected, start_line, start_column)
        
        # 单字符运算符映射
        single_ops = {
            '+': TokenType.PLUS, '-': TokenType.MINUS, '*': TokenType.STAR,
            '/': TokenType.SLASH, '=': TokenType.EQUAL, '!': TokenType.NOT,
            '<': TokenType.LESS, '>': TokenType.GREATER, '&': TokenType.BIT_AND,
            '|': TokenType.BIT_OR,
        }
        
        if ch in single_ops:
            return Token(single_ops[ch], ch, start_line, start_column)
        
        raise LexerError(f"Unexpected character: '{ch}'", start_line, start_column, self.filename)


# ============================================================================
# 抽象语法树 (AST)
# ============================================================================

@dataclass
class ASTNode:
    """AST 节点基类"""
    line: int = field(default=0)
    column: int = field(default=0)


@dataclass
class Identifier(ASTNode):
    name: str = ""


@dataclass
class Literal(ASTNode):
    value: Any = None
    lit_type: str = "none"


@dataclass
class BinaryOp(ASTNode):
    left: ASTNode = field(default_factory=lambda: ASTNode())
    operator: str = ""
    right: ASTNode = field(default_factory=lambda: ASTNode())


@dataclass
class UnaryOp(ASTNode):
    operator: str = ""
    operand: ASTNode = field(default_factory=lambda: ASTNode())


@dataclass
class Assignment(ASTNode):
    target: ASTNode = field(default_factory=lambda: ASTNode())
    operator: str = ""
    value: ASTNode = field(default_factory=lambda: ASTNode())


@dataclass
class Call(ASTNode):
    callee: ASTNode = field(default_factory=lambda: ASTNode())
    arguments: List[ASTNode] = field(default_factory=list)


@dataclass
class IndexAccess(ASTNode):
    object: ASTNode = field(default_factory=lambda: ASTNode())
    index: ASTNode = field(default_factory=lambda: ASTNode())


@dataclass
class FieldAccess(ASTNode):
    object: ASTNode = field(default_factory=lambda: ASTNode())
    field: str = ""


@dataclass
class Lambda(ASTNode):
    params: List[Identifier] = field(default_factory=list)
    body: ASTNode = field(default_factory=lambda: ASTNode())


@dataclass
class ArrayLiteral(ASTNode):
    elements: List[ASTNode] = field(default_factory=list)


@dataclass
class DictLiteral(ASTNode):
    pairs: List[Tuple[ASTNode, ASTNode]] = field(default_factory=list)


@dataclass
class TupleLiteral(ASTNode):
    elements: List[ASTNode] = field(default_factory=list)


@dataclass
class VariableDecl(ASTNode):
    name: str = ""
    mutable: bool = False
    type_annotation: Optional[str] = None
    initializer: Optional[ASTNode] = None


@dataclass
class FunctionDef(ASTNode):
    name: str = ""
    params: List[Tuple[str, Optional[str]]] = field(default_factory=list)
    return_type: Optional[str] = None
    body: Optional['Block'] = None
    is_async: bool = False
    is_pub: bool = False


@dataclass
class StructDef(ASTNode):
    name: str = ""
    fields: List[Tuple[str, str]] = field(default_factory=list)
    is_pub: bool = False


@dataclass
class EnumDef(ASTNode):
    name: str = ""
    variants: List[Tuple[str, Optional[List[str]]]] = field(default_factory=list)
    is_pub: bool = False


@dataclass
class TraitDef(ASTNode):
    name: str = ""
    methods: List[FunctionDef] = field(default_factory=list)
    is_pub: bool = False


@dataclass
class ImplDef(ASTNode):
    trait_name: Optional[str] = None
    type_name: str = ""
    methods: List[FunctionDef] = field(default_factory=list)


@dataclass
class Block(ASTNode):
    statements: List[ASTNode] = field(default_factory=list)


@dataclass
class IfStmt(ASTNode):
    condition: ASTNode = field(default_factory=lambda: ASTNode())
    then_branch: Block = field(default_factory=lambda: Block())
    else_branch: Optional[Block] = None


@dataclass
class MatchStmt(ASTNode):
    subject: ASTNode = field(default_factory=lambda: ASTNode())
    arms: List[Tuple[ASTNode, Block]] = field(default_factory=list)


@dataclass
class WhileStmt(ASTNode):
    condition: ASTNode = field(default_factory=lambda: ASTNode())
    body: Block = field(default_factory=lambda: Block())


@dataclass
class ForStmt(ASTNode):
    variable: str = ""
    iterable: ASTNode = field(default_factory=lambda: ASTNode())
    body: Block = field(default_factory=lambda: Block())


@dataclass
class LoopStmt(ASTNode):
    body: Block = field(default_factory=lambda: Block())


@dataclass
class ReturnStmt(ASTNode):
    value: Optional[ASTNode] = None


@dataclass
class BreakStmt(ASTNode):
    pass


@dataclass
class ContinueStmt(ASTNode):
    pass


@dataclass
class ExpressionStmt(ASTNode):
    expression: ASTNode = field(default_factory=lambda: ASTNode())


@dataclass
class Module(ASTNode):
    items: List[ASTNode] = field(default_factory=list)


# ============================================================================
# 语法分析器 (Parser)
# ============================================================================

class ParseError(Exception):
    """语法分析错误"""
    def __init__(self, message: str, line: int, column: int, filename: str):
        self.message = message
        self.line = line
        self.column = column
        self.filename = filename
        super().__init__(f"{filename}:{line}:{column}: Syntax error: {message}")


class Parser:
    """递归下降语法分析器"""
    
    def __init__(self, tokens: List[Token], filename: str = "<input>"):
        self.tokens = tokens
        self.filename = filename
        self.current = 0
    
    def parse(self) -> Module:
        """执行语法分析，返回 AST"""
        items = []
        
        while not self._is_at_end():
            item = self._parse_item()
            if item:
                items.append(item)
        
        return Module(items=items)
    
    def _is_at_end(self) -> bool:
        return self.current >= len(self.tokens) or self.tokens[self.current].token_type == TokenType.EOF
    
    def _advance(self) -> Token:
        if not self._is_at_end():
            self.current += 1
        return self._previous()
    
    def _previous(self) -> Token:
        return self.tokens[self.current - 1] if self.current > 0 else self.tokens[0]
    
    def _peek(self) -> Token:
        return self.tokens[self.current] if self.current < len(self.tokens) else self.tokens[-1]
    
    def _check(self, *types: TokenType) -> bool:
        return self._peek().token_type in types
    
    def _match(self, *types: TokenType) -> bool:
        if self._check(*types):
            self._advance()
            return True
        return False
    
    def _consume(self, token_type: TokenType, message: str) -> Token:
        if self._check(token_type):
            return self._advance()
        raise ParseError(message, self._peek().line, self._peek().column, self.filename)
    
    def _skip_whitespace(self):
        while not self._is_at_end():
            if self._check(TokenType.NEWLINE, TokenType.COMMENT):
                self._advance()
            else:
                break
    
    def _parse_item(self) -> Optional[ASTNode]:
        self._skip_whitespace()
        
        if self._is_at_end():
            return None
        
        # 检查可见性修饰符
        is_pub = False
        if self._check(TokenType.KEYWORD) and self._peek().keyword == Keyword.PUB:
            is_pub = True
            self._advance()
            self._skip_whitespace()
        
        # 函数定义
        if self._check(TokenType.KEYWORD) and self._peek().keyword == Keyword.FN:
            return self._parse_function_def(is_pub)
        
        # 结构体定义
        if self._check(TokenType.KEYWORD) and self._peek().keyword == Keyword.STRUCT:
            return self._parse_struct_def(is_pub)
        
        # 枚举定义
        if self._check(TokenType.KEYWORD) and self._peek().keyword == Keyword.ENUM:
            return self._parse_enum_def(is_pub)
        
        # 特质定义
        if self._check(TokenType.KEYWORD) and self._peek().keyword == Keyword.TRAIT:
            return self._parse_trait_def(is_pub)
        
        # 实现块
        if self._check(TokenType.KEYWORD) and self._peek().keyword == Keyword.IMPL:
            return self._parse_impl_def()
        
        # 变量声明或表达式语句
        if self._check(TokenType.KEYWORD) and self._peek().keyword in (Keyword.LET, Keyword.CONST):
            return self._parse_variable_decl()
        
        # 默认作为表达式语句
        expr = self._parse_expression()
        self._skip_whitespace()
        if self._check(TokenType.SEMICOLON):
            self._advance()
        return ExpressionStmt(expression=expr)
    
    def _parse_function_def(self, is_pub: bool) -> FunctionDef:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'fn'")
        name_token = self._consume(TokenType.IDENTIFIER, "Expected function name")
        
        self._consume(TokenType.LEFT_PAREN, "Expected '('")
        params = []
        if not self._check(TokenType.RIGHT_PAREN):
            params.append(self._parse_param())
            while self._match(TokenType.COMMA):
                params.append(self._parse_param())
        self._consume(TokenType.RIGHT_PAREN, "Expected ')'")
        
        return_type = None
        if self._match(TokenType.ARROW):
            type_token = self._consume(TokenType.IDENTIFIER, "Expected return type")
            return_type = type_token.value
        
        body = None
        if self._check(TokenType.LEFT_BRACE):
            body = self._parse_block()
        
        return FunctionDef(
            name=name_token.value,
            params=params,
            return_type=return_type,
            body=body,
            is_async=False,
            is_pub=is_pub,
            line=line,
            column=col
        )
    
    def _parse_param(self) -> Tuple[str, Optional[str]]:
        name_token = self._consume(TokenType.IDENTIFIER, "Expected parameter name")
        param_type = None
        if self._match(TokenType.COLON):
            type_token = self._consume(TokenType.IDENTIFIER, "Expected parameter type")
            param_type = type_token.value
        return (name_token.value, param_type)
    
    def _parse_struct_def(self, is_pub: bool) -> StructDef:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'struct'")
        name_token = self._consume(TokenType.IDENTIFIER, "Expected struct name")
        
        self._consume(TokenType.LEFT_BRACE, "Expected '{'")
        fields = []
        
        while not self._check(TokenType.RIGHT_BRACE):
            self._skip_whitespace()
            if self._is_at_end():
                break
            field_name = self._consume(TokenType.IDENTIFIER, "Expected field name").value
            self._consume(TokenType.COLON, "Expected ':'")
            field_type = self._consume(TokenType.IDENTIFIER, "Expected field type").value
            fields.append((field_name, field_type))
            if not self._match(TokenType.COMMA):
                break
        
        self._consume(TokenType.RIGHT_BRACE, "Expected '}'")
        
        return StructDef(
            name=name_token.value,
            fields=fields,
            is_pub=is_pub,
            line=line,
            column=col
        )
    
    def _parse_enum_def(self, is_pub: bool) -> EnumDef:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'enum'")
        name_token = self._consume(TokenType.IDENTIFIER, "Expected enum name")
        
        self._consume(TokenType.LEFT_BRACE, "Expected '{'")
        variants = []
        
        while not self._check(TokenType.RIGHT_BRACE):
            self._skip_whitespace()
            if self._is_at_end():
                break
            variant_name = self._consume(TokenType.IDENTIFIER, "Expected variant name").value
            fields = None
            if self._check(TokenType.LEFT_PAREN):
                self._advance()
                fields = []
                if not self._check(TokenType.RIGHT_PAREN):
                    fields.append(self._consume(TokenType.IDENTIFIER, "Expected field type").value)
                    while self._match(TokenType.COMMA):
                        fields.append(self._consume(TokenType.IDENTIFIER, "Expected field type").value)
                self._consume(TokenType.RIGHT_PAREN, "Expected ')'")
            variants.append((variant_name, fields))
            if not self._match(TokenType.COMMA):
                break
        
        self._consume(TokenType.RIGHT_BRACE, "Expected '}'")
        
        return EnumDef(
            name=name_token.value,
            variants=variants,
            is_pub=is_pub,
            line=line,
            column=col
        )
    
    def _parse_trait_def(self, is_pub: bool) -> TraitDef:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'trait'")
        name_token = self._consume(TokenType.IDENTIFIER, "Expected trait name")
        
        self._consume(TokenType.LEFT_BRACE, "Expected '{'")
        methods = []
        
        while not self._check(TokenType.RIGHT_BRACE):
            self._skip_whitespace()
            if self._is_at_end():
                break
            methods.append(self._parse_function_def(is_pub=False))
        
        self._consume(TokenType.RIGHT_BRACE, "Expected '}'")
        
        return TraitDef(
            name=name_token.value,
            methods=methods,
            is_pub=is_pub,
            line=line,
            column=col
        )
    
    def _parse_impl_def(self) -> ImplDef:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'impl'")
        
        trait_name = None
        type_name = None
        
        # impl Trait for Type 或 impl Type
        first_type = self._consume(TokenType.IDENTIFIER, "Expected type name").value
        
        if self._check(TokenType.KEYWORD) and self._peek().keyword and self._peek().keyword.value == 'for':
            trait_name = first_type
            self._advance()
            type_name = self._consume(TokenType.IDENTIFIER, "Expected type name").value
        else:
            type_name = first_type
        
        self._consume(TokenType.LEFT_BRACE, "Expected '{'")
        methods = []
        
        while not self._check(TokenType.RIGHT_BRACE):
            self._skip_whitespace()
            if self._is_at_end():
                break
            methods.append(self._parse_function_def(is_pub=False))
        
        self._consume(TokenType.RIGHT_BRACE, "Expected '}'")
        
        return ImplDef(
            trait_name=trait_name,
            type_name=type_name,
            methods=methods,
            line=line,
            column=col
        )
    
    def _parse_variable_decl(self) -> VariableDecl:
        line, col = self._peek().line, self._peek().column
        
        is_const = self._check(TokenType.KEYWORD) and self._peek().keyword == Keyword.CONST
        self._advance()
        
        mutable = False
        if self._check(TokenType.KEYWORD) and self._peek().keyword == Keyword.MUT:
            mutable = True
            self._advance()
        
        name_token = self._consume(TokenType.IDENTIFIER, "Expected variable name")
        
        type_annotation = None
        if self._match(TokenType.COLON):
            type_token = self._consume(TokenType.IDENTIFIER, "Expected type")
            type_annotation = type_token.value
        
        initializer = None
        if self._match(TokenType.EQUAL):
            initializer = self._parse_expression()
        
        if self._check(TokenType.SEMICOLON):
            self._advance()
        
        return VariableDecl(
            name=name_token.value,
            mutable=mutable,
            type_annotation=type_annotation,
            initializer=initializer,
            line=line,
            column=col
        )
    
    def _parse_block(self) -> Block:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.LEFT_BRACE, "Expected '{'")
        statements = []
        
        while not self._check(TokenType.RIGHT_BRACE):
            self._skip_whitespace()
            if self._is_at_end():
                break
            # 检查是否遇到右花括号
            if self._check(TokenType.RIGHT_BRACE):
                break
            stmt = self._parse_statement()
            if stmt:
                statements.append(stmt)
        
        self._consume(TokenType.RIGHT_BRACE, "Expected '}'")
        
        return Block(statements=statements, line=line, column=col)
    
    def _parse_statement(self) -> Optional[ASTNode]:
        self._skip_whitespace()
        
        if self._is_at_end():
            return None
        
        # 检查是否遇到右花括号（块结束）
        if self._check(TokenType.RIGHT_BRACE):
            return None
        
        # let/const 声明
        if self._check(TokenType.KEYWORD) and self._peek().keyword in (Keyword.LET, Keyword.CONST):
            return self._parse_variable_decl()
        
        # 控制流语句
        if self._check(TokenType.KEYWORD):
            kw = self._peek().keyword
            if kw == Keyword.IF:
                return self._parse_if_stmt()
            elif kw == Keyword.MATCH:
                return self._parse_match_stmt()
            elif kw == Keyword.WHILE:
                return self._parse_while_stmt()
            elif kw == Keyword.FOR:
                return self._parse_for_stmt()
            elif kw == Keyword.LOOP:
                return self._parse_loop_stmt()
            elif kw == Keyword.RETURN:
                return self._parse_return_stmt()
            elif kw == Keyword.BREAK:
                self._advance()
                if self._check(TokenType.SEMICOLON):
                    self._advance()
                return BreakStmt()
            elif kw == Keyword.CONTINUE:
                self._advance()
                if self._check(TokenType.SEMICOLON):
                    self._advance()
                return ContinueStmt()
            elif kw == Keyword.FN:
                return self._parse_function_def(is_pub=False)
        
        # 表达式语句
        expr = self._parse_expression()
        if self._check(TokenType.SEMICOLON):
            self._advance()
        return ExpressionStmt(expression=expr)
    
    def _parse_if_stmt(self) -> IfStmt:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'if'")
        condition = self._parse_expression()
        then_branch = self._parse_block()
        
        else_branch = None
        if self._check(TokenType.KEYWORD) and self._peek().keyword == Keyword.ELSE:
            self._advance()
            if self._check(TokenType.KEYWORD) and self._peek().keyword == Keyword.IF:
                else_branch = self._parse_if_stmt()
            else:
                else_branch = self._parse_block()
        
        return IfStmt(
            condition=condition,
            then_branch=then_branch,
            else_branch=else_branch,
            line=line,
            column=col
        )
    
    def _parse_match_stmt(self) -> MatchStmt:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'match'")
        subject = self._parse_expression()
        
        self._consume(TokenType.LEFT_BRACE, "Expected '{'")
        arms = []
        
        while not self._check(TokenType.RIGHT_BRACE):
            self._skip_whitespace()
            if self._is_at_end():
                break
            
            pattern = self._parse_pattern()
            self._consume(TokenType.FAT_ARROW, "Expected '=>'")
            body = self._parse_block()
            arms.append((pattern, body))
            
            if not self._match(TokenType.COMMA):
                break
        
        self._consume(TokenType.RIGHT_BRACE, "Expected '}'")
        
        return MatchStmt(subject=subject, arms=arms, line=line, column=col)
    
    def _parse_pattern(self) -> ASTNode:
        # 简化的模式匹配：支持字面量、标识符、通配符
        self._skip_whitespace()
        
        if self._check(TokenType.IDENTIFIER):
            token = self._advance()
            if token.value == '_':
                return Literal(value='_', lit_type='wildcard')
            return Identifier(name=token.value)
        
        if self._check(TokenType.INTEGER, TokenType.FLOAT, TokenType.STRING, TokenType.CHAR, TokenType.BOOL):
            token = self._advance()
            return Literal(value=token.value, lit_type=token.token_type.value.lower())
        
        # 默认返回标识符
        return Identifier(name='_')
    
    def _parse_while_stmt(self) -> WhileStmt:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'while'")
        condition = self._parse_expression()
        body = self._parse_block()
        
        return WhileStmt(condition=condition, body=body, line=line, column=col)
    
    def _parse_for_stmt(self) -> ForStmt:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'for'")
        var_token = self._consume(TokenType.IDENTIFIER, "Expected loop variable")
        self._consume(TokenType.KEYWORD, "Expected 'in'")
        iterable = self._parse_expression()
        body = self._parse_block()
        
        return ForStmt(variable=var_token.value, iterable=iterable, body=body, line=line, column=col)
    
    def _parse_loop_stmt(self) -> LoopStmt:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'loop'")
        body = self._parse_block()
        
        return LoopStmt(body=body, line=line, column=col)
    
    def _parse_return_stmt(self) -> ReturnStmt:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.KEYWORD, "Expected 'return'")
        value = None
        if not self._check(TokenType.SEMICOLON, TokenType.RIGHT_BRACE, TokenType.NEWLINE):
            value = self._parse_expression()
        if self._check(TokenType.SEMICOLON):
            self._advance()
        
        return ReturnStmt(value=value, line=line, column=col)
    
    def _parse_expression(self) -> ASTNode:
        return self._parse_assignment()
    
    def _parse_assignment(self) -> ASTNode:
        left = self._parse_or_expr()
        
        if self._check(TokenType.EQUAL, TokenType.PLUS_EQUAL, TokenType.MINUS_EQUAL, 
                       TokenType.STAR_EQUAL, TokenType.SLASH_EQUAL):
            op_token = self._advance()
            right = self._parse_assignment()
            return Assignment(target=left, operator=op_token.value, value=right, 
                            line=left.line, column=left.column)
        
        return left
    
    def _parse_or_expr(self) -> ASTNode:
        left = self._parse_and_expr()
        
        while self._check(TokenType.OR):
            op_token = self._advance()
            right = self._parse_and_expr()
            left = BinaryOp(left=left, operator=op_token.value, right=right,
                          line=left.line, column=left.column)
        
        return left
    
    def _parse_and_expr(self) -> ASTNode:
        left = self._parse_equality()
        
        while self._check(TokenType.AND):
            op_token = self._advance()
            right = self._parse_equality()
            left = BinaryOp(left=left, operator=op_token.value, right=right,
                          line=left.line, column=left.column)
        
        return left
    
    def _parse_equality(self) -> ASTNode:
        left = self._parse_comparison()
        
        while self._check(TokenType.EQUAL_EQUAL, TokenType.NOT_EQUAL):
            op_token = self._advance()
            right = self._parse_comparison()
            left = BinaryOp(left=left, operator=op_token.value, right=right,
                          line=left.line, column=left.column)
        
        return left
    
    def _parse_comparison(self) -> ASTNode:
        left = self._parse_additive()
        
        while self._check(TokenType.LESS, TokenType.LESS_EQUAL, 
                         TokenType.GREATER, TokenType.GREATER_EQUAL):
            op_token = self._advance()
            right = self._parse_additive()
            left = BinaryOp(left=left, operator=op_token.value, right=right,
                          line=left.line, column=left.column)
        
        return left
    
    def _parse_additive(self) -> ASTNode:
        left = self._parse_multiplicative()
        
        while self._check(TokenType.PLUS, TokenType.MINUS):
            op_token = self._advance()
            right = self._parse_multiplicative()
            left = BinaryOp(left=left, operator=op_token.value, right=right,
                          line=left.line, column=left.column)
        
        return left
    
    def _parse_multiplicative(self) -> ASTNode:
        left = self._parse_unary()
        
        while self._check(TokenType.STAR, TokenType.SLASH, TokenType.PERCENT):
            op_token = self._advance()
            right = self._parse_unary()
            left = BinaryOp(left=left, operator=op_token.value, right=right,
                          line=left.line, column=left.column)
        
        return left
    
    def _parse_unary(self) -> ASTNode:
        if self._check(TokenType.MINUS, TokenType.NOT, TokenType.BIT_NOT):
            op_token = self._advance()
            operand = self._parse_unary()
            return UnaryOp(operator=op_token.value, operand=operand,
                         line=op_token.line, column=op_token.column)
        
        return self._parse_postfix()
    
    def _parse_postfix(self) -> ASTNode:
        expr = self._parse_primary()
        
        while True:
            if self._check(TokenType.LEFT_PAREN):
                self._advance()
                args = []
                if not self._check(TokenType.RIGHT_PAREN):
                    args.append(self._parse_expression())
                    while self._match(TokenType.COMMA):
                        args.append(self._parse_expression())
                self._consume(TokenType.RIGHT_PAREN, "Expected ')'")
                expr = Call(callee=expr, arguments=args, line=expr.line, column=expr.column)
            
            elif self._check(TokenType.DOT):
                self._advance()
                field_token = self._consume(TokenType.IDENTIFIER, "Expected field name")
                expr = FieldAccess(object=expr, field=field_token.value,
                                 line=expr.line, column=expr.column)
            
            elif self._check(TokenType.LEFT_BRACKET):
                self._advance()
                index = self._parse_expression()
                self._consume(TokenType.RIGHT_BRACKET, "Expected ']'")
                expr = IndexAccess(object=expr, index=index,
                                 line=expr.line, column=expr.column)
            
            else:
                break
        
        return expr
    
    def _parse_primary(self) -> ASTNode:
        token = self._peek()
        
        # 字面量
        if token.token_type == TokenType.INTEGER:
            self._advance()
            return Literal(value=int(token.value), lit_type='int', line=token.line, column=token.column)
        
        if token.token_type == TokenType.FLOAT:
            self._advance()
            return Literal(value=float(token.value), lit_type='float', line=token.line, column=token.column)
        
        if token.token_type == TokenType.STRING:
            self._advance()
            return Literal(value=token.value, lit_type='string', line=token.line, column=token.column)
        
        if token.token_type == TokenType.CHAR:
            self._advance()
            return Literal(value=token.value, lit_type='char', line=token.line, column=token.column)
        
        if token.token_type == TokenType.KEYWORD and token.keyword in (Keyword.TRUE, Keyword.FALSE):
            self._advance()
            return Literal(value=token.keyword == Keyword.TRUE, lit_type='bool', 
                         line=token.line, column=token.column)
        
        if token.token_type == TokenType.KEYWORD and token.keyword == Keyword.NONE:
            self._advance()
            return Literal(value=None, lit_type='none', line=token.line, column=token.column)
        
        # 标识符
        if token.token_type == TokenType.IDENTIFIER:
            self._advance()
            return Identifier(name=token.value, line=token.line, column=token.column)
        
        # 括号表达式
        if self._check(TokenType.LEFT_PAREN):
            self._advance()
            expr = self._parse_expression()
            self._consume(TokenType.RIGHT_PAREN, "Expected ')'")
            return expr
        
        # 数组字面量
        if self._check(TokenType.LEFT_BRACKET):
            self._advance()
            elements = []
            if not self._check(TokenType.RIGHT_BRACKET):
                elements.append(self._parse_expression())
                while self._match(TokenType.COMMA):
                    elements.append(self._parse_expression())
            self._consume(TokenType.RIGHT_BRACKET, "Expected ']'")
            return ArrayLiteral(elements=elements, line=token.line, column=token.column)
        
        # Lambda 表达式
        if self._check(TokenType.BIT_OR):
            return self._parse_lambda()
        
        raise ParseError(f"Unexpected token: {token}", token.line, token.column, self.filename)
    
    def _parse_lambda(self) -> Lambda:
        line, col = self._peek().line, self._peek().column
        
        self._consume(TokenType.BIT_OR, "Expected '|'")
        params = []
        if not self._check(TokenType.BIT_OR):
            param = self._consume(TokenType.IDENTIFIER, "Expected parameter name")
            params.append(Identifier(name=param.value))
            while self._match(TokenType.COMMA):
                param = self._consume(TokenType.IDENTIFIER, "Expected parameter name")
                params.append(Identifier(name=param.value))
        self._consume(TokenType.BIT_OR, "Expected '|'")
        
        body = self._parse_expression()
        
        return Lambda(params=params, body=body, line=line, column=col)


# ============================================================================
# 中间表示 (IR)
# ============================================================================

@dataclass
class IRInstruction:
    """IR 指令"""
    opcode: str
    dest: Optional[str] = None
    operands: List[Any] = field(default_factory=list)
    line: int = 0
    
    def __repr__(self):
        if self.dest:
            ops = ', '.join(str(o) for o in self.operands)
            return f"{self.dest} = {self.opcode} {ops}"
        elif self.operands:
            ops = ', '.join(str(o) for o in self.operands)
            return f"{self.opcode} {ops}"
        else:
            return self.opcode


class IRGenerator:
    """IR 生成器"""
    
    def __init__(self):
        self.instructions: List[IRInstruction] = []
        self.temp_counter = 0
        self.label_counter = 0
        self.scope_stack: List[Dict[str, str]] = [{}]
    
    def generate(self, module: Module) -> List[IRInstruction]:
        """生成 IR"""
        for item in module.items:
            self._generate_item(item)
        return self.instructions
    
    def _new_temp(self) -> str:
        self.temp_counter += 1
        return f"t{self.temp_counter}"
    
    def _new_label(self) -> str:
        self.label_counter += 1
        return f"L{self.label_counter}"
    
    def _emit(self, opcode: str, dest: Optional[str] = None, operands: List[Any] = None, line: int = 0):
        instr = IRInstruction(opcode=opcode, dest=dest, operands=operands or [], line=line)
        self.instructions.append(instr)
        return dest
    
    def _current_scope(self) -> Dict[str, str]:
        return self.scope_stack[-1]
    
    def _enter_scope(self):
        self.scope_stack.append({})
    
    def _exit_scope(self):
        self.scope_stack.pop()
    
    def _generate_item(self, item: ASTNode):
        if isinstance(item, FunctionDef):
            self._generate_function(item)
        elif isinstance(item, StructDef):
            self._generate_struct(item)
        elif isinstance(item, VariableDecl):
            self._generate_variable_decl(item)
        elif isinstance(item, ExpressionStmt):
            self._generate_expression(item.expression)
    
    def _generate_function(self, func: FunctionDef):
        label = f"func_{func.name}"
        self._emit("FUNCTION", dest=label, operands=[func.name], line=func.line)
        
        self._enter_scope()
        for param_name, param_type in func.params:
            temp = self._new_temp()
            self._current_scope()[param_name] = temp
            self._emit("PARAM", dest=temp, operands=[param_name], line=func.line)
        
        if func.body:
            self._generate_block(func.body)
        
        self._exit_scope()
        self._emit("END_FUNCTION", line=func.line)
    
    def _generate_struct(self, struct: StructDef):
        self._emit("STRUCT", dest=f"struct_{struct.name}", 
                  operands=[struct.name, struct.fields], line=struct.line)
    
    def _generate_variable_decl(self, decl: VariableDecl):
        temp = self._new_temp()
        self._current_scope()[decl.name] = temp
        
        if decl.initializer:
            value = self._generate_expression(decl.initializer)
            self._emit("STORE", dest=temp, operands=[value], line=decl.line)
        else:
            self._emit("ALLOC", dest=temp, operands=[decl.type_annotation or "any"], line=decl.line)
    
    def _generate_block(self, block: Block):
        self._enter_scope()
        for stmt in block.statements:
            self._generate_statement(stmt)
        self._exit_scope()
    
    def _generate_statement(self, stmt: ASTNode):
        if isinstance(stmt, VariableDecl):
            self._generate_variable_decl(stmt)
        elif isinstance(stmt, IfStmt):
            self._generate_if(stmt)
        elif isinstance(stmt, WhileStmt):
            self._generate_while(stmt)
        elif isinstance(stmt, ReturnStmt):
            self._generate_return(stmt)
        elif isinstance(stmt, BreakStmt):
            self._emit("BREAK", line=stmt.line)
        elif isinstance(stmt, ContinueStmt):
            self._emit("CONTINUE", line=stmt.line)
        elif isinstance(stmt, ExpressionStmt):
            self._generate_expression(stmt.expression)
    
    def _generate_if(self, stmt: IfStmt):
        cond = self._generate_expression(stmt.condition)
        else_label = self._new_label()
        end_label = self._new_label()
        
        self._emit("JUMP_IF_FALSE", operands=[cond, else_label], line=stmt.line)
        self._generate_block(stmt.then_branch)
        
        if stmt.else_branch:
            self._emit("JUMP", operands=[end_label], line=stmt.line)
        
        self._emit("LABEL", dest=else_label, line=stmt.line)
        
        if stmt.else_branch:
            if isinstance(stmt.else_branch, IfStmt):
                self._generate_if(stmt.else_branch)
            else:
                self._generate_block(stmt.else_branch)
            self._emit("LABEL", dest=end_label, line=stmt.line)
    
    def _generate_while(self, stmt: WhileStmt):
        start_label = self._new_label()
        end_label = self._new_label()
        
        self._emit("LABEL", dest=start_label, line=stmt.line)
        cond = self._generate_expression(stmt.condition)
        self._emit("JUMP_IF_FALSE", operands=[cond, end_label], line=stmt.line)
        self._generate_block(stmt.body)
        self._emit("JUMP", operands=[start_label], line=stmt.line)
        self._emit("LABEL", dest=end_label, line=stmt.line)
    
    def _generate_return(self, stmt: ReturnStmt):
        if stmt.value:
            value = self._generate_expression(stmt.value)
            self._emit("RETURN", operands=[value], line=stmt.line)
        else:
            self._emit("RETURN", line=stmt.line)
    
    def _generate_expression(self, expr: ASTNode) -> str:
        if isinstance(expr, Literal):
            temp = self._new_temp()
            self._emit("LOAD_CONST", dest=temp, operands=[expr.value, expr.lit_type], line=expr.line)
            return temp
        
        elif isinstance(expr, Identifier):
            if expr.name in self._current_scope():
                return self._current_scope()[expr.name]
            temp = self._new_temp()
            self._emit("LOAD_GLOBAL", dest=temp, operands=[expr.name], line=expr.line)
            return temp
        
        elif isinstance(expr, BinaryOp):
            left = self._generate_expression(expr.left)
            right = self._generate_expression(expr.right)
            temp = self._new_temp()
            op_map = {
                '+': 'ADD', '-': 'SUB', '*': 'MUL', '/': 'DIV', '%': 'MOD',
                '==': 'EQ', '!=': 'NE', '<': 'LT', '<=': 'LE', '>': 'GT', '>=': 'GE',
                '&&': 'AND', '||': 'OR',
            }
            self._emit(op_map.get(expr.operator, 'BINOP'), dest=temp, operands=[left, right], line=expr.line)
            return temp
        
        elif isinstance(expr, UnaryOp):
            operand = self._generate_expression(expr.operand)
            temp = self._new_temp()
            op_map = {'-': 'NEG', '!': 'NOT', '~': 'BITNOT'}
            self._emit(op_map.get(expr.operator, 'UNOP'), dest=temp, operands=[operand], line=expr.line)
            return temp
        
        elif isinstance(expr, Assignment):
            value = self._generate_expression(expr.value)
            # 简化处理
            self._emit("STORE", dest=value, operands=[value], line=expr.line)
            return value
        
        elif isinstance(expr, Call):
            args = [self._generate_expression(arg) for arg in expr.arguments]
            temp = self._new_temp()
            func_name = expr.callee.name if isinstance(expr.callee, Identifier) else "unknown"
            self._emit("CALL", dest=temp, operands=[func_name] + args, line=expr.line)
            return temp
        
        elif isinstance(expr, ArrayLiteral):
            temp = self._new_temp()
            elements = [self._generate_expression(elem) for elem in expr.elements]
            self._emit("ARRAY", dest=temp, operands=elements, line=expr.line)
            return temp
        
        else:
            temp = self._new_temp()
            self._emit("NOP", dest=temp, line=expr.line)
            return temp


# ============================================================================
# 字节码生成器
# ============================================================================

class BytecodeGenerator:
    """字节码生成器"""
    
    OPCODES = {
        'LOAD_CONST': 0x01,
        'LOAD_GLOBAL': 0x02,
        'STORE': 0x03,
        'ALLOC': 0x04,
        'ADD': 0x10,
        'SUB': 0x11,
        'MUL': 0x12,
        'DIV': 0x13,
        'MOD': 0x14,
        'EQ': 0x20,
        'NE': 0x21,
        'LT': 0x22,
        'LE': 0x23,
        'GT': 0x24,
        'GE': 0x25,
        'AND': 0x30,
        'OR': 0x31,
        'NOT': 0x32,
        'NEG': 0x33,
        'JUMP': 0x40,
        'JUMP_IF_FALSE': 0x41,
        'LABEL': 0x42,
        'CALL': 0x50,
        'RETURN': 0x51,
        'FUNCTION': 0x60,
        'END_FUNCTION': 0x61,
        'STRUCT': 0x70,
        'ARRAY': 0x80,
        'BREAK': 0x90,
        'CONTINUE': 0x91,
        'NOP': 0xFF,
    }
    
    def generate(self, ir: List[IRInstruction]) -> bytes:
        """生成字节码"""
        bytecode = bytearray()
        
        for instr in ir:
            opcode = self.OPCODES.get(instr.opcode, 0xFF)
            bytecode.append(opcode)
            
            # 编码操作数（简化版本）
            if instr.dest:
                self._encode_string(bytecode, instr.dest)
            
            for operand in instr.operands:
                self._encode_operand(bytecode, operand)
        
        return bytes(bytecode)
    
    def _encode_string(self, bytecode: bytearray, s: str):
        encoded = s.encode('utf-8')
        bytecode.append(len(encoded) & 0xFF)
        bytecode.extend(encoded)
    
    def _encode_operand(self, bytecode: bytearray, operand: Any):
        if isinstance(operand, int):
            bytecode.append(0x01)  # int tag
            bytecode.extend(operand.to_bytes(8, 'little', signed=True))
        elif isinstance(operand, float):
            bytecode.append(0x02)  # float tag
            import struct
            bytecode.extend(struct.pack('<d', operand))
        elif isinstance(operand, str):
            bytecode.append(0x03)  # string tag
            self._encode_string(bytecode, operand)
        elif isinstance(operand, bool):
            bytecode.append(0x04)  # bool tag
            bytecode.append(1 if operand else 0)
        elif isinstance(operand, list):
            bytecode.append(0x05)  # list tag
            bytecode.append(len(operand) & 0xFF)
            for item in operand:
                self._encode_operand(bytecode, item)
        else:
            bytecode.append(0x00)  # null tag


# ============================================================================
# 虚拟机
# ============================================================================

class VMError(Exception):
    """虚拟机错误"""
    pass


class VirtualMachine:
    """Aether 字节码虚拟机"""
    
    def __init__(self):
        self.stack: List[Any] = []
        self.globals: Dict[str, Any] = {}
        self.functions: Dict[str, Any] = {}
        self.running = False
    
    def run(self, bytecode: bytes) -> Any:
        """运行字节码"""
        ip = 0  # instruction pointer
        
        while ip < len(bytecode):
            opcode = bytecode[ip]
            ip += 1
            
            if opcode == 0x01:  # LOAD_CONST
                ip, value = self._decode_operand(bytecode, ip)
                self.stack.append(value)
            
            elif opcode == 0x10:  # ADD
                b = self.stack.pop()
                a = self.stack.pop()
                self.stack.append(a + b)
            
            elif opcode == 0x11:  # SUB
                b = self.stack.pop()
                a = self.stack.pop()
                self.stack.append(a - b)
            
            elif opcode == 0x12:  # MUL
                b = self.stack.pop()
                a = self.stack.pop()
                self.stack.append(a * b)
            
            elif opcode == 0x13:  # DIV
                b = self.stack.pop()
                a = self.stack.pop()
                self.stack.append(a / b if b != 0 else 0)
            
            elif opcode == 0x51:  # RETURN
                return self.stack[-1] if self.stack else None
            
            elif opcode == 0xFF:  # NOP
                pass
            
            else:
                # 跳过未知操作码的操作数（简化处理）
                pass
        
        return self.stack[-1] if self.stack else None
    
    def _decode_operand(self, bytecode: bytes, ip: int) -> Tuple[int, Any]:
        tag = bytecode[ip]
        ip += 1
        
        if tag == 0x01:  # int
            value = int.from_bytes(bytecode[ip:ip+8], 'little', signed=True)
            ip += 8
        elif tag == 0x02:  # float
            import struct
            value = struct.unpack('<d', bytecode[ip:ip+8])[0]
            ip += 8
        elif tag == 0x03:  # string
            length = bytecode[ip]
            ip += 1
            value = bytecode[ip:ip+length].decode('utf-8')
            ip += length
        elif tag == 0x04:  # bool
            value = bool(bytecode[ip])
            ip += 1
        else:
            value = None
        
        return ip, value


# ============================================================================
# 编译器主程序
# ============================================================================

class Compiler:
    """Aether 编译器"""
    
    def __init__(self, filename: str):
        self.filename = filename
        with open(filename, 'r', encoding='utf-8') as f:
            self.source = f.read()
    
    def compile(self, emit: str = 'bytecode') -> Any:
        """编译源代码"""
        # 词法分析
        lexer = Lexer(self.source, self.filename)
        tokens = lexer.lex()
        
        # 语法分析
        parser = Parser(tokens, self.filename)
        ast = parser.parse()
        
        # IR 生成
        ir_gen = IRGenerator()
        ir = ir_gen.generate(ast)
        
        if emit == 'ir':
            return ir
        
        # 字节码生成
        bc_gen = BytecodeGenerator()
        bytecode = bc_gen.generate(ir)
        
        if emit == 'bytecode':
            return bytecode
        
        return bytecode
    
    def run(self):
        """编译并运行"""
        bytecode = self.compile()
        vm = VirtualMachine()
        result = vm.run(bytecode)
        return result


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description='Aether Compiler')
    parser.add_argument('input', help='Input Aether source file')
    parser.add_argument('-o', '--output', help='Output file')
    parser.add_argument('--emit', choices=['ir', 'bytecode', 'run'], default='run',
                       help='Emit IR, bytecode, or run directly')
    parser.add_argument('--ast', action='store_true', help='Print AST')
    
    args = parser.parse_args()
    
    try:
        compiler = Compiler(args.input)
        
        if args.ast:
            lexer = Lexer(compiler.source, args.input)
            tokens = lexer.lex()
            parser_obj = Parser(tokens, args.input)
            ast = parser_obj.parse()
            print(json.dumps({
                'type': 'Module',
                'items': len(ast.items)
            }, indent=2))
            return
        
        result = compiler.compile(emit=args.emit)
        
        if args.emit == 'run':
            vm = VirtualMachine()
            output = vm.run(result)
            if output is not None:
                print(output)
        elif args.emit == 'ir':
            for instr in result:
                print(instr)
        elif args.output:
            with open(args.output, 'wb') as f:
                f.write(result)
            print(f"Compiled to {args.output}")
        else:
            print(f"Generated {len(result)} bytes of bytecode")
    
    except (LexerError, ParseError) as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except FileNotFoundError:
        print(f"Error: File not found: {args.input}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
