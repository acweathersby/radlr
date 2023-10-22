
/// ### `sherpa` Rust Parser
///
/// - **GENERATOR**: sherpa 1.0.0-beta2
/// - **SOURCE**: /home/work/projects/lib_sherpa/src/grammar/v2_0_0/grammar.sg
///
/// #### WARNING:
///
/// This is a generated file. Any changes to this file may be **overwritten
/// without notice**.
///
/// #### License:
/// Copyright (c) 2023 Anthony Weathersby
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to
/// deal in the Software without restriction, including without limitation the
/// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
/// sell copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in
/// all copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
/// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
/// IN THE SOFTWARE.

// 
use std::hash::Hash;
use sherpa_rust_runtime::{
  llvm_parser::*,
  types::{ast::*, Token, TokenRange}, deprecate::*,
};



pub trait ASTParse<T>{
  fn default_from(input:T) -> Result<Vec<ASTNode>, SherpaParseError>;
}

macro_rules! into_vec {
  ($fn_name:ident, $out_type: ty, $type:ident) => {
    pub fn $fn_name(self) -> Vec<$out_type> {
      if let ASTNode::$type(v) = self {
        v
      } else {
        vec![]
      }
    }
  };
}

macro_rules! to_numeric {
  ($fn_name:ident,  $Num:ty) => {
    fn $fn_name(&self) -> $Num {
      if self.is_numeric() || matches!(self, ASTNode::STRING(..) | ASTNode::TOKEN(..)) {
        match self {
          ASTNode::STRING(str) => str.parse::<i64>().unwrap_or(0) as $Num,
          ASTNode::TOKEN(tok) => tok.to_string().parse::<i64>().unwrap_or(0) as $Num,
          ASTNode::F64(val) => *val as $Num,
          ASTNode::F32(val) => *val as $Num,
          ASTNode::I64(val) => *val as $Num,
          ASTNode::I32(val) => *val as $Num,
          ASTNode::I16(val) => *val as $Num,
          ASTNode::U64(val) => *val as $Num,
          ASTNode::U32(val) => *val as $Num,
          ASTNode::U16(val) => *val as $Num,
          ASTNode::U8(val) => *val as $Num,
          ASTNode::BOOL(val) => (*val as usize) as $Num,
          _ => 0 as $Num,
        }
      } else {
        0 as $Num
      }
    }
  };
}

#[derive(Clone, Debug)]
#[repr(C, u32)]
pub enum ASTNode{
  NONE,
  NODES(Vec<ASTNode>),
  STRING(String),
  STRINGS(Vec<String>),
  F64(f64),
  F32(f32),
  I64(i64),
  I32(i32),
  I16(i16),
  I8(i8),
  U64(u64),
  U32(u32),
  U16(u16),
  U8(u8),
  BOOL(bool),
  F32Vec(Vec<f32>),
  F64Vec(Vec<f64>),
  I64Vec(Vec<i64>),
  I32Vec(Vec<i32>),
  I16Vec(Vec<i16>),
  I8Vec(Vec<i8>),
  U64Vec(Vec<u64>),
  U32Vec(Vec<u32>),
  U16Vec(Vec<u16>),
  U8Vec(Vec<u8>),
  TOKEN(Token),
  TOKENS(Vec<Token>),
  Indent(Box<Indent>),
  Index(Box<Index>),
  Dedent(Box<Dedent>),
  False(Box<False>),
  Assign(Box<Assign>),
  NewLine(Box<NewLine>),
  Match(Box<Match>),
  Literal(Box<Literal>),
  TypeCall(Box<TypeCall>),
  StringType(Box<StringType>),
  Param(Box<Param>),
  ObjType(Box<ObjType>),
  ExprTuple(Box<ExprTuple>),
  Call(Box<Call>),
  None(Box<None>),
  Type(Box<Type>),
  Tab(Box<Tab>),
  Expression(Box<Expression>),
  BreakPoint(Box<BreakPoint>),
  Text(Box<Text>),
  Funct(Box<Funct>),
  Space(Box<Space>),
  Prop(Box<Prop>),
  Mul(Box<Mul>),
  Div(Box<Div>),
  NotNone(Box<NotNone>),
  LiteralSpace(Box<LiteralSpace>),
  IntType(Box<IntType>),
  Pow(Box<Pow>),
  Add(Box<Add>),
  Ignore(Box<Ignore>),
  Obj(Box<Obj>),
  Sub(Box<Sub>),
  Id(Box<Id>),
  Iterator(Box<Iterator>),
  Length(Box<Length>),
  Keys(Box<Keys>),
  True(Box<True>),
  SBlock(Box<SBlock>),
  FloatType(Box<FloatType>),
  MatchArm(Box<MatchArm>),
  NumType(Box<NumType>),
  Num(Box<Num>),
}

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub enum ASTNodeType{
  NODES,
  STRING,
  STRINGS,
  F64,
  F32,
  I64,
  I32,
  I16,
  I8,
  U64,
  U32,
  U16,
  U8,
  F32Vec,
  F64Vec,
  I64Vec,
  I32Vec,
  I16Vec,
  I8Vec,
  U64Vec,
  U32Vec,
  U16Vec,
  U8Vec,
  TOKEN,
  TOKENS,
  BOOL,
  NONE,
  Indent,
  Index,
  Dedent,
  False,
  Assign,
  NewLine,
  Match,
  Literal,
  TypeCall,
  StringType,
  Param,
  ObjType,
  ExprTuple,
  Call,
  None,
  Type,
  Tab,
  Expression,
  BreakPoint,
  Text,
  Funct,
  Space,
  Prop,
  Mul,
  Div,
  NotNone,
  LiteralSpace,
  IntType,
  Pow,
  Add,
  Ignore,
  Obj,
  Sub,
  Id,
  Iterator,
  Length,
  Keys,
  True,
  SBlock,
  FloatType,
  MatchArm,
  NumType,
  Num,
}

impl ASTNode{
  
  into_vec!(into_nodes, ASTNode, NODES);
  into_vec!(into_f64_vec, f64, F64Vec);
  into_vec!(into_f32_vec, f32, F32Vec);
  into_vec!(into_i64_vec, i64, I64Vec);
  into_vec!(into_i32_vec, i32, I32Vec);
  into_vec!(into_i16_vec, i16, I16Vec);
  into_vec!(into_i8_vec, i8, I8Vec);
  into_vec!(into_u64_vec, u64, U64Vec);
  into_vec!(into_u32_vec, u32, U32Vec);
  into_vec!(into_u16_vec, u16, U16Vec);
  into_vec!(into_u8_vec, u8, U8Vec);
  into_vec!(into_tokens, Token, TOKENS);
  to_numeric!(to_i8, i8);
  to_numeric!(to_i16, i16);
  to_numeric!(to_i32, i32);
  to_numeric!(to_i64, i64);
  to_numeric!(to_u8, u8);
  to_numeric!(to_u16, u16);
  to_numeric!(to_u32, u32);
  to_numeric!(to_u64, u64);
  to_numeric!(to_f32, f32);
  to_numeric!(to_f64, f64);
  
  pub fn is_numeric (&self)-> bool {
    use ASTNode::*;
    matches!(self, F64(_) | F32(_)| I64(_)| I32(_)| I16(_)| I8(_)| U64(_)| U32(_)| U16(_)| U8(_))
  }
  
  pub fn to_bool (&self)-> bool {
    self.to_u8() != 0
  }
  
  pub fn into_strings (self)-> Vec<String> {
    
    match self{
      ASTNode::STRINGS(strings) => strings,
      _ => Default::default(),
    }
  }
  
  pub fn to_string (&self)-> String {
    
    match self{
      ASTNode::BOOL(val) => val.to_string(),
      ASTNode::STRING(string) => string.to_owned(),
      ASTNode::TOKEN(val) => val.to_string(),
      _ => self.to_token().to_string(),
    }
  }
  
  pub fn to_token (&self)-> Token {
    
    match self{
      ASTNode::Indent(node) => node.tok.clone(),
      ASTNode::Index(node) => node.tok.clone(),
      ASTNode::Dedent(node) => node.tok.clone(),
      ASTNode::Assign(node) => node.tok.clone(),
      ASTNode::NewLine(node) => node.tok.clone(),
      ASTNode::Match(node) => node.tok.clone(),
      ASTNode::Literal(node) => node.tok.clone(),
      ASTNode::TypeCall(node) => node.tok.clone(),
      ASTNode::StringType(node) => node.tok.clone(),
      ASTNode::Param(node) => node.tok.clone(),
      ASTNode::ObjType(node) => node.tok.clone(),
      ASTNode::Call(node) => node.tok.clone(),
      ASTNode::Type(node) => node.tok.clone(),
      ASTNode::Tab(node) => node.tok.clone(),
      ASTNode::Expression(node) => node.tok.clone(),
      ASTNode::BreakPoint(node) => node.tok.clone(),
      ASTNode::Text(node) => node.tok.clone(),
      ASTNode::Funct(node) => node.tok.clone(),
      ASTNode::Space(node) => node.tok.clone(),
      ASTNode::Prop(node) => node.tok.clone(),
      ASTNode::Mul(node) => node.tok.clone(),
      ASTNode::Div(node) => node.tok.clone(),
      ASTNode::LiteralSpace(node) => node.tok.clone(),
      ASTNode::IntType(node) => node.tok.clone(),
      ASTNode::Pow(node) => node.tok.clone(),
      ASTNode::Add(node) => node.tok.clone(),
      ASTNode::Obj(node) => node.tok.clone(),
      ASTNode::Sub(node) => node.tok.clone(),
      ASTNode::Id(node) => node.tok.clone(),
      ASTNode::Iterator(node) => node.tok.clone(),
      ASTNode::Length(node) => node.tok.clone(),
      ASTNode::Keys(node) => node.tok.clone(),
      ASTNode::SBlock(node) => node.tok.clone(),
      ASTNode::FloatType(node) => node.tok.clone(),
      ASTNode::MatchArm(node) => node.tok.clone(),
      ASTNode::NumType(node) => node.tok.clone(),
      ASTNode::Num(node) => node.tok.clone(),
      ASTNode::TOKEN(val) => val.to_owned(),
      _ => Token::empty(),
    }
  }
}

pub trait GetASTNodeType{
  fn get_type(&self) -> ASTNodeType;
}

impl GetASTNodeType for ASTNode{
  
  fn get_type (&self)-> ASTNodeType {
    
    match self{
      ASTNode::Indent(..) => ASTNodeType::Indent,
      ASTNode::Index(..) => ASTNodeType::Index,
      ASTNode::Dedent(..) => ASTNodeType::Dedent,
      ASTNode::False(..) => ASTNodeType::False,
      ASTNode::Assign(..) => ASTNodeType::Assign,
      ASTNode::NewLine(..) => ASTNodeType::NewLine,
      ASTNode::Match(..) => ASTNodeType::Match,
      ASTNode::Literal(..) => ASTNodeType::Literal,
      ASTNode::TypeCall(..) => ASTNodeType::TypeCall,
      ASTNode::StringType(..) => ASTNodeType::StringType,
      ASTNode::Param(..) => ASTNodeType::Param,
      ASTNode::ObjType(..) => ASTNodeType::ObjType,
      ASTNode::ExprTuple(..) => ASTNodeType::ExprTuple,
      ASTNode::Call(..) => ASTNodeType::Call,
      ASTNode::None(..) => ASTNodeType::None,
      ASTNode::Type(..) => ASTNodeType::Type,
      ASTNode::Tab(..) => ASTNodeType::Tab,
      ASTNode::Expression(..) => ASTNodeType::Expression,
      ASTNode::BreakPoint(..) => ASTNodeType::BreakPoint,
      ASTNode::Text(..) => ASTNodeType::Text,
      ASTNode::Funct(..) => ASTNodeType::Funct,
      ASTNode::Space(..) => ASTNodeType::Space,
      ASTNode::Prop(..) => ASTNodeType::Prop,
      ASTNode::Mul(..) => ASTNodeType::Mul,
      ASTNode::Div(..) => ASTNodeType::Div,
      ASTNode::NotNone(..) => ASTNodeType::NotNone,
      ASTNode::LiteralSpace(..) => ASTNodeType::LiteralSpace,
      ASTNode::IntType(..) => ASTNodeType::IntType,
      ASTNode::Pow(..) => ASTNodeType::Pow,
      ASTNode::Add(..) => ASTNodeType::Add,
      ASTNode::Ignore(..) => ASTNodeType::Ignore,
      ASTNode::Obj(..) => ASTNodeType::Obj,
      ASTNode::Sub(..) => ASTNodeType::Sub,
      ASTNode::Id(..) => ASTNodeType::Id,
      ASTNode::Iterator(..) => ASTNodeType::Iterator,
      ASTNode::Length(..) => ASTNodeType::Length,
      ASTNode::Keys(..) => ASTNodeType::Keys,
      ASTNode::True(..) => ASTNodeType::True,
      ASTNode::SBlock(..) => ASTNodeType::SBlock,
      ASTNode::FloatType(..) => ASTNodeType::FloatType,
      ASTNode::MatchArm(..) => ASTNodeType::MatchArm,
      ASTNode::NumType(..) => ASTNodeType::NumType,
      ASTNode::Num(..) => ASTNodeType::Num,
      _ => ASTNodeType::NONE,
    }
  }
}

impl Default for ASTNode{
  
  fn default ()-> Self {
    ASTNode::NONE
  }
}

impl Hash for ASTNode{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    use ASTNode::*;
    
    match self{
      NONE => {},
      F32(val) => val.to_le_bytes().hash(hasher),
      F64(val) => val.to_le_bytes().hash(hasher),
      U8(val) => val.hash(hasher),
      U16(val) => val.hash(hasher),
      U32(val) => val.hash(hasher),
      U64(val) => val.hash(hasher),
      I8(val) => val.hash(hasher),
      I32(val) => val.hash(hasher),
      I16(val) => val.hash(hasher),
      I64(val) => val.hash(hasher),
      BOOL(val) => val.hash(hasher),
      I8Vec(val) => val.hash(hasher),
      I16Vec(val) => val.hash(hasher),
      I32Vec(val) => val.hash(hasher),
      I64Vec(val) => val.hash(hasher),
      U8Vec(val) => val.hash(hasher),
      U16Vec(val) => val.hash(hasher),
      U32Vec(val) => val.hash(hasher),
      U64Vec(val) => val.hash(hasher),
      STRING(string) => string.hash(hasher),
      STRINGS(strings) => strings.hash(hasher),
      Indent(node) => node.hash(hasher),
      Index(node) => node.hash(hasher),
      Dedent(node) => node.hash(hasher),
      False(node) => node.hash(hasher),
      Assign(node) => node.hash(hasher),
      NewLine(node) => node.hash(hasher),
      Match(node) => node.hash(hasher),
      Literal(node) => node.hash(hasher),
      TypeCall(node) => node.hash(hasher),
      StringType(node) => node.hash(hasher),
      Param(node) => node.hash(hasher),
      ObjType(node) => node.hash(hasher),
      ExprTuple(node) => node.hash(hasher),
      Call(node) => node.hash(hasher),
      None(node) => node.hash(hasher),
      Type(node) => node.hash(hasher),
      Tab(node) => node.hash(hasher),
      Expression(node) => node.hash(hasher),
      BreakPoint(node) => node.hash(hasher),
      Text(node) => node.hash(hasher),
      Funct(node) => node.hash(hasher),
      Space(node) => node.hash(hasher),
      Prop(node) => node.hash(hasher),
      Mul(node) => node.hash(hasher),
      Div(node) => node.hash(hasher),
      NotNone(node) => node.hash(hasher),
      LiteralSpace(node) => node.hash(hasher),
      IntType(node) => node.hash(hasher),
      Pow(node) => node.hash(hasher),
      Add(node) => node.hash(hasher),
      Ignore(node) => node.hash(hasher),
      Obj(node) => node.hash(hasher),
      Sub(node) => node.hash(hasher),
      Id(node) => node.hash(hasher),
      Iterator(node) => node.hash(hasher),
      Length(node) => node.hash(hasher),
      Keys(node) => node.hash(hasher),
      True(node) => node.hash(hasher),
      SBlock(node) => node.hash(hasher),
      FloatType(node) => node.hash(hasher),
      MatchArm(node) => node.hash(hasher),
      NumType(node) => node.hash(hasher),
      Num(node) => node.hash(hasher),
      
      TOKEN(tk) =>{
        tk.to_string().replace(" ", "").replace("\n", "").hash(hasher);
      }
      
      TOKENS(tks) =>{
        
        for tk in tks{
          tk.to_string().replace(" ", "").replace("\n", "").hash(hasher);
        }
      }
      
      NODES(nodes) =>{
        
        for node in nodes{
          node.hash(hasher);
        }
      }
      
      F32Vec(vals) =>{
        
        for v in vals{
          v.to_le_bytes().hash(hasher);
        }
      }
      
      F64Vec(vals) =>{
        
        for v in vals{
          v.to_le_bytes().hash(hasher);
        }
      }
    }
  }
}

#[derive(Clone, Debug)]
pub struct Indent{
  pub tok: Token, 
}

impl Indent{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Indent
  }
}

impl ASTNode{
  
  pub fn to_Indent (self)-> Box::<Indent> {
    
    match self{
      Self::Indent(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Indent (&self)-> Option<&Indent> {
    
    match self{
      Self::Indent(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Indent_mut (&mut self)-> Option<&mut Indent> {
    
    match self{
      Self::Indent(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Indent{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Index{
  pub expr:ASTNode, 
  pub tok: Token, 
}

impl Index{
  
  pub fn new (expr: ASTNode, tok: Token)-> Self {
    
    Self{
      expr,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Index
  }
}

impl ASTNode{
  
  pub fn to_Index (self)-> Box::<Index> {
    
    match self{
      Self::Index(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Index (&self)-> Option<&Index> {
    
    match self{
      Self::Index(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Index_mut (&mut self)-> Option<&mut Index> {
    
    match self{
      Self::Index(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Index{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.expr.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Dedent{
  pub tok: Token, 
}

impl Dedent{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Dedent
  }
}

impl ASTNode{
  
  pub fn to_Dedent (self)-> Box::<Dedent> {
    
    match self{
      Self::Dedent(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Dedent (&self)-> Option<&Dedent> {
    
    match self{
      Self::Dedent(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Dedent_mut (&mut self)-> Option<&mut Dedent> {
    
    match self{
      Self::Dedent(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Dedent{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct False{
}

impl False{
  
  pub fn new ()-> Self {
    
    Self{
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::False
  }
}

impl ASTNode{
  
  pub fn to_False (self)-> Box::<False> {
    
    match self{
      Self::False(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_False (&self)-> Option<&False> {
    
    match self{
      Self::False(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_False_mut (&mut self)-> Option<&mut False> {
    
    match self{
      Self::False(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for False{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Assign{
  pub expr:ASTNode, 
  pub id:Box<Id>, 
  pub tok: Token, 
}

impl Assign{
  
  pub fn new (expr: ASTNode, id: Box<Id>, tok: Token)-> Self {
    
    Self{
      expr,
      id,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Assign
  }
}

impl ASTNode{
  
  pub fn to_Assign (self)-> Box::<Assign> {
    
    match self{
      Self::Assign(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Assign (&self)-> Option<&Assign> {
    
    match self{
      Self::Assign(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Assign_mut (&mut self)-> Option<&mut Assign> {
    
    match self{
      Self::Assign(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Assign{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.expr.hash(hasher);
    self.id.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct NewLine{
  pub tok: Token, 
}

impl NewLine{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::NewLine
  }
}

impl ASTNode{
  
  pub fn to_NewLine (self)-> Box::<NewLine> {
    
    match self{
      Self::NewLine(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_NewLine (&self)-> Option<&NewLine> {
    
    match self{
      Self::NewLine(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_NewLine_mut (&mut self)-> Option<&mut NewLine> {
    
    match self{
      Self::NewLine(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for NewLine{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Match{
  pub expr:Box<ExprTuple>, 
  pub matches:Vec<Box<MatchArm>>, 
  pub tok: Token, 
}

impl Match{
  
  pub fn new (expr: Box<ExprTuple>, matches: Vec<Box<MatchArm>>, tok: Token)-> Self {
    
    Self{
      expr,
      matches,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Match
  }
}

impl ASTNode{
  
  pub fn to_Match (self)-> Box::<Match> {
    
    match self{
      Self::Match(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Match (&self)-> Option<&Match> {
    
    match self{
      Self::Match(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Match_mut (&mut self)-> Option<&mut Match> {
    
    match self{
      Self::Match(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Match{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.expr.hash(hasher);
    
    for val in &self.matches{
      val.hash(hasher);
    }
  }
}

#[derive(Clone, Debug)]
pub struct Literal{
  pub val:String, 
  pub tok: Token, 
}

impl Literal{
  
  pub fn new (val: String, tok: Token)-> Self {
    
    Self{
      val,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Literal
  }
}

impl ASTNode{
  
  pub fn to_Literal (self)-> Box::<Literal> {
    
    match self{
      Self::Literal(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Literal (&self)-> Option<&Literal> {
    
    match self{
      Self::Literal(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Literal_mut (&mut self)-> Option<&mut Literal> {
    
    match self{
      Self::Literal(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Literal{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.val.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct TypeCall{
  pub expressions:Vec<ASTNode>, 
  pub tok: Token, 
}

impl TypeCall{
  
  pub fn new (expressions: Vec<ASTNode>, tok: Token)-> Self {
    
    Self{
      expressions,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::TypeCall
  }
}

impl ASTNode{
  
  pub fn to_TypeCall (self)-> Box::<TypeCall> {
    
    match self{
      Self::TypeCall(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_TypeCall (&self)-> Option<&TypeCall> {
    
    match self{
      Self::TypeCall(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_TypeCall_mut (&mut self)-> Option<&mut TypeCall> {
    
    match self{
      Self::TypeCall(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for TypeCall{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    
    for val in &self.expressions{
      val.hash(hasher);
    }
  }
}

#[derive(Clone, Debug)]
pub struct StringType{
  pub tok: Token, 
}

impl StringType{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::StringType
  }
}

impl ASTNode{
  
  pub fn to_StringType (self)-> Box::<StringType> {
    
    match self{
      Self::StringType(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_StringType (&self)-> Option<&StringType> {
    
    match self{
      Self::StringType(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_StringType_mut (&mut self)-> Option<&mut StringType> {
    
    match self{
      Self::StringType(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for StringType{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Param{
  pub name:Box<Id>, 
  pub ty:String, 
  pub tok: Token, 
}

impl Param{
  
  pub fn new (name: Box<Id>, ty: String, tok: Token)-> Self {
    
    Self{
      name,
      ty,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Param
  }
}

impl ASTNode{
  
  pub fn to_Param (self)-> Box::<Param> {
    
    match self{
      Self::Param(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Param (&self)-> Option<&Param> {
    
    match self{
      Self::Param(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Param_mut (&mut self)-> Option<&mut Param> {
    
    match self{
      Self::Param(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Param{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.name.hash(hasher);
    self.ty.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct ObjType{
  pub tok: Token, 
}

impl ObjType{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::ObjType
  }
}

impl ASTNode{
  
  pub fn to_ObjType (self)-> Box::<ObjType> {
    
    match self{
      Self::ObjType(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_ObjType (&self)-> Option<&ObjType> {
    
    match self{
      Self::ObjType(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_ObjType_mut (&mut self)-> Option<&mut ObjType> {
    
    match self{
      Self::ObjType(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for ObjType{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct ExprTuple{
  pub expressions:Vec<ASTNode>, 
}

impl ExprTuple{
  
  pub fn new (expressions: Vec<ASTNode>)-> Self {
    
    Self{
      expressions,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::ExprTuple
  }
}

impl ASTNode{
  
  pub fn to_ExprTuple (self)-> Box::<ExprTuple> {
    
    match self{
      Self::ExprTuple(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_ExprTuple (&self)-> Option<&ExprTuple> {
    
    match self{
      Self::ExprTuple(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_ExprTuple_mut (&mut self)-> Option<&mut ExprTuple> {
    
    match self{
      Self::ExprTuple(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for ExprTuple{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    
    for val in &self.expressions{
      val.hash(hasher);
    }
  }
}

#[derive(Clone, Debug)]
pub struct Call{
  pub args:Vec<ASTNode>, 
  pub name:String, 
  pub tok: Token, 
}

impl Call{
  
  pub fn new (args: Vec<ASTNode>, name: String, tok: Token)-> Self {
    
    Self{
      args,
      name,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Call
  }
}

impl ASTNode{
  
  pub fn to_Call (self)-> Box::<Call> {
    
    match self{
      Self::Call(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Call (&self)-> Option<&Call> {
    
    match self{
      Self::Call(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Call_mut (&mut self)-> Option<&mut Call> {
    
    match self{
      Self::Call(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Call{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    
    for val in &self.args{
      val.hash(hasher);
    }
    self.name.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct None{
}

impl None{
  
  pub fn new ()-> Self {
    
    Self{
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::None
  }
}

impl ASTNode{
  
  pub fn to_None (self)-> Box::<None> {
    
    match self{
      Self::None(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_None (&self)-> Option<&None> {
    
    match self{
      Self::None(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_None_mut (&mut self)-> Option<&mut None> {
    
    match self{
      Self::None(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for None{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Type{
  pub tok: Token, 
}

impl Type{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Type
  }
}

impl ASTNode{
  
  pub fn to_Type (self)-> Box::<Type> {
    
    match self{
      Self::Type(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Type (&self)-> Option<&Type> {
    
    match self{
      Self::Type(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Type_mut (&mut self)-> Option<&mut Type> {
    
    match self{
      Self::Type(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Type{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Tab{
  pub tok: Token, 
}

impl Tab{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Tab
  }
}

impl ASTNode{
  
  pub fn to_Tab (self)-> Box::<Tab> {
    
    match self{
      Self::Tab(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Tab (&self)-> Option<&Tab> {
    
    match self{
      Self::Tab(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Tab_mut (&mut self)-> Option<&mut Tab> {
    
    match self{
      Self::Tab(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Tab{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Expression{
  pub val:ASTNode, 
  pub tok: Token, 
}

impl Expression{
  
  pub fn new (val: ASTNode, tok: Token)-> Self {
    
    Self{
      val,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Expression
  }
}

impl ASTNode{
  
  pub fn to_Expression (self)-> Box::<Expression> {
    
    match self{
      Self::Expression(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Expression (&self)-> Option<&Expression> {
    
    match self{
      Self::Expression(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Expression_mut (&mut self)-> Option<&mut Expression> {
    
    match self{
      Self::Expression(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Expression{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.val.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct BreakPoint{
  pub tok: Token, 
}

impl BreakPoint{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::BreakPoint
  }
}

impl ASTNode{
  
  pub fn to_BreakPoint (self)-> Box::<BreakPoint> {
    
    match self{
      Self::BreakPoint(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_BreakPoint (&self)-> Option<&BreakPoint> {
    
    match self{
      Self::BreakPoint(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_BreakPoint_mut (&mut self)-> Option<&mut BreakPoint> {
    
    match self{
      Self::BreakPoint(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for BreakPoint{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Text{
  pub val:String, 
  pub tok: Token, 
}

impl Text{
  
  pub fn new (val: String, tok: Token)-> Self {
    
    Self{
      val,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Text
  }
}

impl ASTNode{
  
  pub fn to_Text (self)-> Box::<Text> {
    
    match self{
      Self::Text(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Text (&self)-> Option<&Text> {
    
    match self{
      Self::Text(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Text_mut (&mut self)-> Option<&mut Text> {
    
    match self{
      Self::Text(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Text{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.val.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Funct{
  pub content:Vec<ASTNode>, 
  pub name:String, 
  pub params:Vec<Box<Param>>, 
  pub tok: Token, 
}

impl Funct{
  
  pub fn new (content: Vec<ASTNode>, name: String, params: Vec<Box<Param>>, tok: Token)-> Self {
    
    Self{
      content,
      name,
      params,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Funct
  }
}

impl ASTNode{
  
  pub fn to_Funct (self)-> Box::<Funct> {
    
    match self{
      Self::Funct(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Funct (&self)-> Option<&Funct> {
    
    match self{
      Self::Funct(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Funct_mut (&mut self)-> Option<&mut Funct> {
    
    match self{
      Self::Funct(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Funct{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    
    for val in &self.content{
      val.hash(hasher);
    }
    self.name.hash(hasher);
    
    for val in &self.params{
      val.hash(hasher);
    }
  }
}

#[derive(Clone, Debug)]
pub struct Space{
  pub count:u32, 
  pub tok: Token, 
}

impl Space{
  
  pub fn new (count: u32, tok: Token)-> Self {
    
    Self{
      count,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Space
  }
}

impl ASTNode{
  
  pub fn to_Space (self)-> Box::<Space> {
    
    match self{
      Self::Space(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Space (&self)-> Option<&Space> {
    
    match self{
      Self::Space(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Space_mut (&mut self)-> Option<&mut Space> {
    
    match self{
      Self::Space(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Space{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.count.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Prop{
  pub name:String, 
  pub tok: Token, 
}

impl Prop{
  
  pub fn new (name: String, tok: Token)-> Self {
    
    Self{
      name,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Prop
  }
}

impl ASTNode{
  
  pub fn to_Prop (self)-> Box::<Prop> {
    
    match self{
      Self::Prop(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Prop (&self)-> Option<&Prop> {
    
    match self{
      Self::Prop(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Prop_mut (&mut self)-> Option<&mut Prop> {
    
    match self{
      Self::Prop(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Prop{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.name.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Mul{
  pub l:ASTNode, 
  pub r:ASTNode, 
  pub tok: Token, 
}

impl Mul{
  
  pub fn new (l: ASTNode, r: ASTNode, tok: Token)-> Self {
    
    Self{
      l,
      r,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Mul
  }
}

impl ASTNode{
  
  pub fn to_Mul (self)-> Box::<Mul> {
    
    match self{
      Self::Mul(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Mul (&self)-> Option<&Mul> {
    
    match self{
      Self::Mul(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Mul_mut (&mut self)-> Option<&mut Mul> {
    
    match self{
      Self::Mul(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Mul{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.l.hash(hasher);
    self.r.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Div{
  pub l:ASTNode, 
  pub r:ASTNode, 
  pub tok: Token, 
}

impl Div{
  
  pub fn new (l: ASTNode, r: ASTNode, tok: Token)-> Self {
    
    Self{
      l,
      r,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Div
  }
}

impl ASTNode{
  
  pub fn to_Div (self)-> Box::<Div> {
    
    match self{
      Self::Div(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Div (&self)-> Option<&Div> {
    
    match self{
      Self::Div(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Div_mut (&mut self)-> Option<&mut Div> {
    
    match self{
      Self::Div(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Div{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.l.hash(hasher);
    self.r.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct NotNone{
}

impl NotNone{
  
  pub fn new ()-> Self {
    
    Self{
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::NotNone
  }
}

impl ASTNode{
  
  pub fn to_NotNone (self)-> Box::<NotNone> {
    
    match self{
      Self::NotNone(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_NotNone (&self)-> Option<&NotNone> {
    
    match self{
      Self::NotNone(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_NotNone_mut (&mut self)-> Option<&mut NotNone> {
    
    match self{
      Self::NotNone(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for NotNone{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct LiteralSpace{
  pub content:Vec<ASTNode>, 
  pub tok: Token, 
}

impl LiteralSpace{
  
  pub fn new (content: Vec<ASTNode>, tok: Token)-> Self {
    
    Self{
      content,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::LiteralSpace
  }
}

impl ASTNode{
  
  pub fn to_LiteralSpace (self)-> Box::<LiteralSpace> {
    
    match self{
      Self::LiteralSpace(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_LiteralSpace (&self)-> Option<&LiteralSpace> {
    
    match self{
      Self::LiteralSpace(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_LiteralSpace_mut (&mut self)-> Option<&mut LiteralSpace> {
    
    match self{
      Self::LiteralSpace(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for LiteralSpace{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    
    for val in &self.content{
      val.hash(hasher);
    }
  }
}

#[derive(Clone, Debug)]
pub struct IntType{
  pub tok: Token, 
}

impl IntType{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::IntType
  }
}

impl ASTNode{
  
  pub fn to_IntType (self)-> Box::<IntType> {
    
    match self{
      Self::IntType(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_IntType (&self)-> Option<&IntType> {
    
    match self{
      Self::IntType(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_IntType_mut (&mut self)-> Option<&mut IntType> {
    
    match self{
      Self::IntType(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for IntType{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Pow{
  pub l:ASTNode, 
  pub r:ASTNode, 
  pub tok: Token, 
}

impl Pow{
  
  pub fn new (l: ASTNode, r: ASTNode, tok: Token)-> Self {
    
    Self{
      l,
      r,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Pow
  }
}

impl ASTNode{
  
  pub fn to_Pow (self)-> Box::<Pow> {
    
    match self{
      Self::Pow(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Pow (&self)-> Option<&Pow> {
    
    match self{
      Self::Pow(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Pow_mut (&mut self)-> Option<&mut Pow> {
    
    match self{
      Self::Pow(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Pow{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.l.hash(hasher);
    self.r.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Add{
  pub l:ASTNode, 
  pub r:ASTNode, 
  pub tok: Token, 
}

impl Add{
  
  pub fn new (l: ASTNode, r: ASTNode, tok: Token)-> Self {
    
    Self{
      l,
      r,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Add
  }
}

impl ASTNode{
  
  pub fn to_Add (self)-> Box::<Add> {
    
    match self{
      Self::Add(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Add (&self)-> Option<&Add> {
    
    match self{
      Self::Add(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Add_mut (&mut self)-> Option<&mut Add> {
    
    match self{
      Self::Add(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Add{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.l.hash(hasher);
    self.r.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Ignore{
}

impl Ignore{
  
  pub fn new ()-> Self {
    
    Self{
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Ignore
  }
}

impl ASTNode{
  
  pub fn to_Ignore (self)-> Box::<Ignore> {
    
    match self{
      Self::Ignore(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Ignore (&self)-> Option<&Ignore> {
    
    match self{
      Self::Ignore(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Ignore_mut (&mut self)-> Option<&mut Ignore> {
    
    match self{
      Self::Ignore(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Ignore{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Obj{
  pub id:Box<Id>, 
  pub path:Vec<ASTNode>, 
  pub tok: Token, 
}

impl Obj{
  
  pub fn new (id: Box<Id>, path: Vec<ASTNode>, tok: Token)-> Self {
    
    Self{
      id,
      path,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Obj
  }
}

impl ASTNode{
  
  pub fn to_Obj (self)-> Box::<Obj> {
    
    match self{
      Self::Obj(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Obj (&self)-> Option<&Obj> {
    
    match self{
      Self::Obj(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Obj_mut (&mut self)-> Option<&mut Obj> {
    
    match self{
      Self::Obj(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Obj{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.id.hash(hasher);
    
    for val in &self.path{
      val.hash(hasher);
    }
  }
}

#[derive(Clone, Debug)]
pub struct Sub{
  pub l:ASTNode, 
  pub r:ASTNode, 
  pub tok: Token, 
}

impl Sub{
  
  pub fn new (l: ASTNode, r: ASTNode, tok: Token)-> Self {
    
    Self{
      l,
      r,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Sub
  }
}

impl ASTNode{
  
  pub fn to_Sub (self)-> Box::<Sub> {
    
    match self{
      Self::Sub(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Sub (&self)-> Option<&Sub> {
    
    match self{
      Self::Sub(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Sub_mut (&mut self)-> Option<&mut Sub> {
    
    match self{
      Self::Sub(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Sub{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.l.hash(hasher);
    self.r.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Id{
  pub at:bool, 
  pub name:String, 
  pub tok: Token, 
}

impl Id{
  
  pub fn new (at: bool, name: String, tok: Token)-> Self {
    
    Self{
      at,
      name,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Id
  }
}

impl ASTNode{
  
  pub fn to_Id (self)-> Box::<Id> {
    
    match self{
      Self::Id(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Id (&self)-> Option<&Id> {
    
    match self{
      Self::Id(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Id_mut (&mut self)-> Option<&mut Id> {
    
    match self{
      Self::Id(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Id{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.at.hash(hasher);
    self.name.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Iterator{
  pub call:Box<Call>, 
  pub tok: Token, 
}

impl Iterator{
  
  pub fn new (call: Box<Call>, tok: Token)-> Self {
    
    Self{
      call,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Iterator
  }
}

impl ASTNode{
  
  pub fn to_Iterator (self)-> Box::<Iterator> {
    
    match self{
      Self::Iterator(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Iterator (&self)-> Option<&Iterator> {
    
    match self{
      Self::Iterator(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Iterator_mut (&mut self)-> Option<&mut Iterator> {
    
    match self{
      Self::Iterator(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Iterator{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.call.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Length{
  pub tok: Token, 
}

impl Length{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Length
  }
}

impl ASTNode{
  
  pub fn to_Length (self)-> Box::<Length> {
    
    match self{
      Self::Length(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Length (&self)-> Option<&Length> {
    
    match self{
      Self::Length(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Length_mut (&mut self)-> Option<&mut Length> {
    
    match self{
      Self::Length(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Length{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Keys{
  pub tok: Token, 
}

impl Keys{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Keys
  }
}

impl ASTNode{
  
  pub fn to_Keys (self)-> Box::<Keys> {
    
    match self{
      Self::Keys(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Keys (&self)-> Option<&Keys> {
    
    match self{
      Self::Keys(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Keys_mut (&mut self)-> Option<&mut Keys> {
    
    match self{
      Self::Keys(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Keys{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct True{
}

impl True{
  
  pub fn new ()-> Self {
    
    Self{
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::True
  }
}

impl ASTNode{
  
  pub fn to_True (self)-> Box::<True> {
    
    match self{
      Self::True(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_True (&self)-> Option<&True> {
    
    match self{
      Self::True(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_True_mut (&mut self)-> Option<&mut True> {
    
    match self{
      Self::True(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for True{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct SBlock{
  pub content:Vec<ASTNode>, 
  pub ty:String, 
  pub tok: Token, 
}

impl SBlock{
  
  pub fn new (content: Vec<ASTNode>, ty: String, tok: Token)-> Self {
    
    Self{
      content,
      ty,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::SBlock
  }
}

impl ASTNode{
  
  pub fn to_SBlock (self)-> Box::<SBlock> {
    
    match self{
      Self::SBlock(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_SBlock (&self)-> Option<&SBlock> {
    
    match self{
      Self::SBlock(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_SBlock_mut (&mut self)-> Option<&mut SBlock> {
    
    match self{
      Self::SBlock(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for SBlock{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    
    for val in &self.content{
      val.hash(hasher);
    }
    self.ty.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct FloatType{
  pub tok: Token, 
}

impl FloatType{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::FloatType
  }
}

impl ASTNode{
  
  pub fn to_FloatType (self)-> Box::<FloatType> {
    
    match self{
      Self::FloatType(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_FloatType (&self)-> Option<&FloatType> {
    
    match self{
      Self::FloatType(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_FloatType_mut (&mut self)-> Option<&mut FloatType> {
    
    match self{
      Self::FloatType(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for FloatType{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct MatchArm{
  pub content:Vec<ASTNode>, 
  pub default:bool, 
  pub match_expr:Option<Box<ExprTuple>>, 
  pub tok: Token, 
}

impl MatchArm{
  
  pub fn new (content: Vec<ASTNode>, default: bool, match_expr: Option<Box<ExprTuple>>, tok: Token)-> Self {
    
    Self{
      content,
      default,
      match_expr,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::MatchArm
  }
}

impl ASTNode{
  
  pub fn to_MatchArm (self)-> Box::<MatchArm> {
    
    match self{
      Self::MatchArm(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_MatchArm (&self)-> Option<&MatchArm> {
    
    match self{
      Self::MatchArm(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_MatchArm_mut (&mut self)-> Option<&mut MatchArm> {
    
    match self{
      Self::MatchArm(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for MatchArm{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    
    for val in &self.content{
      val.hash(hasher);
    }
    self.default.hash(hasher);
    self.match_expr.hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct NumType{
  pub tok: Token, 
}

impl NumType{
  
  pub fn new (tok: Token)-> Self {
    
    Self{
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::NumType
  }
}

impl ASTNode{
  
  pub fn to_NumType (self)-> Box::<NumType> {
    
    match self{
      Self::NumType(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_NumType (&self)-> Option<&NumType> {
    
    match self{
      Self::NumType(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_NumType_mut (&mut self)-> Option<&mut NumType> {
    
    match self{
      Self::NumType(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for NumType{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
  }
}

#[derive(Clone, Debug)]
pub struct Num{
  pub val:String, 
  pub tok: Token, 
}

impl Num{
  
  pub fn new (val: String, tok: Token)-> Self {
    
    Self{
      val,
      tok,
    }
  }
  
  pub fn get_type (&self)-> ASTNodeType {
    ASTNodeType::Num
  }
}

impl ASTNode{
  
  pub fn to_Num (self)-> Box::<Num> {
    
    match self{
      Self::Num(val) => val,
      _ => panic!()
    }
  }
  
  pub fn as_Num (&self)-> Option<&Num> {
    
    match self{
      Self::Num(val) => Some(val.as_ref()),
      _ => None
    }
  }
  
  pub fn as_Num_mut (&mut self)-> Option<&mut Num> {
    
    match self{
      Self::Num(val) => Some(val.as_mut()),
      _ => None
    }
  }
}

impl Hash for Num{
  
  fn hash<H: std::hash::Hasher> (&self,hasher: &mut H) {
    self.get_type().hash(hasher);
    self.val.hash(hasher);
  }
}


/* script_statement+ */
fn reducer_000 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* script_statement */
fn reducer_001 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* script_statement+ */
fn reducer_002 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* text */
fn reducer_003 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* assignment */
fn reducer_004 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* function */
fn reducer_005 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* format */
fn reducer_006 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* call */
fn reducer_007 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* object */
fn reducer_008 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* literal_space */
fn reducer_009 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* block::<t_SBlock, script_statement*> */
fn reducer_010 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* ( c:sym | '.' )
                                    :ast { t_Text, val: str(tok), tok } */
fn reducer_011 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let tok_rule_0 = __rule_rng__;
  let tok_rule_0 = tok_rule_0.to_token(unsafe{&mut *_ctx_}.get_reader_mut());
  let tok_rule_0 = tok_rule_0.to_string();
  let var_2_0 = Text::new(
    tok_rule_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Text(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* ( c:sym | '.' )
                                    :ast { t_Text, val: str(tok), tok } */
fn reducer_012 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let tok_rule_0 = __rule_rng__;
  let tok_rule_0 = tok_rule_0.to_token(unsafe{&mut *_ctx_}.get_reader_mut());
  let tok_rule_0 = tok_rule_0.to_string();
  let var_2_0 = Text::new(
    tok_rule_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Text(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( "\\" c:any ) 
                                    :ast { t_Text, val: str(tok<1>), tok } */
fn reducer_013 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let tok_rule_0 = __rule_rng__;
  let tok_rule_0 = tok_rule_0.trim(1, 0);
  let tok_rule_0 = tok_rule_0.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_2_0 = Text::new(
    tok_rule_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Text(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* num */
fn reducer_014 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* at_string_literal */
fn reducer_015 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* expression */
fn reducer_016 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* c:sym */
fn reducer_017 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* '.' */
fn reducer_018 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* binding_id "={"{:9999} expr "}"
                                    :ast { t_Assign, id: $1, expr: $3, tok } */
fn reducer_019 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, _, _) = slots.take(2);
  let AstSlot (_, __tok_rng_3, _) = slots.take(3);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_3;
  let obj_2_0 = ref_2;
  let obj_0_1 = ref_0;
  let obj_0_1 = obj_0_1.to_Id();
  let var_5_0 = Assign::new(
    obj_2_0,
    obj_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Assign(Box::new(var_5_0)), __rule_rng__, TokenRange::default()));
}


/* fn_name param* '{' ( function_statement | function )* "}"{1}
                                    
                                    :ast { t_Funct, name: str($1), params:$2, content: $4, tok } */
fn reducer_020 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  slots.take(2);
  let AstSlot (ref_3, _, _) = slots.take(3);
  let AstSlot (_, __tok_rng_4, _) = slots.take(4);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_4;
  let obj_3_0 = ref_3.into_nodes();
  let tok_0_1 = __tok_rng_0.to_token(unsafe{&mut*_ctx_}.get_reader_mut());
  let tok_0_1 = tok_0_1.to_string();
  let obj_1_2 = ref_1.into_nodes();
  let var_6_0 = Funct::new(
    obj_3_0,
    tok_0_1,
    obj_1_2.into_iter().map(|v|match v { ASTNode::Param(node) => node, _ => panic!("could not convert")}).collect::<Vec<_>>(),
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Funct(Box::new(var_6_0)), __rule_rng__, TokenRange::default()));
}


/* fn_name param* '{' ( function_statement | function )* "}"{1}
                                    
                                    :ast { t_Funct, name: str($1), params:$2, content: $4, tok } */
fn reducer_021 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, _, _) = slots.take(2);
  let AstSlot (_, __tok_rng_3, _) = slots.take(3);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_3;
  let obj_2_0 = ref_2.into_nodes();
  let tok_0_1 = __tok_rng_0.to_token(unsafe{&mut*_ctx_}.get_reader_mut());
  let tok_0_1 = tok_0_1.to_string();
  let var_5_0 = Funct::new(
    obj_2_0,
    tok_0_1,
    vec![],
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Funct(Box::new(var_5_0)), __rule_rng__, TokenRange::default()));
}


/* fn_name param* '{' ( function_statement | function )* "}"{1}
                                    
                                    :ast { t_Funct, name: str($1), params:$2, content: $4, tok } */
fn reducer_022 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  slots.take(2);
  let AstSlot (_, __tok_rng_3, _) = slots.take(3);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_3;
  let tok_0_1 = __tok_rng_0.to_token(unsafe{&mut*_ctx_}.get_reader_mut());
  let tok_0_1 = tok_0_1.to_string();
  let obj_1_2 = ref_1.into_nodes();
  let var_5_0 = Funct::new(
    vec![],
    tok_0_1,
    obj_1_2.into_iter().map(|v|match v { ASTNode::Param(node) => node, _ => panic!("could not convert")}).collect::<Vec<_>>(),
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Funct(Box::new(var_5_0)), __rule_rng__, TokenRange::default()));
}


/* fn_name param* '{' ( function_statement | function )* "}"{1}
                                    
                                    :ast { t_Funct, name: str($1), params:$2, content: $4, tok } */
fn reducer_023 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let tok_0_1 = __tok_rng_0.to_token(unsafe{&mut*_ctx_}.get_reader_mut());
  let tok_0_1 = tok_0_1.to_string();
  let var_4_0 = Funct::new(
    vec![],
    tok_0_1,
    vec![],
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Funct(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* param */
fn reducer_024 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* param* */
fn reducer_025 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* function_statement */
fn reducer_026 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* function */
fn reducer_027 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* ( function_statement | function ) */
fn reducer_028 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* ( function_statement | function ) */
fn reducer_029 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* ( function_statement | function )* */
fn reducer_030 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* ( function_statement | function )* */
fn reducer_031 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* tk:( '@' "+"{:9999})          :ast { t_Indent, tok } */
fn reducer_032 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = Indent::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Indent(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( '@' "-"{:9999})          :ast { t_Dedent, tok } */
fn reducer_033 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = Dedent::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Dedent(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( '@' ";"{:9999})          :ast { t_BreakPoint, tok } */
fn reducer_034 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = BreakPoint::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::BreakPoint(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* '@' tk:(c:num+)               :ast { t_Space, count: u32($2), tok } */
fn reducer_035 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_1_0 = __tok_rng_1;
  let tok_1_0 = tok_1_0.parse::<u32>(unsafe{&*_ctx_}.get_str());
  let var_3_0 = Space::new(
    tok_1_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Space(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( "\\" " "{:9999})         :ast { t_Space, tok } */
fn reducer_036 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = Space::new(
    0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Space(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( "\\" "n"{:9999})         :ast { t_NewLine, tok } */
fn reducer_037 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = NewLine::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::NewLine(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* fn_name args              :ast { t_Call, name: str($1), args: $2, tok } */
fn reducer_038 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1.into_nodes();
  let tok_0_1 = __tok_rng_0.to_token(unsafe{&mut*_ctx_}.get_reader_mut());
  let tok_0_1 = tok_0_1.to_string();
  let var_3_0 = Call::new(
    obj_1_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Call(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* id  object_accessor*        
                                    :ast { t_Obj, id: $1, path:$2, tok } */
fn reducer_039 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_0_0 = ref_0;
  let obj_0_0 = obj_0_0.to_Id();
  let obj_1_1 = ref_1.into_nodes();
  let var_3_0 = Obj::new(
    obj_0_0,
    obj_1_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Obj(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* id  object_accessor*        
                                    :ast { t_Obj, id: $1, path:$2, tok } */
fn reducer_040 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let obj_0_0 = obj_0_0.to_Id();
  let var_2_0 = Obj::new(
    obj_0_0,
    vec![],
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Obj(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* object_accessor */
fn reducer_041 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* object_accessor* */
fn reducer_042 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* "@{"  literal_space_statement+  "}"     
                                      :ast { t_LiteralSpace, content: $2, tok } */
fn reducer_043 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let var_4_0 = LiteralSpace::new(
    obj_1_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::LiteralSpace(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* literal_space_statement */
fn reducer_044 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* literal_space_statement+ */
fn reducer_045 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* "[" Content? "]"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_046 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_4_0 = SBlock::new(
    obj_1_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* "[" Content? "]"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_047 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_3_0 = SBlock::new(
    vec![],
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* "(" Content? ")"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_048 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_4_0 = SBlock::new(
    obj_1_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* "(" Content? ")"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_049 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_3_0 = SBlock::new(
    vec![],
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* "{" Content? "}"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_050 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_4_0 = SBlock::new(
    obj_1_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* "{" Content? "}"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_051 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_3_0 = SBlock::new(
    vec![],
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* script_statement */
fn reducer_052 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* script_statement* */
fn reducer_053 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* script_statement */
fn reducer_054 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* script_statement* */
fn reducer_055 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* script_statement */
fn reducer_056 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* script_statement* */
fn reducer_057 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* tk:( c:num+ ) 
                                      :ast { t_Num, val: str(tok), tok } */
fn reducer_058 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let tok_rule_0 = __rule_rng__;
  let tok_rule_0 = tok_rule_0.to_token(unsafe{&mut *_ctx_}.get_reader_mut());
  let tok_rule_0 = tok_rule_0.to_string();
  let var_2_0 = Num::new(
    tok_rule_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Num(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "@\""  tk:( c:any | "\\" c:any )+  "\""     
                                      :ast { t_Literal, val: str($2), tok } */
fn reducer_059 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_tokens();
  let obj_1_0 = (obj_1_0.first().unwrap() + obj_1_0.last().unwrap()).to_string();
  let var_4_0 = Literal::new(
    obj_1_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Literal(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( c:any | "\\" c:any ) */
fn reducer_060 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let tok_0_0 = __tok_rng_0.to_token(unsafe{&mut*_ctx_}.get_reader_mut());
  let mut obj_2_0 = vec![];
  obj_2_0.push(tok_0_0);
  slots.assign(0, AstSlot(ASTNode::TOKENS(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* tk:( c:any | "\\" c:any )+ */
fn reducer_061 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_1_0 = __tok_rng_1.to_token(unsafe{&mut*_ctx_}.get_reader_mut());
  let mut obj_0_0 = ref_0.into_tokens();
  obj_0_0.push(tok_1_0);
  slots.assign(0, AstSlot(ASTNode::TOKENS(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* "@["  expr  "]"              :ast { t_Expression, val: $2, tok } */
fn reducer_062 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1;
  let var_4_0 = Expression::new(
    obj_1_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Expression(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( '@' ( c:id | "_" | '-' ) ( c:id | "_" | '-' | c:num )* )
                                    :ast { t_Id, name: str($1), at: true, tok } */
fn reducer_063 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_2_0 = true;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_3_0 = Id::new(
    obj_2_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Id(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* expr "+"{1} expr{1}           :ast { t_Add, l:$1, r:$3, tok } */
fn reducer_064 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_0_0 = ref_0;
  let obj_2_1 = ref_2;
  let var_4_0 = Add::new(
    obj_0_0,
    obj_2_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Add(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* expr "-"{1} expr{1}           :ast { t_Sub, l:$1, r:$3, tok } */
fn reducer_065 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_0_0 = ref_0;
  let obj_2_1 = ref_2;
  let var_4_0 = Sub::new(
    obj_0_0,
    obj_2_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Sub(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* expr "*"{3} expr{3}           :ast { t_Mul, l:$1, r:$3, tok } */
fn reducer_066 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_0_0 = ref_0;
  let obj_2_1 = ref_2;
  let var_4_0 = Mul::new(
    obj_0_0,
    obj_2_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Mul(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* expr "^"{4} expr{4}           :ast { t_Pow, l:$1, r:$3, tok } */
fn reducer_067 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_0_0 = ref_0;
  let obj_2_1 = ref_2;
  let var_4_0 = Pow::new(
    obj_0_0,
    obj_2_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Pow(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* expr "/"{2} expr{2}           :ast { t_Div, l:$1, r:$3, tok } */
fn reducer_068 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_0_0 = ref_0;
  let obj_2_1 = ref_2;
  let var_4_0 = Div::new(
    obj_0_0,
    obj_2_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Div(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* num */
fn reducer_069 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* object */
fn reducer_070 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* at_string_literal */
fn reducer_071 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* plain_string_literal */
fn reducer_072 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* id ":" type              
                                    :ast { t_Param, name: $1, ty: str($3), tok } */
fn reducer_073 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_0_0 = ref_0;
  let obj_0_0 = obj_0_0.to_Id();
  let tok_2_1 = __tok_rng_2;
  let tok_2_1 = tok_2_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_4_0 = Param::new(
    obj_0_0,
    tok_2_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Param(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* text */
fn reducer_074 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* assignment */
fn reducer_075 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* format */
fn reducer_076 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* call */
fn reducer_077 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* match */
fn reducer_078 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* literal_space */
fn reducer_079 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* object */
fn reducer_080 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* block::<t_SBlock, function_statement*> */
fn reducer_081 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* tk:( '#' ( c:id | "_" | '-' ) ( c:id | "_" | '-' | c:num )* ) */
fn reducer_082 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* "(" expr(*",") ")" 
                                    :ast { [$2] } */
fn reducer_083 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  slots.assign(0, AstSlot(ASTNode::NODES(obj_1_0), __rule_rng__, TokenRange::default()));
}


/* "(" expr(*",") ")" 
                                    :ast { [$2] } */
fn reducer_084 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_3_0 = vec![];;
  slots.assign(0, AstSlot(ASTNode::NODES(obj_3_0), __rule_rng__, TokenRange::default()));
}


/* expr */
fn reducer_085 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* expr(*",") */
fn reducer_086 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_2_0 = ref_2;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_2_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* tk:( '.' non_binding_id)         :ast { t_Prop, name: str(tok<1>), tok } */
fn reducer_087 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let tok_rule_0 = __rule_rng__;
  let tok_rule_0 = tok_rule_0.trim(1, 0);
  let tok_rule_0 = tok_rule_0.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_2_0 = Prop::new(
    tok_rule_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Prop(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( '.' "[") expr "]"           :ast { t_Index, expr: $2, tok } */
fn reducer_088 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1;
  let var_4_0 = Index::new(
    obj_1_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Index(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( '.' "(") expr(*",") ")"     :ast { t_TypeCall, expressions: $2, tok } */
fn reducer_089 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let var_4_0 = TypeCall::new(
    obj_1_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::TypeCall(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( '.' "(") expr(*",") ")"     :ast { t_TypeCall, expressions: $2, tok } */
fn reducer_090 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let var_3_0 = TypeCall::new(
    vec![],
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::TypeCall(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( '.' "#type")                :ast { t_Type, tok } */
fn reducer_091 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = Type::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Type(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( '.' "#keys")                :ast { t_Keys, tok } */
fn reducer_092 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = Keys::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Keys(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( '.' "#len")                 :ast { t_Length, tok } */
fn reducer_093 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = Length::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Length(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( '.' 'iter') call            :ast { t_Iterator, call, tok } */
fn reducer_094 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let obj_1_0 = obj_1_0.to_Call();
  let var_3_0 = Iterator::new(
    obj_1_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Iterator(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* expr */
fn reducer_095 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* expr(*",") */
fn reducer_096 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_2_0 = ref_2;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_2_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* binding_id */
fn reducer_097 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* non_binding_id */
fn reducer_098 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* text */
fn reducer_099 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* format */
fn reducer_100 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* literal_format */
fn reducer_101 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* call */
fn reducer_102 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* object */
fn reducer_103 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* block::<t_SBlock, literal_space_statement+> */
fn reducer_104 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* "\""  tk:( c:any | "\\" c:any )+  "\""     
                                      :ast { t_Literal, val: str($2), tok } */
fn reducer_105 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_tokens();
  let obj_1_0 = (obj_1_0.first().unwrap() + obj_1_0.last().unwrap()).to_string();
  let var_4_0 = Literal::new(
    obj_1_0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Literal(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( c:any | "\\" c:any ) */
fn reducer_106 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let tok_0_0 = __tok_rng_0.to_token(unsafe{&mut*_ctx_}.get_reader_mut());
  let mut obj_2_0 = vec![];
  obj_2_0.push(tok_0_0);
  slots.assign(0, AstSlot(ASTNode::TOKENS(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* tk:( c:any | "\\" c:any )+ */
fn reducer_107 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_1_0 = __tok_rng_1.to_token(unsafe{&mut*_ctx_}.get_reader_mut());
  let mut obj_0_0 = ref_0.into_tokens();
  obj_0_0.push(tok_1_0);
  slots.assign(0, AstSlot(ASTNode::TOKENS(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* "obj"                         :ast { t_ObjType, tok } */
fn reducer_108 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = ObjType::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::ObjType(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "num"                         :ast { t_NumType, tok } */
fn reducer_109 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = NumType::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::NumType(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "int"                         :ast { t_IntType, tok } */
fn reducer_110 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = IntType::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::IntType(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "flt"                         :ast { t_FloatType, tok } */
fn reducer_111 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = FloatType::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::FloatType(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "str"                         :ast { t_StringType, tok } */
fn reducer_112 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = StringType::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::StringType(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "match" tuple "{"  match_arm*  "}" 
                                    :ast { t_Match, expr: $2, matches:$4, tok } */
fn reducer_113 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  slots.take(2);
  let AstSlot (ref_3, _, _) = slots.take(3);
  let AstSlot (_, __tok_rng_4, _) = slots.take(4);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_4;
  let obj_1_0 = ref_1;
  let obj_1_0 = obj_1_0.to_ExprTuple();
  let obj_3_1 = ref_3.into_nodes();
  let var_6_0 = Match::new(
    obj_1_0,
    obj_3_1.into_iter().map(|v|match v { ASTNode::MatchArm(node) => node, _ => panic!("could not convert")}).collect::<Vec<_>>(),
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Match(Box::new(var_6_0)), __rule_rng__, TokenRange::default()));
}


/* "match" tuple "{"  match_arm*  "}" 
                                    :ast { t_Match, expr: $2, matches:$4, tok } */
fn reducer_114 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  slots.take(2);
  let AstSlot (_, __tok_rng_3, _) = slots.take(3);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_3;
  let obj_1_0 = ref_1;
  let obj_1_0 = obj_1_0.to_ExprTuple();
  let var_5_0 = Match::new(
    obj_1_0,
    vec![],
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Match(Box::new(var_5_0)), __rule_rng__, TokenRange::default()));
}


/* match_arm */
fn reducer_115 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* match_arm* */
fn reducer_116 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* "[" Content? "]"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_117 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_4_0 = SBlock::new(
    obj_1_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* "[" Content? "]"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_118 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_3_0 = SBlock::new(
    vec![],
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* "(" Content? ")"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_119 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_4_0 = SBlock::new(
    obj_1_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* "(" Content? ")"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_120 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_3_0 = SBlock::new(
    vec![],
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* "{" Content? "}"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_121 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_4_0 = SBlock::new(
    obj_1_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* "{" Content? "}"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_122 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_3_0 = SBlock::new(
    vec![],
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* function_statement */
fn reducer_123 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* function_statement* */
fn reducer_124 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* function_statement */
fn reducer_125 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* function_statement* */
fn reducer_126 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* function_statement */
fn reducer_127 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* function_statement* */
fn reducer_128 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* tk:( ( c:id | "_" | '-' ) ( c:id | "_" | '-' | c:num )* )
                                    :ast { t_Id, name: str($1), tok } */
fn reducer_129 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_2_0 = Id::new(
    false,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Id(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( c:tab )                  :ast { t_Tab, tok } */
fn reducer_130 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = Tab::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Tab(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( " " )                    :ast { t_Space, tok } */
fn reducer_131 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = Space::new(
    0,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::Space(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* tk:( c:nl )                   :ast { t_NewLine, tok } */
fn reducer_132 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = NewLine::new(
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::NewLine(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "[" Content? "]"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_133 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_4_0 = SBlock::new(
    obj_1_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* "[" Content? "]"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_134 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_3_0 = SBlock::new(
    vec![],
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* "(" Content? ")"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_135 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_4_0 = SBlock::new(
    obj_1_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* "(" Content? ")"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_136 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_3_0 = SBlock::new(
    vec![],
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* "{" Content? "}"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_137 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_4_0 = SBlock::new(
    obj_1_0,
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* "{" Content? "}"{1}            :ast { t_T, ty: str($1), content: $2, tok } */
fn reducer_138 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let tok_0_1 = __tok_rng_0;
  let tok_0_1 = tok_0_1.to_slice(unsafe{&*_ctx_}.get_str()).to_string();
  let var_3_0 = SBlock::new(
    vec![],
    tok_0_1,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::SBlock(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* literal_space_statement */
fn reducer_139 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* literal_space_statement+ */
fn reducer_140 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* literal_space_statement */
fn reducer_141 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* literal_space_statement+ */
fn reducer_142 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* literal_space_statement */
fn reducer_143 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* literal_space_statement+ */
fn reducer_144 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* tuple "{" ( function_statement )*{2} "}"
                                    :ast { t_MatchArm, match_expr: $1, content: $3, tok } */
fn reducer_145 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, _, _) = slots.take(2);
  let AstSlot (_, __tok_rng_3, _) = slots.take(3);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_3;
  let obj_2_0 = ref_2.into_nodes();
  let obj_0_2 = ref_0;
  let obj_0_2 = obj_0_2.to_ExprTuple();
  let var_5_0 = MatchArm::new(
    obj_2_0,
    false,
    Some(obj_0_2),
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::MatchArm(Box::new(var_5_0)), __rule_rng__, TokenRange::default()));
}


/* tuple "{" ( function_statement )*{2} "}"
                                    :ast { t_MatchArm, match_expr: $1, content: $3, tok } */
fn reducer_146 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_0_2 = ref_0;
  let obj_0_2 = obj_0_2.to_ExprTuple();
  let var_4_0 = MatchArm::new(
    vec![],
    false,
    Some(obj_0_2),
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::MatchArm(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* "{" ( function_statement )*{2} "}"
                                    :ast { t_MatchArm, default: true, content: $2, tok } */
fn reducer_147 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let obj_4_1 = true;
  let var_5_0 = MatchArm::new(
    obj_1_0,
    obj_4_1,
    None,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::MatchArm(Box::new(var_5_0)), __rule_rng__, TokenRange::default()));
}


/* "{" ( function_statement )*{2} "}"
                                    :ast { t_MatchArm, default: true, content: $2, tok } */
fn reducer_148 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (_, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_3_1 = true;
  let var_4_0 = MatchArm::new(
    vec![],
    obj_3_1,
    None,
    __rule_rng__.to_token(unsafe{{&mut*_ctx_}}.get_reader_mut()),
  );
  slots.assign(0, AstSlot(ASTNode::MatchArm(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* function_statement */
fn reducer_149 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* ( function_statement ) */
fn reducer_150 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* ( function_statement )* */
fn reducer_151 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* function_statement */
fn reducer_152 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* ( function_statement ) */
fn reducer_153 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* ( function_statement )* */
fn reducer_154 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, __tok_rng_1, _) = slots.take(1);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_1;
  let obj_1_0 = ref_1;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_1_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* "(" tuple_ele(+",") ")" 
                                    :ast { t_ExprTuple, expressions: $2 } */
fn reducer_155 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let AstSlot (ref_1, _, _) = slots.take(1);
  let AstSlot (_, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_1_0 = ref_1.into_nodes();
  let var_4_0 = ExprTuple::new(
    obj_1_0,
  );
  slots.assign(0, AstSlot(ASTNode::ExprTuple(Box::new(var_4_0)), __rule_rng__, TokenRange::default()));
}


/* tuple_ele
                                    :ast { t_ExprTuple, expressions: [$1] } */
fn reducer_156 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  let var_3_0 = ExprTuple::new(
    obj_2_0,
  );
  slots.assign(0, AstSlot(ASTNode::ExprTuple(Box::new(var_3_0)), __rule_rng__, TokenRange::default()));
}


/* tuple_ele */
fn reducer_157 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let obj_0_0 = ref_0;
  let mut obj_2_0 = vec![];
  obj_2_0.push(obj_0_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_2_0), __rule_rng__, TokenRange::default()));
}


/* tuple_ele(+",") */
fn reducer_158 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  slots.take(1);
  let AstSlot (ref_2, __tok_rng_2, _) = slots.take(2);
  let __rule_rng__ = __tok_rng_0 + __tok_rng_2;
  let obj_2_0 = ref_2;
  let mut obj_0_0 = ref_0.into_nodes();
  obj_0_0.push(obj_2_0);
  slots.assign(0, AstSlot(ASTNode::NODES(obj_0_0), __rule_rng__, TokenRange::default()));
}


/* expr */
fn reducer_159 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (ref_0, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  slots.assign(0, AstSlot(ref_0, __rule_rng__, TokenRange::default()));
}


/* "_"                             :ast { t_Ignore } */
fn reducer_160 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = Ignore::new();
  slots.assign(0, AstSlot(ASTNode::Ignore(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "*"                             :ast { t_Ignore } */
fn reducer_161 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = Ignore::new();
  slots.assign(0, AstSlot(ASTNode::Ignore(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "true"                          :ast { t_True } */
fn reducer_162 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = True::new();
  slots.assign(0, AstSlot(ASTNode::True(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "false"                         :ast { t_False } */
fn reducer_163 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = False::new();
  slots.assign(0, AstSlot(ASTNode::False(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "Some"                          :ast { t_NotNone } */
fn reducer_164 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = NotNone::new();
  slots.assign(0, AstSlot(ASTNode::NotNone(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "&"                             :ast { t_NotNone } */
fn reducer_165 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = NotNone::new();
  slots.assign(0, AstSlot(ASTNode::NotNone(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}


/* "None"                          :ast { t_None } */
fn reducer_166 <R: Reader + UTF8Reader, M, const UP: bool> (_ctx_: *mut ParseContext<R, M>,slots: &AstStackSlice<AstSlot<ASTNode>, UP>) {
  let AstSlot (_, __tok_rng_0, _) = slots.take(0);
  let __rule_rng__ = __tok_rng_0;
  let var_2_0 = None::new();
  slots.assign(0, AstSlot(ASTNode::None(Box::new(var_2_0)), __rule_rng__, TokenRange::default()));
}

struct ReduceFunctions<R: Reader + UTF8Reader, M, const UP: bool>(
  pub [Reducer<R, M, ASTNode, UP>; 167]
);

impl<R: Reader + UTF8Reader, M, const UP: bool> ReduceFunctions<R, M, UP>{
  
  pub const fn new ()-> Self {
    
    Self([
      reducer_000::<R, M, UP>,
      reducer_001::<R, M, UP>,
      reducer_002::<R, M, UP>,
      reducer_003::<R, M, UP>,
      reducer_004::<R, M, UP>,
      reducer_005::<R, M, UP>,
      reducer_006::<R, M, UP>,
      reducer_007::<R, M, UP>,
      reducer_008::<R, M, UP>,
      reducer_009::<R, M, UP>,
      reducer_010::<R, M, UP>,
      reducer_011::<R, M, UP>,
      reducer_012::<R, M, UP>,
      reducer_013::<R, M, UP>,
      reducer_014::<R, M, UP>,
      reducer_015::<R, M, UP>,
      reducer_016::<R, M, UP>,
      reducer_017::<R, M, UP>,
      reducer_018::<R, M, UP>,
      reducer_019::<R, M, UP>,
      reducer_020::<R, M, UP>,
      reducer_021::<R, M, UP>,
      reducer_022::<R, M, UP>,
      reducer_023::<R, M, UP>,
      reducer_024::<R, M, UP>,
      reducer_025::<R, M, UP>,
      reducer_026::<R, M, UP>,
      reducer_027::<R, M, UP>,
      reducer_028::<R, M, UP>,
      reducer_029::<R, M, UP>,
      reducer_030::<R, M, UP>,
      reducer_031::<R, M, UP>,
      reducer_032::<R, M, UP>,
      reducer_033::<R, M, UP>,
      reducer_034::<R, M, UP>,
      reducer_035::<R, M, UP>,
      reducer_036::<R, M, UP>,
      reducer_037::<R, M, UP>,
      reducer_038::<R, M, UP>,
      reducer_039::<R, M, UP>,
      reducer_040::<R, M, UP>,
      reducer_041::<R, M, UP>,
      reducer_042::<R, M, UP>,
      reducer_043::<R, M, UP>,
      reducer_044::<R, M, UP>,
      reducer_045::<R, M, UP>,
      reducer_046::<R, M, UP>,
      reducer_047::<R, M, UP>,
      reducer_048::<R, M, UP>,
      reducer_049::<R, M, UP>,
      reducer_050::<R, M, UP>,
      reducer_051::<R, M, UP>,
      reducer_052::<R, M, UP>,
      reducer_053::<R, M, UP>,
      reducer_054::<R, M, UP>,
      reducer_055::<R, M, UP>,
      reducer_056::<R, M, UP>,
      reducer_057::<R, M, UP>,
      reducer_058::<R, M, UP>,
      reducer_059::<R, M, UP>,
      reducer_060::<R, M, UP>,
      reducer_061::<R, M, UP>,
      reducer_062::<R, M, UP>,
      reducer_063::<R, M, UP>,
      reducer_064::<R, M, UP>,
      reducer_065::<R, M, UP>,
      reducer_066::<R, M, UP>,
      reducer_067::<R, M, UP>,
      reducer_068::<R, M, UP>,
      reducer_069::<R, M, UP>,
      reducer_070::<R, M, UP>,
      reducer_071::<R, M, UP>,
      reducer_072::<R, M, UP>,
      reducer_073::<R, M, UP>,
      reducer_074::<R, M, UP>,
      reducer_075::<R, M, UP>,
      reducer_076::<R, M, UP>,
      reducer_077::<R, M, UP>,
      reducer_078::<R, M, UP>,
      reducer_079::<R, M, UP>,
      reducer_080::<R, M, UP>,
      reducer_081::<R, M, UP>,
      reducer_082::<R, M, UP>,
      reducer_083::<R, M, UP>,
      reducer_084::<R, M, UP>,
      reducer_085::<R, M, UP>,
      reducer_086::<R, M, UP>,
      reducer_087::<R, M, UP>,
      reducer_088::<R, M, UP>,
      reducer_089::<R, M, UP>,
      reducer_090::<R, M, UP>,
      reducer_091::<R, M, UP>,
      reducer_092::<R, M, UP>,
      reducer_093::<R, M, UP>,
      reducer_094::<R, M, UP>,
      reducer_095::<R, M, UP>,
      reducer_096::<R, M, UP>,
      reducer_097::<R, M, UP>,
      reducer_098::<R, M, UP>,
      reducer_099::<R, M, UP>,
      reducer_100::<R, M, UP>,
      reducer_101::<R, M, UP>,
      reducer_102::<R, M, UP>,
      reducer_103::<R, M, UP>,
      reducer_104::<R, M, UP>,
      reducer_105::<R, M, UP>,
      reducer_106::<R, M, UP>,
      reducer_107::<R, M, UP>,
      reducer_108::<R, M, UP>,
      reducer_109::<R, M, UP>,
      reducer_110::<R, M, UP>,
      reducer_111::<R, M, UP>,
      reducer_112::<R, M, UP>,
      reducer_113::<R, M, UP>,
      reducer_114::<R, M, UP>,
      reducer_115::<R, M, UP>,
      reducer_116::<R, M, UP>,
      reducer_117::<R, M, UP>,
      reducer_118::<R, M, UP>,
      reducer_119::<R, M, UP>,
      reducer_120::<R, M, UP>,
      reducer_121::<R, M, UP>,
      reducer_122::<R, M, UP>,
      reducer_123::<R, M, UP>,
      reducer_124::<R, M, UP>,
      reducer_125::<R, M, UP>,
      reducer_126::<R, M, UP>,
      reducer_127::<R, M, UP>,
      reducer_128::<R, M, UP>,
      reducer_129::<R, M, UP>,
      reducer_130::<R, M, UP>,
      reducer_131::<R, M, UP>,
      reducer_132::<R, M, UP>,
      reducer_133::<R, M, UP>,
      reducer_134::<R, M, UP>,
      reducer_135::<R, M, UP>,
      reducer_136::<R, M, UP>,
      reducer_137::<R, M, UP>,
      reducer_138::<R, M, UP>,
      reducer_139::<R, M, UP>,
      reducer_140::<R, M, UP>,
      reducer_141::<R, M, UP>,
      reducer_142::<R, M, UP>,
      reducer_143::<R, M, UP>,
      reducer_144::<R, M, UP>,
      reducer_145::<R, M, UP>,
      reducer_146::<R, M, UP>,
      reducer_147::<R, M, UP>,
      reducer_148::<R, M, UP>,
      reducer_149::<R, M, UP>,
      reducer_150::<R, M, UP>,
      reducer_151::<R, M, UP>,
      reducer_152::<R, M, UP>,
      reducer_153::<R, M, UP>,
      reducer_154::<R, M, UP>,
      reducer_155::<R, M, UP>,
      reducer_156::<R, M, UP>,
      reducer_157::<R, M, UP>,
      reducer_158::<R, M, UP>,
      reducer_159::<R, M, UP>,
      reducer_160::<R, M, UP>,
      reducer_161::<R, M, UP>,
      reducer_162::<R, M, UP>,
      reducer_163::<R, M, UP>,
      reducer_164::<R, M, UP>,
      reducer_165::<R, M, UP>,
      reducer_166::<R, M, UP>,
    ])
  }
}
    
pub trait Reader: ByteReader + MutByteReader + UTF8Reader {}

impl<T: ByteReader + MutByteReader + UTF8Reader> Reader for T {}

pub type Parser<T, UserCTX, Bytecode> = sherpa_rust_runtime::deprecate::ByteCodeParser<T, UserCTX, Bytecode>;

pub mod meta{
  
  pub const nonterm_names: [&'static str;58] = [
    "ascript_form",
    "ascript_form_list",
    "script_statement",
    "text",
    "text_group",
    "assignment",
    "function",
    "function_list",
    "function_group_1",
    "function_list_2",
    "format",
    "call",
    "object",
    "object_list",
    "literal_space",
    "literal_space_list",
    "block_template_5912456036338616932",
    "block_template_5912456036338616932_list",
    "block_template_5912456036338616932_list_1",
    "block_template_5912456036338616932_list_2",
    "num",
    "at_string_literal",
    "at_string_literal_list",
    "expression",
    "binding_id",
    "expr",
    "param",
    "function_statement",
    "fn_name",
    "args",
    "args_list",
    "object_accessor",
    "object_accessor_list",
    "id",
    "literal_space_statement",
    "plain_string_literal",
    "plain_string_literal_list",
    "type",
    "match",
    "match_list",
    "block_template_7571693507362039102",
    "block_template_7571693507362039102_list",
    "block_template_7571693507362039102_list_1",
    "block_template_7571693507362039102_list_2",
    "non_binding_id",
    "literal_format",
    "block_template_9874968537493629707",
    "block_template_9874968537493629707_list",
    "block_template_9874968537493629707_list_1",
    "block_template_9874968537493629707_list_2",
    "match_arm",
    "match_arm_group",
    "match_arm_list_1",
    "match_arm_group_2",
    "match_arm_list_3",
    "tuple",
    "tuple_list",
    "tuple_ele",
  ];
  
  pub const symbol_string: [&'static str;58] = [
    r####"Default"####,
    r####"c:sp"####,
    r####"c:nl"####,
    r####"c:sym"####,
    r####" . "####,
    r####"tk:nonterm"####,
    r####" } "####,
    r####" ={ "####,
    r####" { "####,
    r####" @ "####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####" @{ "####,
    r####" ( "####,
    r####" ) "####,
    r####" [ "####,
    r####" ] "####,
    r####"tk:nonterm"####,
    r####" " "####,
    r####" @" "####,
    r####" @[ "####,
    r####"tk:nonterm"####,
    r####" * "####,
    r####" + "####,
    r####" - "####,
    r####" / "####,
    r####" ^ "####,
    r####"nonterm"####,
    r####" : "####,
    r####"tk:nonterm"####,
    r####" , "####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####" obj "####,
    r####" num "####,
    r####" str "####,
    r####" flt "####,
    r####" int "####,
    r####" match "####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####"tk:nonterm"####,
    r####" & "####,
    r####" _ "####,
    r####" Some "####,
    r####" None "####,
    r####" true "####,
    r####" false "####,
  ];
}

pub fn new_default_parser<'a, T: Reader, UserCTX> (reader: &'a mut T)-> Parser<T, UserCTX, &'static [u8]> {
  let mut parser = Parser::<T, UserCTX, &'static [u8]>::new(reader, &bytecode);
  parser.init_parser(8);
  parser
}

pub static bytecode: [u8; 90018] = [
  0,211,200,197,210,208,193,2,15,1,151,41,1,0,17,1,21,0,0,0,1,21,1,7,3,0,0,152,41,1,0,21,0,0,0,4,0,0,0,16,232,9,133,1,48,67,129,2,48,3,128,3,240,86,131,4,168,21,128,5, 
  96,20,128,17,24,73,131,23,72,7,128,8,144,83,130,9,192,82,130,10,120,17,128,11,48,16,128,12,232,14,128,13,72,13,128,14,0,12,128,15,184,10,128,19,72,8,128,24,72,6,128,25,128,5,128,33,184,4,128,48, 
  56,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,22,34,1,0,15,1,127,41,1,0,15,1,139,41,1,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0, 
  0,0,1,0,15,1,22,34,1,0,17,1,249,31,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,22,34,1,0,17,1,198,31,1,0,1,4,15,1,22,34,1,0,15,1,127,41,1,0,15,1,174,31,1, 
  0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,22,34,1,0,15,1,127,41,1,0,15,1,174,31,1,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,22,34,1,0,15,1,127,41,1,0,15, 
  1,0,28,1,0,17,1,23,16,1,0,1,4,15,1,22,34,1,0,15,1,127,41,1,0,15,1,0,28,1,0,17,1,46,4,1,0,1,4,15,1,22,34,1,0,15,1,127,41,1,0,15,1,34,4,1,0,17,1,35, 
  220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,1,0,0,0,1,0,0,0,1,0,17,1,22,34,1,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,2,0, 
  0,0,6,0,0,0,1,0,19,1,0,0,0,1,0,0,0,1,0,17,1,22,34,1,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0, 
  19,1,0,0,0,1,0,0,0,1,0,17,1,22,34,1,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,1,0,0,0,1,0,0,0,1,0,17,1,22,34,1,0,1,4, 
  19,10,0,0,0,33,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,1,0,0,0,1,0,0,0,1,0,17,1,22,34,1,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,2,0,0,0,6,0,0, 
  0,1,0,19,1,0,0,0,1,0,0,0,1,0,17,1,22,34,1,0,1,4,15,1,22,34,1,0,15,1,127,41,1,0,15,1,23,220,0,0,17,1,234,219,0,0,1,4,15,1,22,34,1,0,15,1,127,41,1,0, 
  15,1,0,28,1,0,17,1,29,3,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,1,0,0,0,1,0,0,0,1,0,17,1,22,34,1,0,1,4,19,3,0,0,0, 
  12,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,1,0,0,0,1,0,0,0,1,0,17,1,22,34,1,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,1, 
  0,0,0,1,0,0,0,1,0,17,1,22,34,1,0,1,2,21,1,24,3,0,0,110,44,1,0,22,0,0,0,4,0,0,0,16,8,74,133,1,80,195,131,2,80,3,128,3,120,151,131,4,48,22,128,5,232,20,128,6, 
  128,20,128,23,104,7,128,8,176,147,130,9,224,146,130,10,152,17,128,11,80,16,128,12,8,15,128,13,104,13,128,14,32,12,128,15,216,10,128,17,56,9,129,19,104,8,128,24,104,6,128,25,160,5,128,33,216,4,128,48, 
  88,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,55,206,0,0,15,1,222,219,0,0,15,1,139,41,1,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0, 
  0,0,1,0,15,1,55,206,0,0,17,1,36,85,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,55,206,0,0,17,1,54,6,0,0,1,4,15,1,55,206,0,0,15,1,222,219,0,0,15,1,174,31,1, 
  0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,55,206,0,0,15,1,222,219,0,0,15,1,174,31,1,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,55,206,0,0,15,1,222,219,0,0,15, 
  1,0,28,1,0,17,1,23,16,1,0,1,4,15,1,55,206,0,0,15,1,222,219,0,0,15,1,0,28,1,0,17,1,46,4,1,0,1,4,15,1,55,206,0,0,15,1,222,219,0,0,15,1,34,4,1,0,17,1,35, 
  220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,19,0,0,0,56,0,0,0,1,0,17,1,55,206,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,2,0, 
  0,0,6,0,0,0,1,0,19,19,0,0,0,56,0,0,0,1,0,17,1,55,206,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0, 
  19,19,0,0,0,56,0,0,0,1,0,17,1,55,206,0,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,19,0,0,0,56,0,0,0,1,0,17,1,55,206,0,0,1,4, 
  19,10,0,0,0,33,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,19,0,0,0,56,0,0,0,1,0,17,1,55,206,0,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,2,0,0,0,6,0,0, 
  0,1,0,19,19,0,0,0,56,0,0,0,1,0,17,1,55,206,0,0,1,4,15,1,55,206,0,0,15,1,222,219,0,0,15,1,23,220,0,0,17,1,234,219,0,0,1,4,15,1,55,206,0,0,15,1,222,219,0,0, 
  15,1,0,28,1,0,17,1,29,3,0,0,1,4,19,16,0,0,0,51,0,0,0,2,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,19,0,0,0,56,0,0,0,1,0, 
  17,1,55,206,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,19,0,0,0,56,0,0,0,1,0,17,1,55,206,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0, 
  19,2,0,0,0,3,0,0,0,1,0,19,19,0,0,0,56,0,0,0,1,0,17,1,55,206,0,0,1,2,21,1,39,0,0,0,58,45,1,0,3,0,0,0,1,0,0,0,2,48,1,128,1,48,65,128,7,240,0,128, 
  4,17,1,105,6,0,0,1,8,19,33,0,0,0,97,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22, 
  184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,120,69,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0, 
  1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,120,69,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,120,69,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,120,69,0,0,15, 
  1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,120,69,0,0,1,2,21,1,50,0,0,0,52,50,1,0,3,0,0,0,1,0,0, 
  0,2,240,0,128,1,240,64,128,21,248,0,128,8,4,19,36,0,0,0,106,0,0,0,1,0,17,1,102,7,0,0,1,2,21,0,40,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,36,208,0,128,35,56,1,128, 
  15,1,102,7,0,0,17,1,143,7,0,0,1,1,2,21,1,62,0,0,0,146,50,1,0,4,0,0,0,2,0,0,0,21,136,1,128,1,16,193,127,2,16,65,128,22,24,1,128,8,4,19,35,0,0,0,105,0,0,0, 
  3,0,14,1,4,19,36,0,0,0,107,0,0,0,2,0,1,2,19,25,0,0,0,72,0,0,0,1,0,1,19,25,0,0,0,71,0,0,0,1,0,1,21,1,235,0,0,0,237,50,1,0,9,0,0,0,3,0,0,0, 
  40,144,3,128,1,16,198,129,2,16,6,128,35,160,2,128,36,24,6,128,37,112,5,128,38,128,4,128,39,176,1,128,41,184,6,128,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17, 
  1,107,68,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,107,68,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17, 
  1,107,68,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,107,68,0,0,1,4,15,1,107,68,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,8,4,15,1, 
  107,68,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,15,1,107,68,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,56,0,0,0,42,54,1,0,3, 
  0,0,0,1,0,0,0,2,240,0,128,1,240,64,128,33,248,0,128,8,4,19,28,0,0,0,82,0,0,0,1,0,15,1,133,44,0,0,17,1,22,9,0,0,1,2,21,7,36,0,0,0,255,255,255,255,1,0,0,0, 
  0,0,0,0,40,176,0,128,4,15,1,121,44,0,0,17,1,86,9,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,2,21,1,218,0,0,0,180,54,1,0, 
  8,0,0,0,3,0,0,0,48,152,1,128,1,144,193,128,2,144,65,128,18,120,5,128,25,232,2,128,13,224,5,128,22,216,4,128,23,56,4,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0, 
  0,0,1,0,15,1,49,10,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,49,10,0,0,15,1,24,85,0,0,17,1, 
  230,7,0,0,1,4,15,1,49,10,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,49,10,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,29,0,0,0,84,0,0,0,2,0,1,4,19, 
  20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,49,10,0,0,1,2,21,0,206,0,0,0,255,255,255,255,10,0,0,0,3,0,0,0,24,48,4,128,25,200,195,129,20,80,197,129,35, 
  96,2,128,12,224,133,127,21,192,132,128,30,88,3,128,29,192,3,128,33,240,2,128,44,208,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,49,10,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,49,10, 
  0,0,1,15,1,49,10,0,0,17,1,130,43,0,0,1,15,1,49,10,0,0,17,1,65,40,0,0,1,1,15,1,49,10,0,0,17,1,0,11,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,49,10,0,0, 
  1,19,25,0,0,0,71,0,0,0,1,0,17,1,49,10,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,49,10,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,49,10,0,0,1,2,21,7,78,0, 
  0,0,255,255,255,255,5,0,0,0,2,0,0,0,47,112,1,128,45,240,1,128,42,48,129,128,43,176,65,127,94,48,2,128,4,17,1,199,37,0,0,1,4,17,1,77,35,0,0,1,4,17,1,211,32,0,0,1,4,17, 
  1,213,18,0,0,1,4,17,1,117,11,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,30,0,0,0,85,0,0,0,1,0,1,21,1,201,0,0,0,108, 
  49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0, 
  0,1,0,15,1,63,12,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,63,12,0,0,15,1,24,85,0,0,17,1,230, 
  7,0,0,1,4,15,1,63,12,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,63,12,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0, 
  0,69,0,0,0,1,0,17,1,63,12,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2, 
  128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,63,12,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,63,12,0,0,1,15,1,63,12,0,0,17,1,209,16,0,0,1,15,1,63,12,0,0, 
  17,1,248,12,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,63,12,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,63,12,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,63,12,0,0, 
  1,19,25,0,0,0,70,0,0,0,1,0,17,1,63,12,0,0,1,2,21,7,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,94,176,0,128,4,17,1,62,13,0,0,1,21,9,27,0,0,0,255,255,255,255, 
  2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,25,0,0,0,67,0,0,0,3,0,14,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128, 
  23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,8,14,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4, 
  19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,8,14,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,8,14,0,0,15,1,218,7,0,0,17,1,12,28,1,0, 
  1,4,15,1,8,14,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,8,14,0,0,1,2,21,0,184,0,0,0,255,255, 
  255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,8,14,0,0,1, 
  19,25,0,0,0,72,0,0,0,1,0,17,1,8,14,0,0,1,15,1,8,14,0,0,17,1,193,14,0,0,1,15,1,8,14,0,0,17,1,248,12,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,8,14,0, 
  0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,8,14,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,8,14,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,8,14,0,0,1,2,21,1,235, 
  0,0,0,59,55,1,0,9,0,0,0,3,0,0,0,40,192,5,128,1,80,199,129,2,80,7,128,35,176,1,128,36,160,2,128,37,32,5,128,38,48,4,128,39,64,3,128,41,176,6,128,4,19,31,0,0,0,87,0,0, 
  0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0, 
  0,0,1,0,17,1,184,15,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,251,60,0,0, 
  1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,8,19,12,0,0,0,40,0,0, 
  0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,15,1,184,15,0,0,17,1, 
  247,15,0,0,1,1,2,21,1,193,0,0,0,59,55,1,0,9,0,0,0,3,0,0,0,40,72,5,128,1,16,196,129,2,16,4,128,35,80,3,128,36,224,2,128,37,176,1,128,38,32,2,128,39,136,4,128,41,24,4, 
  128,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,87, 
  0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0, 
  0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,19,13,0,0,0,42,0,0,0,2,0,1,21,1,235,0,0,0,75,56,1,0,9,0,0,0,3, 
  0,0,0,40,224,3,128,1,80,199,129,2,80,7,128,35,96,6,128,36,64,3,128,37,192,5,128,38,208,4,128,39,176,1,128,41,160,2,128,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0, 
  1,0,17,1,200,17,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,93,0,0,0,1, 
  0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,15,1,200,17,0,0,15,1,95,68, 
  0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255, 
  255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,15,1,200,17,0,0,17,1,7,18,0,0,1,1,2,21,1,193,0,0, 
  0,75,56,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1,64,197,129,2,64,5,128,35,128,4,128,36,112,2,128,37,16,4,128,38,72,5,128,39,80,3,128,41,224,2,128,4,19,31,0,0,0,93,0,0,0,1, 
  0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,145,44,0,0,1,4,15,1,197,16,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0, 
  0,0,2,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0, 
  0,42,0,0,0,2,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22, 
  184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,159,19,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0, 
  1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,159,19,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,159,19,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,159,19,0,0,15, 
  1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,159,19,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0, 
  0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,159,19,0,0,1,19,25,0,0,0,72,0,0,0, 
  1,0,17,1,159,19,0,0,1,15,1,159,19,0,0,17,1,220,31,0,0,1,15,1,159,19,0,0,17,1,88,20,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,159,19,0,0,1,19,25,0,0,0,71,0, 
  0,0,1,0,17,1,159,19,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,159,19,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,159,19,0,0,1,2,21,7,78,0,0,0,255,255,255,255,5,0, 
  0,0,2,0,0,0,47,48,2,128,45,176,1,128,42,240,129,128,43,112,65,127,94,48,1,128,4,17,1,62,13,0,0,1,4,17,1,236,28,0,0,1,4,17,1,114,26,0,0,1,4,17,1,166,23,0,0,1,4,17, 
  1,206,20,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,25,0,0,0,65,0,0,0,3,0,14,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0, 
  2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,152,21, 
  0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,152,21,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1, 
  152,21,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,152,21,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0, 
  17,1,152,21,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33, 
  0,0,0,98,0,0,0,1,0,17,1,152,21,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,152,21,0,0,1,15,1,152,21,0,0,17,1,175,22,0,0,1,15,1,152,21,0,0,17,1,81,22,0,0,1, 
  19,33,0,0,0,97,0,0,0,1,0,17,1,152,21,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,152,21,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,152,21,0,0,1,19,25,0,0,0,70, 
  0,0,0,1,0,17,1,152,21,0,0,1,2,21,7,54,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,42,48,129,128,47,240,0,128,94,112,1,128,4,17,1,206,20,0,0,1,4,17,1,166,23,0,0,1,4, 
  17,1,62,13,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,25,0,0,0,68,0,0,0,3,0,14,1,21,1,235,0,0,0,59,55,1,0,9,0,0, 
  0,3,0,0,0,40,104,6,128,1,96,198,129,2,96,6,128,35,80,2,128,36,64,3,128,37,208,4,128,38,112,5,128,39,224,3,128,41,176,1,128,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,221,8,0,0, 
  1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,92,0,0, 
  0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0, 
  0,0,1,0,17,1,184,15,0,0,1,8,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0, 
  108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0, 
  0,0,1,0,15,1,112,24,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,112,24,0,0,15,1,24,85,0,0,17,1, 
  230,7,0,0,1,4,15,1,112,24,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,112,24,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0, 
  0,0,69,0,0,0,1,0,17,1,112,24,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176, 
  2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,112,24,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,112,24,0,0,1,15,1,112,24,0,0,17,1,123,25,0,0,1,15,1,112,24,0, 
  0,17,1,41,25,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,112,24,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,112,24,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,112,24,0, 
  0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,112,24,0,0,1,2,21,7,42,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,42,16,65,128,94,208,0,128,4,17,1,62,13,0,0,1,4,17,1,166,23, 
  0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,25,0,0,0,66,0,0,0,3,0,14,1,21,1,235,0,0,0,59,55,1,0,9,0,0,0,3,0,0, 
  0,40,224,3,128,1,176,198,129,2,176,6,128,35,208,4,128,36,64,3,128,37,160,2,128,38,192,5,128,39,176,1,128,41,184,6,128,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0, 
  17,1,184,15,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19, 
  13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19, 
  13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,8,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0, 
  7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0, 
  15,1,60,27,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,60,27,0,0,15,1,24,85,0,0,17,1,230,7,0,0, 
  1,4,15,1,60,27,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,60,27,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0, 
  0,0,1,0,17,1,60,27,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144, 
  1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,60,27,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,60,27,0,0,1,15,1,60,27,0,0,17,1,245,27,0,0,1,15,1,60,27,0,0,17,1,88, 
  20,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,60,27,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,60,27,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,60,27,0,0,1,19,25, 
  0,0,0,70,0,0,0,1,0,17,1,60,27,0,0,1,2,21,1,235,0,0,0,59,55,1,0,9,0,0,0,3,0,0,0,40,96,6,128,1,80,199,129,2,80,7,128,35,176,1,128,36,144,3,128,37,32,5,128,38, 
  160,2,128,39,48,4,128,41,192,5,128,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0, 
  0,0,1,0,17,1,184,15,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0, 
  1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0, 
  0,0,1,0,17,1,184,15,0,0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88, 
  133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,182,29,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63, 
  0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,182,29,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,182,29,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,182,29, 
  0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,182,29,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0, 
  3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,182,29,0,0,1,19,25,0,0,0,72, 
  0,0,0,1,0,17,1,182,29,0,0,1,15,1,182,29,0,0,17,1,229,30,0,0,1,15,1,182,29,0,0,17,1,111,30,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,182,29,0,0,1,19,25,0,0, 
  0,71,0,0,0,1,0,17,1,182,29,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,182,29,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,182,29,0,0,1,2,21,7,78,0,0,0,255,255,255, 
  255,5,0,0,0,2,0,0,0,47,240,1,128,45,112,1,128,42,48,130,128,43,48,65,127,94,176,1,128,4,17,1,236,28,0,0,1,4,17,1,114,26,0,0,1,4,17,1,62,13,0,0,1,4,17,1,206,20,0,0, 
  1,4,17,1,166,23,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,25,0,0,0,64,0,0,0,3,0,14,1,21,1,235,0,0,0,59,55,1,0,9, 
  0,0,0,3,0,0,0,40,120,5,128,1,112,197,129,2,112,5,128,35,104,6,128,36,224,3,128,37,64,3,128,38,128,4,128,39,176,1,128,41,160,2,128,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0, 
  41,0,0,0,1,0,17,1,184,15,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,184,15,0,0, 
  15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,8,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0, 
  0,41,0,0,0,1,0,17,1,184,15,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,235,0, 
  0,0,75,56,1,0,9,0,0,0,3,0,0,0,40,192,5,128,1,176,198,129,2,176,6,128,35,160,2,128,36,32,5,128,37,128,4,128,38,144,3,128,39,176,1,128,41,184,6,128,4,19,31,0,0,0,92,0,0,0, 
  1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,19,31,0,0,0,91,0,0,0, 
  1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1, 
  4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,8,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,19,12,0,0,0,40,0,0,0, 
  1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0, 
  1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,157,33,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,157,33,0, 
  0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,157,33,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,157,33,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0, 
  58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,157,33,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12, 
  48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,157,33,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,157,33,0,0,1,15,1,157,33,0,0,17,1,86, 
  34,0,0,1,15,1,157,33,0,0,17,1,111,30,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,157,33,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,157,33,0,0,1,19,25,0,0,0,69,0, 
  0,0,1,0,17,1,157,33,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,157,33,0,0,1,2,21,1,235,0,0,0,75,56,1,0,9,0,0,0,3,0,0,0,40,32,5,128,1,80,199,129,2,80,7,128, 
  35,48,4,128,36,16,6,128,37,176,6,128,38,176,1,128,39,160,2,128,41,144,3,128,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,19,31,0,0,0, 
  92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0, 
  0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,145, 
  44,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1, 
  112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,23,36,0,0,15,1,24,85,0,0,17, 
  1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,23,36,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,23,36,0,0,15,1,218,7,0, 
  0,17,1,12,28,1,0,1,4,15,1,23,36,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,23,36,0,0,1,2,21, 
  0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0, 
  17,1,23,36,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,23,36,0,0,1,15,1,23,36,0,0,17,1,208,36,0,0,1,15,1,23,36,0,0,17,1,81,22,0,0,1,19,33,0,0,0,97,0,0,0, 
  1,0,17,1,23,36,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,23,36,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,23,36,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,23,36, 
  0,0,1,2,21,1,235,0,0,0,75,56,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1,96,198,129,2,96,6,128,35,208,4,128,36,192,5,128,37,48,4,128,38,104,6,128,39,160,2,128,41,144,3,128,4,19, 
  31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,15, 
  1,200,17,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1, 
  0,17,1,200,17,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,8,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,19, 
  12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19, 
  44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,145,38,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0, 
  0,1,0,15,1,145,38,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,145,38,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,145,38,0,0,15,1,206,7,0,0,17,1,51,7,0, 
  0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,145,38,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20, 
  160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,145,38,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,145,38,0,0,1,15, 
  1,145,38,0,0,17,1,74,39,0,0,1,15,1,145,38,0,0,17,1,41,25,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,145,38,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,145,38,0,0, 
  1,19,25,0,0,0,69,0,0,0,1,0,17,1,145,38,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,145,38,0,0,1,2,21,1,235,0,0,0,75,56,1,0,9,0,0,0,3,0,0,0,40,160,2,128, 
  1,192,197,129,2,192,5,128,35,144,3,128,36,184,6,128,37,128,4,128,38,200,5,128,39,176,1,128,41,32,5,128,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0, 
  0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0, 
  0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,8,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0, 
  41,0,0,0,1,0,17,1,200,17,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,7,48,0,0,0,255,255,255,255,2,0,0,0,1, 
  0,0,0,44,208,0,128,41,16,1,128,4,17,1,141,40,0,0,1,4,19,29,0,0,0,83,0,0,0,3,0,14,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8, 
  2,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0, 
  19,33,0,0,0,98,0,0,0,1,0,15,1,87,41,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,87,41,0,0,15, 
  1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,87,41,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,87,41,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0, 
  0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,87,41,0,0,1,2,21,0,189,0,0,0,255,255,255,255,9,0,0,0,3,0,0,0,24,168,3,128,25,64,131,129,20,200,132,129,35,64,2,128,12,88,133, 
  127,21,56,4,128,30,56,3,128,33,208,2,128,44,176,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,87,41,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,87,41,0,0,1,15,1,87,41,0,0,17, 
  1,139,42,0,0,1,1,15,1,87,41,0,0,17,1,21,42,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,87,41,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,87,41,0,0,1,19,25,0,0, 
  0,69,0,0,0,1,0,17,1,87,41,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,87,41,0,0,1,2,21,7,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,47,240,1,128,45,176,1,128,42, 
  48,130,128,43,48,65,127,94,112,1,128,4,17,1,211,32,0,0,1,4,17,1,117,11,0,0,1,4,17,1,213,18,0,0,1,4,17,1,77,35,0,0,1,4,17,1,199,37,0,0,1,21,9,27,0,0,0,255,255,255, 
  255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,30,0,0,0,86,0,0,0,3,0,14,1,21,1,235,0,0,0,75,56,1,0,9,0,0,0,3,0,0,0,40,208,4,128,1,192,197,129,2,192,5, 
  128,35,224,3,128,36,200,5,128,37,64,3,128,38,176,1,128,39,104,6,128,41,160,2,128,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,15,1,200,17, 
  0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1, 
  200,17,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,8,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0, 
  0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,235,0,0,0,75,56,1,0,9,0,0,0,3,0,0,0,40,104,6,128, 
  1,96,198,129,2,96,6,128,35,128,4,128,36,80,2,128,37,224,3,128,38,240,2,128,39,112,5,128,41,176,1,128,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,200,17,0,0,15, 
  1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,251,60,0, 
  0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0, 
  0,1,8,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,19,11,0,0,0,38,0,0,0,2,0,1,19,31,0, 
  0,0,94,0,0,0,2,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0, 
  0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,91,45,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1, 
  0,15,1,91,45,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,91,45,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,91,45,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1, 
  4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,91,45,0,0,1,2,21,0,189,0,0,0,255,255,255,255,9,0,0,0,3,0,0,0,24,168,3,128,25,64,67,129,20,200,132, 
  129,35,64,2,128,12,88,133,127,21,56,4,128,33,208,2,128,31,56,3,128,44,176,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,91,45,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,91,45,0,0, 
  1,15,1,91,45,0,0,17,1,4,60,0,0,1,1,15,1,91,45,0,0,17,1,25,46,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,91,45,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,91, 
  45,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,91,45,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,91,45,0,0,1,2,21,7,96,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,47, 
  144,1,128,45,208,193,128,42,80,194,128,43,16,66,127,93,144,2,128,94,80,1,128,4,17,1,138,57,0,0,1,4,17,1,16,55,0,0,1,4,17,1,150,52,0,0,1,4,17,1,28,50,0,0,1,4,17,1,149,46, 
  0,0,1,4,19,31,0,0,0,88,0,0,0,3,0,14,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,2,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2, 
  0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,95,47,0, 
  0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,95,47,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,95, 
  47,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,95,47,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17, 
  1,95,47,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0, 
  0,0,98,0,0,0,1,0,17,1,95,47,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,95,47,0,0,1,15,1,95,47,0,0,17,1,24,48,0,0,1,15,1,95,47,0,0,17,1,41,25,0,0,1,19, 
  33,0,0,0,97,0,0,0,1,0,17,1,95,47,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,95,47,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,95,47,0,0,1,19,25,0,0,0,70,0, 
  0,0,1,0,17,1,95,47,0,0,1,2,21,1,235,0,0,0,225,56,1,0,9,0,0,0,3,0,0,0,40,144,3,128,1,192,197,129,2,192,5,128,35,160,2,128,36,184,6,128,37,32,5,128,38,176,1,128,39,200, 
  5,128,41,128,4,128,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17, 
  1,15,49,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,15, 
  49,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,8,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0, 
  17,1,145,44,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0, 
  17,1,15,49,0,0,1,15,1,15,49,0,0,17,1,78,49,0,0,1,1,2,21,1,193,0,0,0,225,56,1,0,9,0,0,0,3,0,0,0,40,128,4,128,1,64,197,129,2,64,5,128,35,72,5,128,36,112,2,128, 
  37,16,4,128,38,176,1,128,39,80,3,128,41,224,2,128,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,145,44,0,0,1,4,15,1,197,16,0, 
  0,17,1,221,8,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13, 
  0,0,0,42,0,0,0,2,0,1,8,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,21,1,201,0,0,0,108,49,1,0,7,0, 
  0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1, 
  230,50,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,230,50,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4, 
  15,1,230,50,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,230,50,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0, 
  1,0,17,1,230,50,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128, 
  19,33,0,0,0,98,0,0,0,1,0,17,1,230,50,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,230,50,0,0,1,15,1,230,50,0,0,17,1,159,51,0,0,1,15,1,230,50,0,0,17,1,111,30,0, 
  0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,230,50,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,230,50,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,230,50,0,0,1,19,25,0,0, 
  0,70,0,0,0,1,0,17,1,230,50,0,0,1,2,21,1,235,0,0,0,225,56,1,0,9,0,0,0,3,0,0,0,40,240,2,128,1,112,197,129,2,112,5,128,35,104,6,128,36,224,3,128,37,176,1,128,38,120,5, 
  128,39,128,4,128,41,80,2,128,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1, 
  0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0, 
  1,0,17,1,15,49,0,0,1,8,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0, 
  0,1,0,17,1,15,49,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128, 
  22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,96,53,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0, 
  0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,96,53,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,96,53,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,96,53,0,0, 
  15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,96,53,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0, 
  0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,96,53,0,0,1,19,25,0,0,0,72,0,0, 
  0,1,0,17,1,96,53,0,0,1,15,1,96,53,0,0,17,1,25,54,0,0,1,15,1,96,53,0,0,17,1,88,20,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,96,53,0,0,1,19,25,0,0,0,71, 
  0,0,0,1,0,17,1,96,53,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,96,53,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,96,53,0,0,1,2,21,1,235,0,0,0,225,56,1,0,9, 
  0,0,0,3,0,0,0,40,128,4,128,1,96,198,129,2,96,6,128,35,112,5,128,36,176,1,128,37,80,2,128,38,240,2,128,39,104,6,128,41,224,3,128,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,145,44, 
  0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,15,1,15,49,0,0, 
  15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0, 
  41,0,0,0,1,0,17,1,15,49,0,0,1,8,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0, 
  0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0, 
  98,0,0,0,1,0,15,1,218,55,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,218,55,0,0,15,1,24,85,0,0, 
  17,1,230,7,0,0,1,4,15,1,218,55,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,218,55,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19, 
  25,0,0,0,69,0,0,0,1,0,17,1,218,55,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128, 
  33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,218,55,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,218,55,0,0,1,15,1,218,55,0,0,17,1,147,56,0,0,1,15,1,218, 
  55,0,0,17,1,81,22,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,218,55,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,218,55,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,218, 
  55,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,218,55,0,0,1,2,21,1,235,0,0,0,225,56,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1,80,199,129,2,80,7,128,35,32,5,128,36,144,3, 
  128,37,16,6,128,38,48,4,128,39,160,2,128,41,176,6,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19, 
  13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0, 
  17,1,15,49,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1, 
  15,49,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128, 
  23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,84,58,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4, 
  19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,84,58,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,84,58,0,0,15,1,218,7,0,0,17,1,12,28,1,0, 
  1,4,15,1,84,58,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,84,58,0,0,1,2,21,0,184,0,0,0,255,255, 
  255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,84,58,0,0,1, 
  19,25,0,0,0,72,0,0,0,1,0,17,1,84,58,0,0,1,15,1,84,58,0,0,17,1,13,59,0,0,1,15,1,84,58,0,0,17,1,248,12,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,84,58,0, 
  0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,84,58,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,84,58,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,84,58,0,0,1,2,21,1,235, 
  0,0,0,225,56,1,0,9,0,0,0,3,0,0,0,40,64,3,128,1,80,199,129,2,80,7,128,35,96,6,128,36,192,5,128,37,176,1,128,38,80,2,128,39,208,4,128,41,48,4,128,4,15,1,15,49,0,0,15,1, 
  95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0, 
  0,0,1,0,17,1,15,49,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0, 
  1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,8,19,12,0,0,0,40,0,0, 
  0,1,0,1,21,1,235,0,0,0,225,56,1,0,9,0,0,0,3,0,0,0,40,112,5,128,1,80,199,129,2,80,7,128,35,128,4,128,36,176,1,128,37,224,3,128,38,240,2,128,39,96,6,128,41,80,2,128,4,15, 
  1,15,49,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1, 
  0,17,1,15,49,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,19, 
  31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,8,19, 
  12,0,0,0,40,0,0,0,1,0,1,21,1,218,0,0,0,180,54,1,0,8,0,0,0,3,0,0,0,48,152,1,128,1,144,193,128,2,144,65,128,18,120,5,128,25,232,2,128,13,224,5,128,22,216,4,128,23,56,4, 
  128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,214,61,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0, 
  0,97,0,0,0,1,0,15,1,214,61,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,214,61,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,214,61,0,0,15,1,206,7,0,0,17, 
  1,51,7,0,0,1,4,19,31,0,0,0,90,0,0,0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,214,61,0,0,1,2,21,0,206,0,0,0,255,255,255,255, 
  10,0,0,0,3,0,0,0,24,48,132,129,25,200,195,129,20,80,197,129,35,96,2,128,12,224,133,127,21,192,4,128,32,88,3,128,31,192,3,128,33,240,2,128,44,208,1,128,19,33,0,0,0,98,0,0,0,1,0,17, 
  1,214,61,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,214,61,0,0,1,15,1,214,61,0,0,17,1,104,67,0,0,1,15,1,214,61,0,0,17,1,26,63,0,0,1,1,15,1,214,61,0,0,17,1,165, 
  62,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,214,61,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,214,61,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,214,61,0,0,1,19,25, 
  0,0,0,70,0,0,0,1,0,17,1,214,61,0,0,1,2,21,7,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,47,48,2,128,45,176,1,128,42,112,129,128,43,48,65,127,94,240,1,128,4,17,1,28,50, 
  0,0,1,4,17,1,149,46,0,0,1,4,17,1,150,52,0,0,1,4,17,1,138,57,0,0,1,4,17,1,16,55,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0, 
  128,8,19,32,0,0,0,95,0,0,0,1,0,1,21,7,48,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,44,208,0,128,41,16,1,128,4,17,1,102,63,0,0,1,4,19,31,0,0,0,89,0,0,0,3,0, 
  14,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,2,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128, 
  23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,48,64,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4, 
  19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,48,64,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,48,64,0,0,15,1,218,7,0,0,17,1,12,28,1,0, 
  1,4,15,1,48,64,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,48,64,0,0,1,2,21,0,189,0,0,0,255,255, 
  255,255,9,0,0,0,3,0,0,0,24,168,131,129,25,64,131,129,20,200,132,129,35,64,2,128,12,88,133,127,21,56,4,128,32,56,3,128,33,208,2,128,44,176,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,48, 
  64,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,48,64,0,0,1,15,1,48,64,0,0,17,1,100,65,0,0,1,1,15,1,48,64,0,0,17,1,238,64,0,0,1,19,33,0,0,0,97,0,0,0,1,0, 
  17,1,48,64,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,48,64,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,48,64,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,48,64,0,0, 
  1,2,21,7,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,47,48,1,128,45,176,1,128,42,48,130,128,43,240,65,127,94,112,1,128,4,17,1,77,35,0,0,1,4,17,1,117,11,0,0,1,4,17,1,213, 
  18,0,0,1,4,17,1,211,32,0,0,1,4,17,1,199,37,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,32,0,0,0,96,0,0,0,3,0,14,1, 
  21,1,235,0,0,0,75,56,1,0,9,0,0,0,3,0,0,0,40,192,5,128,1,176,198,129,2,176,6,128,35,144,3,128,36,128,4,128,37,184,6,128,38,160,2,128,39,176,1,128,41,32,5,128,4,19,31,0,0,0, 
  92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,91,66,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,91,66,0,0,1,4,19,31,0,0,0, 
  87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,91,66,0,0,1,4,15,1,91,66,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,15,1,91,66,0,0,15,1,95,68,0,0,17,1,221, 
  8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,91,66,0,0,1,8,4,15,1,91,66,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,19,12,0,0,0, 
  40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,91,66,0,0,1,15,1,91,66,0, 
  0,17,1,154,66,0,0,1,1,2,21,1,193,0,0,0,75,56,1,0,9,0,0,0,3,0,0,0,40,128,4,128,1,64,197,129,2,64,5,128,35,72,5,128,36,112,2,128,37,16,4,128,38,176,1,128,39,80,3,128, 
  41,224,2,128,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,145,44,0,0,1,4,15,1,197,16,0,0,17,1,221,8,0,0,1,4,19,31,0, 
  0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4, 
  19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,21,1,235,0,0,0,75,56,1,0,9,0,0,0,3,0,0,0,40,40,5,128,1,64, 
  195,129,2,64,3,128,35,72,3,128,36,24,6,128,37,184,6,128,38,80,2,128,39,56,4,128,41,176,1,128,4,15,1,91,66,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,91,0,0,0, 
  1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,91,66,0,0,1,8,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,91,66,0,0,1,4,19,31,0,0,0,92,0,0, 
  0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,91,66,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,91,66,0,0,1,4,15,1,91,66,0,0,15,1, 
  95,68,0,0,17,1,145,44,0,0,1,4,15,1,91,66,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,19,13,0,0,0,41,0,0,0,1,0,1,21,0,62,0,0, 
  0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,107,68,0,0,1,15,1,107,68,0,0,17,1,170,68,0,0,1,1,2,21,1, 
  193,0,0,0,237,50,1,0,9,0,0,0,3,0,0,0,40,112,2,128,1,144,197,129,2,144,5,128,35,48,3,128,36,176,4,128,37,32,5,128,38,240,3,128,39,176,1,128,41,152,5,128,4,19,31,0,0,0,92,0, 
  0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0, 
  0,2,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,145,44,0,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,8,4,15,1, 
  197,16,0,0,17,1,221,8,0,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,21,0,189,0,0,0,255,255,255,255,9,0,0,0,3,0,0,0,24,160,3,128,25,56,131,129,20,192,132,129,35,64,2,128,12,80, 
  133,127,5,224,69,128,21,48,4,128,33,208,2,128,44,176,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,120,69,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,120,69,0,0,1,15,1,120,69,0,0, 
  17,1,33,84,0,0,1,15,1,120,69,0,0,17,1,54,70,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,120,69,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,120,69,0,0,1,19,25,0,0, 
  0,69,0,0,0,1,0,17,1,120,69,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,120,69,0,0,1,1,2,21,7,96,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,47,144,1,128,45,208,1,129, 
  42,80,130,128,43,16,66,127,94,80,1,128,125,144,2,128,4,17,1,167,81,0,0,1,4,17,1,45,79,0,0,1,4,17,1,179,76,0,0,1,4,17,1,57,74,0,0,1,4,17,1,178,70,0,0,1,4,19,5,0, 
  0,0,19,0,0,0,4,0,14,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,2,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128, 
  1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,124,71,0,0,15,1,24,85,0,0, 
  17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,124,71,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,124,71,0,0,15,1,218,7, 
  0,0,17,1,12,28,1,0,1,4,15,1,124,71,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,124,71,0,0,1,2, 
  21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1, 
  0,17,1,124,71,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,124,71,0,0,1,15,1,124,71,0,0,17,1,53,72,0,0,1,15,1,124,71,0,0,17,1,41,25,0,0,1,19,33,0,0,0,97,0,0, 
  0,1,0,17,1,124,71,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,124,71,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,124,71,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,124, 
  71,0,0,1,2,21,1,235,0,0,0,108,57,1,0,9,0,0,0,3,0,0,0,40,128,4,128,1,16,198,129,2,16,6,128,35,160,2,128,36,24,6,128,37,112,5,128,38,144,3,128,39,176,1,128,41,184,6,128,4, 
  19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4, 
  19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4, 
  15,1,44,73,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,8,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1, 
  19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1, 
  15,1,44,73,0,0,17,1,107,73,0,0,1,1,2,21,1,193,0,0,0,108,57,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1,144,197,129,2,144,5,128,35,112,2,128,36,152,5,128,37,160,3,128,38,16,4, 
  128,39,208,4,128,41,48,3,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1, 
  197,16,0,0,17,1,221,8,0,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,92,0,0,0,1, 
  0,19,13,0,0,0,42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,145,44,0,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48, 
  120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,3,75,0,0,15,1,24, 
  85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,3,75,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,3,75,0,0,15, 
  1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,3,75,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,3,75,0, 
  0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0, 
  0,0,1,0,17,1,3,75,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,3,75,0,0,1,15,1,3,75,0,0,17,1,188,75,0,0,1,15,1,3,75,0,0,17,1,111,30,0,0,1,19,33,0,0,0, 
  97,0,0,0,1,0,17,1,3,75,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,3,75,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,3,75,0,0,1,19,25,0,0,0,70,0,0,0,1,0, 
  17,1,3,75,0,0,1,2,21,1,235,0,0,0,108,57,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1,80,199,129,2,80,7,128,35,144,3,128,36,176,6,128,37,112,5,128,38,128,4,128,39,160,2,128,41,16, 
  6,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0, 
  0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0, 
  0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,145,44,0, 
  0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2, 
  128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,125,77,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0, 
  0,97,0,0,0,1,0,15,1,125,77,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,125,77,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,125,77,0,0,15,1,206,7,0,0,17, 
  1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,125,77,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25, 
  24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,125,77,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,125,77, 
  0,0,1,15,1,125,77,0,0,17,1,54,78,0,0,1,15,1,125,77,0,0,17,1,88,20,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,125,77,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1, 
  125,77,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,125,77,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,125,77,0,0,1,2,21,1,235,0,0,0,108,57,1,0,9,0,0,0,3,0,0,0, 
  40,104,6,128,1,96,198,129,2,96,6,128,35,160,2,128,36,208,4,128,37,48,4,128,38,112,5,128,39,176,1,128,41,144,3,128,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17, 
  1,44,73,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,44, 
  73,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17, 
  1,44,73,0,0,1,8,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7, 
  0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15, 
  1,247,79,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,247,79,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1, 
  4,15,1,247,79,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,247,79,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0, 
  0,1,0,17,1,247,79,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1, 
  128,19,33,0,0,0,98,0,0,0,1,0,17,1,247,79,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,247,79,0,0,1,15,1,247,79,0,0,17,1,176,80,0,0,1,15,1,247,79,0,0,17,1,81,22, 
  0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,247,79,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,247,79,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,247,79,0,0,1,19,25,0, 
  0,0,70,0,0,0,1,0,17,1,247,79,0,0,1,2,21,1,235,0,0,0,108,57,1,0,9,0,0,0,3,0,0,0,40,224,3,128,1,192,197,129,2,192,5,128,35,104,6,128,36,176,1,128,37,200,5,128,38,208, 
  4,128,39,240,2,128,41,80,2,128,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,92,0,0,0, 
  1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,19,31,0,0,0,91,0,0,0, 
  1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,8,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0, 
  0,0,1,0,17,1,44,73,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133, 
  128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,113,82,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0, 
  0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,113,82,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,113,82,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,113,82,0, 
  0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,113,82,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3, 
  0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,113,82,0,0,1,19,25,0,0,0,72,0, 
  0,0,1,0,17,1,113,82,0,0,1,15,1,113,82,0,0,17,1,42,83,0,0,1,15,1,113,82,0,0,17,1,248,12,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,113,82,0,0,1,19,25,0,0,0, 
  71,0,0,0,1,0,17,1,113,82,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,113,82,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,113,82,0,0,1,2,21,1,235,0,0,0,108,57,1,0, 
  9,0,0,0,3,0,0,0,40,104,6,128,1,48,196,129,2,48,4,128,35,64,3,128,36,160,2,128,37,40,5,128,38,56,4,128,39,176,1,128,41,200,5,128,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0, 
  0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44, 
  73,0,0,1,8,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,44,73, 
  0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,235, 
  0,0,0,108,57,1,0,9,0,0,0,3,0,0,0,40,64,3,128,1,208,196,129,2,208,4,128,35,120,5,128,36,48,4,128,37,216,4,128,38,104,6,128,39,176,1,128,41,160,2,128,4,19,31,0,0,0,92,0,0, 
  0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0, 
  0,0,1,0,17,1,44,73,0,0,1,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,8,4,15,1,44,73,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,87,0, 
  0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,44,73,0,0,1,19,12,0,0,0,40,0,0, 
  0,1,0,1,19,25,0,0,0,70,0,0,0,1,0,1,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4, 
  19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,155,205,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0, 
  0,0,1,0,15,1,155,205,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,186,85,0,0,1,2,21,1,54,3,0,0,179,58,1,0,23,0,0,0, 
  4,0,0,0,16,248,138,133,1,112,195,131,2,112,3,128,3,104,152,131,4,32,23,128,5,216,21,128,6,112,21,128,23,88,8,128,8,160,148,130,9,208,147,130,10,136,18,128,11,64,17,128,12,248,15,128,13,88,14,128, 
  14,16,13,128,15,200,139,129,17,40,10,129,19,88,9,128,24,88,7,128,25,144,6,128,33,200,5,128,47,248,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0, 
  15,1,39,198,0,0,15,1,161,204,0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0, 
  0,1,0,15,1,39,198,0,0,17,1,237,122,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,39,198,0,0,17,1,186,122,0,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,174,122,0,0, 
  15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1, 
  162,122,0,0,17,1,195,112,0,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,216,102,0,0,17,1,35,220, 
  0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0, 
  0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19, 
  9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,19, 
  10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0,0,0, 
  1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15, 
  1,162,122,0,0,17,1,241,88,0,0,1,4,19,6,0,0,0,23,0,0,0,3,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17, 
  1,39,198,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19, 
  27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,2,21,1,66,3,0,0,179,58,1,0,23,0,0,0,4,0,0,0,16,88,139,133,1,112,195,131,2,112,3,128,3, 
  200,152,131,4,128,23,128,5,56,22,128,6,208,21,128,23,184,8,128,8,0,149,130,9,48,148,130,10,232,18,128,11,160,17,128,12,88,16,128,13,184,14,128,14,112,13,128,15,40,140,129,17,136,10,129,19,184,9,128,24, 
  184,7,128,25,240,6,128,33,200,5,128,47,248,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,115,92,0,0,15,1,192,102,0,0,15,1,173,204,0, 
  0,17,1,31,33,1,0,1,4,15,1,115,92,0,0,15,1,192,102,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,115,92,0,0,15,1,192,102,0,0,15,1, 
  103,92,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,115,92,0,0,17,1,52,92,0,0,1,4,15,1,115,92,0,0,15,1,192,102,0,0,15,1,174,122,0,0,15,1,186,31,1, 
  0,17,1,179,28,1,0,1,4,15,1,115,92,0,0,15,1,192,102,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,115,92,0,0,15,1,192,102,0,0,15,1,162,122,0,0,17, 
  1,195,112,0,0,1,4,15,1,115,92,0,0,15,1,192,102,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,115,92,0,0,15,1,192,102,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19, 
  10,0,0,0,37,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,43,0,0,0,127,0,0,0,1,0,17,1,115,92,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0, 
  1,0,19,43,0,0,0,127,0,0,0,1,0,17,1,115,92,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,43,0,0,0,127, 
  0,0,0,1,0,17,1,115,92,0,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,43,0,0,0,127,0,0,0,1,0,17,1,115,92,0,0,1,4,19,10,0,0,0,33, 
  0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,43,0,0,0,127,0,0,0,1,0,17,1,115,92,0,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,43,0, 
  0,0,127,0,0,0,1,0,17,1,115,92,0,0,1,4,15,1,115,92,0,0,15,1,192,102,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,115,92,0,0,15,1,192,102,0,0,15,1,162,122,0,0, 
  17,1,241,88,0,0,1,4,19,40,0,0,0,122,0,0,0,2,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,43,0,0,0,127,0,0,0,1,0,17,1,115,92,0,0, 
  1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,43,0,0,0,127,0,0,0,1,0,17,1,115,92,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74, 
  0,0,0,1,0,19,43,0,0,0,127,0,0,0,1,0,17,1,115,92,0,0,1,2,21,1,39,0,0,0,225,59,1,0,3,0,0,0,1,0,0,0,2,48,1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0, 
  0,1,8,19,33,0,0,0,97,0,0,0,1,0,1,19,27,0,0,0,77,0,0,0,1,0,1,21,0,116,1,0,0,255,255,255,255,17,0,0,0,4,0,0,0,21,32,7,128,33,200,4,128,27,152,69,131,3,16,11, 
  128,20,176,7,128,5,128,202,126,38,56,4,128,23,144,6,128,24,40,70,129,28,48,197,129,10,240,9,128,11,96,201,125,12,208,72,127,40,168,3,128,14,64,8,128,43,64,3,128,44,176,2,128,19,33,0,0,0,98,0, 
  0,0,1,0,17,1,115,92,0,0,1,15,1,115,92,0,0,17,1,31,96,0,0,1,19,27,0,0,0,81,0,0,0,1,0,17,1,115,92,0,0,1,19,27,0,0,0,78,0,0,0,1,0,17,1,115,92,0,0,1, 
  15,1,115,92,0,0,17,1,27,94,0,0,1,15,1,115,92,0,0,17,1,22,9,0,0,1,19,43,0,0,0,127,0,0,0,1,0,17,1,115,92,0,0,1,15,1,115,92,0,0,17,1,232,93,0,0,1,19,3,0, 
  0,0,16,0,0,0,1,0,17,1,115,92,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,115,92,0,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,115,92,0,0,1,19,27,0,0,0,79,0,0,0, 
  1,0,17,1,115,92,0,0,1,19,27,0,0,0,80,0,0,0,1,0,17,1,115,92,0,0,1,19,27,0,0,0,77,0,0,0,1,0,17,1,115,92,0,0,1,19,27,0,0,0,76,0,0,0,1,0,17,1,115,92, 
  0,0,1,19,27,0,0,0,75,0,0,0,1,0,17,1,115,92,0,0,1,19,27,0,0,0,74,0,0,0,1,0,17,1,115,92,0,0,1,2,21,1,39,0,0,0,225,59,1,0,3,0,0,0,1,0,0,0,2,48, 
  1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0,0,1,8,19,33,0,0,0,97,0,0,0,1,0,1,21,1,235,0,0,0,198,60,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1,96,198,129,2,96,6, 
  128,35,160,2,128,36,48,4,128,37,192,5,128,38,208,4,128,39,104,6,128,41,144,3,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,18,95,0,0,1,4,19,31,0,0, 
  0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,18,95,0,0,1,4,15,1,18,95,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,18,95,0,0,15,1,95,68,0,0,17,1, 
  145,44,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,18,95,0,0,1,4,15,1,18,95,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,8,4,19,31,0, 
  0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,18,95,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128, 
  13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,18,95,0,0,1,15,1,18,95,0,0,17,1,81,95,0,0,1,1,2,21,1,193,0,0,0,198,60,1,0,9,0,0,0,3,0,0,0,40, 
  176,1,128,1,144,197,129,2,144,5,128,35,112,2,128,36,152,5,128,37,160,3,128,38,16,4,128,39,208,4,128,41,48,3,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4, 
  19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,221,8,0,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0, 
  0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,145,44,0,0,1,19,12,0,0,0,39, 
  0,0,0,2,0,14,1,21,1,215,2,0,0,179,58,1,0,23,0,0,0,4,0,0,0,16,56,138,133,1,112,195,131,2,112,3,128,3,160,149,131,4,136,20,128,5,112,19,128,6,0,19,128,23,40,8,128,8,96,146, 
  130,9,192,145,130,10,168,16,128,11,144,15,128,12,120,14,128,13,8,13,128,14,240,11,128,15,216,138,129,17,152,9,129,19,248,8,128,24,88,7,128,25,96,6,128,33,104,5,128,47,200,4,128,48,120,3,128,8,4,19, 
  44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,180,102,0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,180,102,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1, 
  4,19,28,0,0,0,82,0,0,0,1,0,15,1,180,102,0,0,15,1,103,92,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,180,102,0,0,15,1,42,99,0,0,17,1,247,98,0, 
  0,1,4,15,1,180,102,0,0,15,1,174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,180,102,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,180,102,0, 
  0,15,1,162,122,0,0,17,1,195,112,0,0,1,4,15,1,180,102,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,180,102,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0, 
  37,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,43,0,0,0,128,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,43,0,0,0,128,0,0, 
  0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,43,0,0,0,128,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0, 
  1,0,19,27,0,0,0,76,0,0,0,1,0,19,43,0,0,0,128,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,43,0,0,0,128,0,0,0,2,0,1, 
  4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,43,0,0,0,128,0,0,0,2,0,1,4,15,1,180,102,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,180,102, 
  0,0,15,1,162,122,0,0,17,1,241,88,0,0,1,4,19,40,0,0,0,121,0,0,0,3,0,14,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,43,0,0,0,128,0,0, 
  0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,43,0,0,0,128,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0, 
  1,0,19,43,0,0,0,128,0,0,0,2,0,1,2,21,1,39,0,0,0,159,61,1,0,3,0,0,0,1,0,0,0,2,240,0,128,1,240,64,128,7,248,0,128,8,4,17,1,105,6,0,0,1,19,33,0,0,0,97, 
  0,0,0,1,0,1,21,0,82,1,0,0,255,255,255,255,16,0,0,0,4,0,0,0,21,16,6,128,33,64,4,128,27,16,5,128,3,0,10,128,20,160,6,128,5,112,201,126,38,176,3,128,23,128,5,128,24,24,69,129, 
  28,168,132,129,10,224,8,128,11,80,200,125,12,192,71,127,40,32,3,128,14,48,7,128,44,144,2,128,19,33,0,0,0,98,0,0,0,1,0,17,1,42,99,0,0,1,19,27,0,0,0,81,0,0,0,1,0,17,1,42, 
  99,0,0,1,19,27,0,0,0,78,0,0,0,1,0,17,1,42,99,0,0,1,15,1,42,99,0,0,17,1,176,100,0,0,1,15,1,42,99,0,0,17,1,22,9,0,0,1,1,15,1,42,99,0,0,17,1,125,100,0, 
  0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,42,99,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,42,99,0,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,42,99,0,0,1,19,27,0,0, 
  0,79,0,0,0,1,0,17,1,42,99,0,0,1,19,27,0,0,0,80,0,0,0,1,0,17,1,42,99,0,0,1,19,27,0,0,0,77,0,0,0,1,0,17,1,42,99,0,0,1,19,27,0,0,0,76,0,0,0,1, 
  0,17,1,42,99,0,0,1,19,27,0,0,0,75,0,0,0,1,0,17,1,42,99,0,0,1,19,27,0,0,0,74,0,0,0,1,0,17,1,42,99,0,0,1,2,21,1,39,0,0,0,159,61,1,0,3,0,0,0,1, 
  0,0,0,2,48,1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0,0,1,8,19,33,0,0,0,97,0,0,0,1,0,1,21,1,235,0,0,0,154,62,1,0,9,0,0,0,3,0,0,0,40,240,2,128,1,112, 
  197,129,2,112,5,128,35,104,6,128,36,176,1,128,37,224,3,128,38,120,5,128,39,128,4,128,41,80,2,128,4,15,1,167,101,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,15,1,167,101,0,0,15,1,95, 
  68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,167,101,0,0,1,4,15,1,167,101,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1, 
  4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,167,101,0,0,1,8,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,167,101,0,0, 
  1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,167,101,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0, 
  0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,167,101,0,0,1,15,1,167,101,0,0,17,1,230,101,0,0,1,1,2,21,1,193,0,0,0,154,62,1,0,9,0,0,0, 
  3,0,0,0,40,72,5,128,1,64,197,129,2,64,5,128,35,32,2,128,36,208,4,128,37,224,2,128,38,16,4,128,39,80,3,128,41,176,1,128,4,15,1,197,16,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0, 
  87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0, 
  0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,145,44,0,0,1,8,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,19, 
  12,0,0,0,39,0,0,0,2,0,14,1,19,43,0,0,0,128,0,0,0,2,0,1,19,43,0,0,0,127,0,0,0,1,0,1,19,27,0,0,0,76,0,0,0,1,0,1,19,27,0,0,0,79,0,0,0,1,0,1, 
  21,1,66,3,0,0,137,63,1,0,23,0,0,0,4,0,0,0,16,192,139,133,1,112,67,129,2,112,131,131,3,200,152,131,4,128,23,128,5,56,22,128,17,240,138,131,23,184,8,128,8,104,149,130,9,152,148,130,10,80, 
  19,128,11,8,18,128,12,192,16,128,13,32,15,128,14,216,13,128,15,144,140,129,18,136,10,128,19,184,9,128,24,184,7,128,25,240,6,128,33,200,5,128,47,248,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0, 
  0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,90,106,0,0,15,1,183,112,0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,90,106,0,0,15,1,183,112,0,0,15,1,27,198,0,0,17,1, 
  76,149,0,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,90,106,0,0,15,1,183,112,0,0,15,1,103,92,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,90,106,0,0, 
  17,1,39,106,0,0,1,4,15,1,90,106,0,0,15,1,183,112,0,0,15,1,174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,90,106,0,0,15,1,183,112,0,0,15,1,174,122,0,0,15,1, 
  167,28,1,0,17,1,12,28,1,0,1,4,15,1,90,106,0,0,15,1,183,112,0,0,15,1,162,122,0,0,17,1,195,112,0,0,1,4,19,40,0,0,0,120,0,0,0,2,0,1,4,15,1,90,106,0,0,15,1,183, 
  112,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,90,106,0,0,15,1,183,112,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0,76, 
  0,0,0,1,0,19,42,0,0,0,125,0,0,0,1,0,17,1,90,106,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,42,0,0,0,125,0,0,0,1,0,17,1,90, 
  106,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,42,0,0,0,125,0,0,0,1,0,17,1,90,106,0,0,1,4,19,10,0, 
  0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,42,0,0,0,125,0,0,0,1,0,17,1,90,106,0,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0, 
  19,42,0,0,0,125,0,0,0,1,0,17,1,90,106,0,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,42,0,0,0,125,0,0,0,1,0,17,1,90,106,0,0,1,4, 
  15,1,90,106,0,0,15,1,183,112,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,90,106,0,0,15,1,183,112,0,0,15,1,162,122,0,0,17,1,241,88,0,0,1,4,19,3,0,0,0,13,0,0, 
  0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,42,0,0,0,125,0,0,0,1,0,17,1,90,106,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,42,0,0,0, 
  125,0,0,0,1,0,17,1,90,106,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,42,0,0,0,125,0,0,0,1,0,17,1,90,106,0,0,1,2,21,1,39,0,0, 
  0,97,64,1,0,3,0,0,0,1,0,0,0,2,48,1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0,0,1,8,19,33,0,0,0,97,0,0,0,1,0,1,21,0,116,1,0,0,255,255,255,255,17,0,0,0, 
  4,0,0,0,21,32,7,128,33,200,4,128,27,152,5,128,3,16,11,128,20,176,7,128,5,128,202,126,38,56,4,128,23,144,6,128,24,40,70,129,28,48,197,129,10,240,73,129,11,96,201,125,12,208,72,127,40,168,3,128, 
  14,64,8,128,42,64,3,128,44,176,2,128,19,33,0,0,0,98,0,0,0,1,0,17,1,90,106,0,0,1,15,1,90,106,0,0,17,1,211,109,0,0,1,19,27,0,0,0,81,0,0,0,1,0,17,1,90,106,0,0, 
  1,19,27,0,0,0,78,0,0,0,1,0,17,1,90,106,0,0,1,15,1,90,106,0,0,17,1,207,107,0,0,1,15,1,90,106,0,0,17,1,22,9,0,0,1,19,42,0,0,0,125,0,0,0,1,0,17,1,90,106, 
  0,0,1,15,1,90,106,0,0,17,1,39,106,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,90,106,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,90,106,0,0,1,19,3,0,0,0,14,0,0, 
  0,1,0,17,1,90,106,0,0,1,19,27,0,0,0,79,0,0,0,1,0,17,1,90,106,0,0,1,19,27,0,0,0,80,0,0,0,1,0,17,1,90,106,0,0,1,19,27,0,0,0,77,0,0,0,1,0,17,1,90, 
  106,0,0,1,19,27,0,0,0,76,0,0,0,1,0,17,1,90,106,0,0,1,19,27,0,0,0,75,0,0,0,1,0,17,1,90,106,0,0,1,19,27,0,0,0,74,0,0,0,1,0,17,1,90,106,0,0,1,2,21, 
  1,235,0,0,0,70,65,1,0,9,0,0,0,3,0,0,0,40,216,4,128,1,208,196,129,2,208,4,128,35,240,2,128,36,176,1,128,37,200,5,128,38,104,6,128,39,224,3,128,41,80,2,128,4,15,1,198,108,0,0, 
  15,1,95,68,0,0,17,1,145,44,0,0,1,4,15,1,198,108,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,198,108, 
  0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,198,108,0,0,1,8,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,198, 
  108,0,0,1,4,15,1,198,108,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,198,108,0,0,1,19,12,0,0,0,40, 
  0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,198,108,0,0,1,15,1,198,108,0,0, 
  17,1,5,109,0,0,1,1,2,21,1,193,0,0,0,70,65,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1,144,197,129,2,144,5,128,35,112,2,128,36,152,5,128,37,160,3,128,38,16,4,128,39,208,4,128,41, 
  48,3,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1, 
  221,8,0,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0, 
  42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,145,44,0,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,21,1,215,2,0,0,137,63,1,0,23,0,0,0,4,0,0,0,16,168,138,133,1,112,67, 
  129,2,112,131,131,3,160,149,131,4,136,20,128,5,112,19,128,17,8,138,131,23,40,8,128,8,208,146,130,9,48,146,130,10,24,17,128,11,0,16,128,12,232,14,128,13,120,13,128,14,96,12,128,15,72,139,129,18,152,9, 
  128,19,248,8,128,24,88,7,128,25,96,6,128,33,104,5,128,47,200,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,171,112,0,0,15,1,173,204,0, 
  0,17,1,31,33,1,0,1,4,15,1,171,112,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,171,112,0,0,15,1,103,92,0,0,17,1,22,9,0,0,1,4, 
  19,24,0,0,0,63,0,0,0,1,0,15,1,171,112,0,0,15,1,42,99,0,0,17,1,247,98,0,0,1,4,15,1,171,112,0,0,15,1,174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,171, 
  112,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,171,112,0,0,15,1,162,122,0,0,17,1,195,112,0,0,1,4,19,40,0,0,0,119,0,0,0,3,0,14,1,4,15,1,171, 
  112,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,171,112,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19, 
  42,0,0,0,126,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,42,0,0,0,126,0,0,0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3, 
  0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,42,0,0,0,126,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,42,0,0,0, 
  126,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,42,0,0,0,126,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76, 
  0,0,0,1,0,19,42,0,0,0,126,0,0,0,2,0,1,4,15,1,171,112,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,171,112,0,0,15,1,162,122,0,0,17,1,241,88,0,0,1,4,19,3, 
  0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,42,0,0,0,126,0,0,0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,42,0,0,0, 
  126,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,42,0,0,0,126,0,0,0,2,0,1,2,19,42,0,0,0,126,0,0,0,2,0,1,19,42,0,0,0, 
  125,0,0,0,1,0,1,21,1,66,3,0,0,31,66,1,0,23,0,0,0,4,0,0,0,16,192,139,133,1,112,67,129,2,112,3,128,3,200,88,131,4,128,87,131,5,56,22,128,17,240,138,131,23,184,8,128,8,104,149, 
  130,9,152,148,130,10,80,19,128,11,8,18,128,12,192,16,128,13,32,15,128,14,216,13,128,15,144,140,129,19,32,10,128,20,184,9,128,24,184,7,128,25,240,6,128,33,200,5,128,47,248,4,128,48,120,3,128,8,4,19, 
  44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,57,116,0,0,15,1,150,122,0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,57,116,0,0,15,1,150,122,0,0,15, 
  1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,57,116,0,0,15,1,150,122,0,0,15,1,103,92,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1, 
  0,15,1,57,116,0,0,17,1,6,116,0,0,1,4,15,1,57,116,0,0,15,1,150,122,0,0,15,1,174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,57,116,0,0,15,1,150,122,0,0,15, 
  1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,19,40,0,0,0,118,0,0,0,2,0,1,4,15,1,57,116,0,0,15,1,150,122,0,0,15,1,162,122,0,0,17,1,195,112,0,0,1,4,15,1, 
  57,116,0,0,15,1,150,122,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,57,116,0,0,15,1,150,122,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1, 
  0,19,27,0,0,0,76,0,0,0,1,0,19,41,0,0,0,123,0,0,0,1,0,17,1,57,116,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,41,0,0,0,123,0, 
  0,0,1,0,17,1,57,116,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,41,0,0,0,123,0,0,0,1,0,17,1,57,116, 
  0,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,41,0,0,0,123,0,0,0,1,0,17,1,57,116,0,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0, 
  0,76,0,0,0,1,0,19,41,0,0,0,123,0,0,0,1,0,17,1,57,116,0,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,41,0,0,0,123,0,0,0,1,0,17, 
  1,57,116,0,0,1,4,15,1,57,116,0,0,15,1,150,122,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,57,116,0,0,15,1,150,122,0,0,15,1,162,122,0,0,17,1,241,88,0,0,1,4,19, 
  3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,41,0,0,0,123,0,0,0,1,0,17,1,57,116,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0, 
  1,0,19,41,0,0,0,123,0,0,0,1,0,17,1,57,116,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,41,0,0,0,123,0,0,0,1,0,17,1,57,116,0,0, 
  1,2,21,1,39,0,0,0,247,66,1,0,3,0,0,0,1,0,0,0,2,48,1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0,0,1,8,19,33,0,0,0,97,0,0,0,1,0,1,21,0,116,1,0,0,255, 
  255,255,255,17,0,0,0,4,0,0,0,21,32,7,128,33,200,4,128,27,152,5,128,3,16,11,128,20,176,7,128,5,128,202,126,38,56,4,128,23,144,6,128,24,40,198,129,41,64,3,128,10,240,9,128,11,96,201,125,12, 
  208,72,128,28,48,197,128,14,64,8,128,40,168,3,128,44,176,2,128,19,33,0,0,0,98,0,0,0,1,0,17,1,57,116,0,0,1,15,1,57,116,0,0,17,1,178,119,0,0,1,19,27,0,0,0,81,0,0,0,1, 
  0,17,1,57,116,0,0,1,19,27,0,0,0,78,0,0,0,1,0,17,1,57,116,0,0,1,15,1,57,116,0,0,17,1,174,117,0,0,1,15,1,57,116,0,0,17,1,22,9,0,0,1,19,41,0,0,0,123,0,0, 
  0,1,0,17,1,57,116,0,0,1,15,1,57,116,0,0,17,1,6,116,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,57,116,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,57,116,0,0,1,19, 
  3,0,0,0,14,0,0,0,1,0,17,1,57,116,0,0,1,19,27,0,0,0,79,0,0,0,1,0,17,1,57,116,0,0,1,19,27,0,0,0,80,0,0,0,1,0,17,1,57,116,0,0,1,19,27,0,0,0,77,0, 
  0,0,1,0,17,1,57,116,0,0,1,19,27,0,0,0,76,0,0,0,1,0,17,1,57,116,0,0,1,19,27,0,0,0,75,0,0,0,1,0,17,1,57,116,0,0,1,19,27,0,0,0,74,0,0,0,1,0,17,1, 
  57,116,0,0,1,2,21,1,235,0,0,0,220,67,1,0,9,0,0,0,3,0,0,0,40,160,2,128,1,128,196,129,2,128,4,128,35,176,1,128,36,184,6,128,37,136,4,128,38,40,5,128,39,144,3,128,41,24,6,128, 
  4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,165,118,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,165,118,0,0,1, 
  4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,165,118,0,0,1,8,4,15,1,165,118,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0, 
  0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,165,118,0,0,1,4,15,1,165,118,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,165,118,0,0,15,1,95,68,0,0,17,1,145,44,0,0, 
  1,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,165,118,0,0, 
  1,15,1,165,118,0,0,17,1,228,118,0,0,1,1,2,21,1,193,0,0,0,220,67,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1,144,197,129,2,144,5,128,35,112,2,128,36,152,5,128,37,160,3,128,38,16, 
  4,128,39,208,4,128,41,48,3,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15, 
  1,197,16,0,0,17,1,221,8,0,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,92,0,0,0, 
  1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,145,44,0,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,21,1,215,2,0,0,31,66,1,0,23,0,0,0,4,0,0,0, 
  16,168,138,133,1,112,67,129,2,112,3,128,3,160,85,131,4,136,84,131,5,112,19,128,17,8,138,131,23,40,8,128,8,208,146,130,9,48,146,130,10,24,17,128,11,0,16,128,12,232,14,128,13,120,13,128,14,96,12,128, 
  15,72,139,129,19,104,9,128,20,248,8,128,24,88,7,128,25,96,6,128,33,104,5,128,47,200,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,138,122, 
  0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,138,122,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,138,122,0,0,15,1,103,92,0,0,17, 
  1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,138,122,0,0,15,1,42,99,0,0,17,1,247,98,0,0,1,4,15,1,138,122,0,0,15,1,174,122,0,0,15,1,186,31,1,0,17,1,179,28, 
  1,0,1,4,15,1,138,122,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,19,40,0,0,0,117,0,0,0,3,0,14,1,4,15,1,138,122,0,0,15,1,162,122,0,0,17,1,195,112, 
  0,0,1,4,15,1,138,122,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,138,122,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0, 
  76,0,0,0,1,0,19,41,0,0,0,124,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,41,0,0,0,124,0,0,0,2,0,1,4,19,20,0,0,0,58, 
  0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,41,0,0,0,124,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0, 
  1,0,19,41,0,0,0,124,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,41,0,0,0,124,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1, 
  0,19,27,0,0,0,76,0,0,0,1,0,19,41,0,0,0,124,0,0,0,2,0,1,4,15,1,138,122,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,138,122,0,0,15,1,162,122,0,0,17,1,241, 
  88,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,41,0,0,0,124,0,0,0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0, 
  1,0,19,41,0,0,0,124,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,41,0,0,0,124,0,0,0,2,0,1,2,19,41,0,0,0,124,0,0,0,2, 
  0,1,19,41,0,0,0,123,0,0,0,1,0,1,19,27,0,0,0,81,0,0,0,1,0,1,19,27,0,0,0,74,0,0,0,1,0,1,21,1,39,0,0,0,225,59,1,0,3,0,0,0,1,0,0,0,2,240,0,128, 
  1,240,64,128,7,248,0,128,8,4,17,1,105,6,0,0,1,19,33,0,0,0,97,0,0,0,1,0,1,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17, 
  248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,131,123,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0, 
  63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,131,123,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,186,85,0,0,1,2,21, 
  0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,33,152,1,128,26,0,2,128,11,144,1,128,44,240,2,128,29,144,2,128,6,144,1,128,7,16,4,128,1,15,1,131,123,0,0,17,1,185,204, 
  0,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,131,123,0,0,1,19,11,0,0,0,38,0,0,0,2,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,131,123,0,0,1,19,33,0,0,0,97,0,0,0, 
  1,0,17,1,131,123,0,0,1,15,1,131,123,0,0,17,1,19,124,0,0,1,2,21,1,119,0,0,0,181,68,1,0,5,0,0,0,2,0,0,0,8,120,3,129,1,48,129,128,2,48,1,128,25,88,2,128,48,56,1, 
  128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,64,149,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1, 
  0,15,1,64,149,0,0,17,1,185,204,0,0,1,4,17,1,139,124,0,0,1,2,21,1,55,3,0,0,179,58,1,0,23,0,0,0,4,0,0,0,16,248,138,133,1,112,195,131,2,112,3,128,3,112,152,131,4,40,23, 
  128,5,224,21,128,6,112,21,128,23,88,8,128,8,160,148,130,9,208,147,130,10,136,18,128,11,64,17,128,12,248,15,128,13,88,14,128,14,16,13,128,15,200,139,129,17,40,10,129,19,88,9,128,24,88,7,128,25,144,6, 
  128,33,200,5,128,47,248,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,233,128,0,0,15,1,161,204,0,0,15,1,173,204,0,0,17,1,31,33,1, 
  0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,233,128,0,0,17,1,195,127,0,0,1,4,19,24,0,0,0,63, 
  0,0,0,1,0,15,1,233,128,0,0,17,1,52,92,0,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,233,128,0,0,15,1,161, 
  204,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,162,122,0,0,17,1,195,112,0,0,1,4,15,1,233,128,0,0,15,1,161,204,0, 
  0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0,76,0,0, 
  0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0, 
  0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,10,0,0,0, 
  34,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9, 
  0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,15,1, 
  233,128,0,0,15,1,161,204,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,162,122,0,0,17,1,241,88,0,0,1,4,19,6,0,0,0,22,0,0,0,4, 
  0,14,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0, 
  0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17, 
  1,233,128,0,0,1,2,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0, 
  1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,89,128,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,89,128,0, 
  0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,186,85,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,33, 
  144,1,128,26,248,1,128,11,136,2,128,44,240,2,128,29,144,2,128,6,136,2,128,7,16,4,128,15,1,89,128,0,0,17,1,185,204,0,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,89,128,0,0,1,1,19, 
  11,0,0,0,38,0,0,0,2,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,89,128,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,89,128,0,0,1,15,1,89,128,0,0,17,1,19,124,0,0,1, 
  2,21,0,138,1,0,0,255,255,255,255,18,0,0,0,4,0,0,0,21,216,6,128,33,128,4,128,27,80,5,128,3,192,11,128,20,104,7,128,5,48,203,126,6,160,74,130,23,72,6,128,24,224,5,130,9,56,10,128,10, 
  168,9,128,11,24,201,125,12,136,72,128,28,232,4,129,14,248,7,128,38,240,3,128,40,96,3,128,44,208,2,128,19,33,0,0,0,98,0,0,0,1,0,17,1,233,128,0,0,1,19,27,0,0,0,81,0,0,0,1,0, 
  17,1,233,128,0,0,1,19,27,0,0,0,78,0,0,0,1,0,17,1,233,128,0,0,1,15,1,233,128,0,0,17,1,73,148,0,0,1,15,1,233,128,0,0,17,1,35,147,0,0,1,19,9,0,0,0,28,0,0,0, 
  1,0,17,1,233,128,0,0,1,15,1,233,128,0,0,17,1,232,93,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,233,128,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,233,128,0,0,1,19,3, 
  0,0,0,14,0,0,0,1,0,17,1,233,128,0,0,1,19,27,0,0,0,79,0,0,0,1,0,17,1,233,128,0,0,1,19,27,0,0,0,80,0,0,0,1,0,17,1,233,128,0,0,1,19,27,0,0,0,77,0,0, 
  0,1,0,17,1,233,128,0,0,1,19,27,0,0,0,76,0,0,0,1,0,17,1,233,128,0,0,1,15,1,233,128,0,0,17,1,116,130,0,0,1,19,9,0,0,0,29,0,0,0,1,0,17,1,233,128,0,0,1,19, 
  27,0,0,0,75,0,0,0,1,0,17,1,233,128,0,0,1,19,27,0,0,0,74,0,0,0,1,0,17,1,233,128,0,0,1,2,21,1,210,2,0,0,179,58,1,0,23,0,0,0,4,0,0,0,16,8,138,133,1,112, 
  195,131,2,112,3,128,3,120,149,131,4,96,20,128,5,72,19,128,6,208,18,128,23,248,7,128,8,48,146,130,9,144,145,130,10,120,16,128,11,96,15,128,12,72,14,128,13,216,12,128,14,192,11,128,15,168,138,129,17,104, 
  9,129,19,200,8,128,24,40,7,128,25,48,6,128,33,104,5,128,47,200,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,23,147,0,0,15,1,173,204, 
  0,0,17,1,31,33,1,0,1,4,15,1,23,147,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,160,143,0,0,17,1,71,133,0,0,1,4,19,24,0,0,0, 
  63,0,0,0,1,0,15,1,23,147,0,0,15,1,42,99,0,0,17,1,247,98,0,0,1,4,15,1,23,147,0,0,15,1,174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,23,147,0,0,15,1, 
  174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,23,147,0,0,15,1,162,122,0,0,17,1,195,112,0,0,1,4,15,1,23,147,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1, 
  23,147,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,10,0,0,0,36,0, 
  0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1, 
  0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0, 
  19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,15, 
  1,23,147,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,23,147,0,0,15,1,162,122,0,0,17,1,241,88,0,0,1,4,19,6,0,0,0,20,0,0,0,5,0,14,14,1,4,19,3,0,0,0,13, 
  0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,30,0,0,0, 
  2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,2,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68, 
  129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,16,143,0,0,15,1,43,206,0,0,17,1,185, 
  204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,16,143,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1, 
  4,17,1,221,133,0,0,1,2,21,1,54,3,0,0,179,58,1,0,23,0,0,0,4,0,0,0,16,248,138,133,1,112,195,131,2,112,3,128,3,104,152,131,4,32,23,128,5,216,21,128,6,112,21,128,23,88,8,128,8, 
  160,148,130,9,208,147,130,10,136,18,128,11,64,17,128,12,248,15,128,13,88,14,128,14,16,13,128,15,200,139,129,17,40,10,129,19,88,9,128,24,88,7,128,25,144,6,128,33,200,5,128,47,248,4,128,48,120,3,128,8, 
  4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,39,198,0,0,15,1,161,204,0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,39,198,0,0,15,1,161,204,0, 
  0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,39,198,0,0,17,1,20,137,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,39,198,0,0,17,1,52, 
  92,0,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,174,122,0,0,15,1,167,28,1, 
  0,17,1,12,28,1,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,162,122,0,0,17,1,195,112,0,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1, 
  4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0, 
  17,1,39,198,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0, 
  19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0, 
  0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0, 
  0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,204,102, 
  0,0,17,1,234,219,0,0,1,4,15,1,39,198,0,0,15,1,161,204,0,0,15,1,162,122,0,0,17,1,241,88,0,0,1,4,19,6,0,0,0,23,0,0,0,3,0,1,4,19,3,0,0,0,13,0,0,0,1,0, 
  19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0, 
  0,1,0,17,1,39,198,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,2,21,1,149,0,0,0,247,57, 
  1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1, 
  170,137,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,170,137,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4, 
  15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,221,133,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,40,2,128,33,168,3,128,26,144,1,128,11,32,2,128,44,184,2,128, 
  29,72,3,128,6,32,2,128,7,16,4,128,19,7,0,0,0,24,0,0,0,1,0,17,1,170,137,0,0,1,1,19,33,0,0,0,97,0,0,0,1,0,17,1,170,137,0,0,1,19,33,0,0,0,98,0,0,0,1,0, 
  17,1,170,137,0,0,1,19,11,0,0,0,38,0,0,0,2,0,1,15,1,170,137,0,0,17,1,185,204,0,0,1,15,1,170,137,0,0,17,1,58,138,0,0,1,2,21,1,119,0,0,0,181,68,1,0,5,0,0,0, 
  2,0,0,0,8,120,3,129,1,48,129,128,2,48,1,128,25,88,2,128,48,56,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,64,149,0,0,17,1,185,204,0,0, 
  1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,64,149,0,0,17,1,185,204,0,0,1,4,17,1,178,138,0,0,1,2,21,1,55,3,0,0,179,58,1,0,23,0,0,0, 
  4,0,0,0,16,248,138,133,1,112,195,131,2,112,3,128,3,112,152,131,4,40,23,128,5,224,21,128,6,112,21,128,23,88,8,128,8,160,148,130,9,208,147,130,10,136,18,128,11,64,17,128,12,248,15,128,13,88,14,128, 
  14,16,13,128,15,200,139,129,17,40,10,129,19,88,9,128,24,88,7,128,25,144,6,128,33,200,5,128,47,248,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0, 
  15,1,233,128,0,0,15,1,161,204,0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0, 
  0,1,0,15,1,233,128,0,0,17,1,234,141,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,233,128,0,0,17,1,232,93,0,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,174,122,0,0, 
  15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1, 
  162,122,0,0,17,1,195,112,0,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,216,102,0,0,17,1,35,220, 
  0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0, 
  0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19, 
  9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19, 
  10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0,0,0, 
  1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,233,128,0,0,15,1,161,204,0,0,15, 
  1,162,122,0,0,17,1,241,88,0,0,1,4,19,6,0,0,0,22,0,0,0,4,0,14,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0, 
  17,1,233,128,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0, 
  19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,28,0,0,0,1,0,17,1,233,128,0,0,1,2,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128, 
  17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,128,142,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0, 
  0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,128,142,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,221,133,0,0,1,2, 
  21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,136,2,128,33,16,4,128,26,144,1,128,11,24,3,128,44,128,3,128,29,32,3,128,6,24,3,128,7,32,2,128,19,7,0,0,0,24,0,0,0,1, 
  0,17,1,128,142,0,0,1,15,1,128,142,0,0,17,1,58,138,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,128,142,0,0,1,1,19,11,0,0,0,38,0,0,0,2,0,1,19,33,0,0,0,98,0,0, 
  0,1,0,17,1,128,142,0,0,1,15,1,128,142,0,0,17,1,185,204,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,144,1,128,33,32,2,128,26,232,3,128,11,240,2,128,44,88, 
  3,128,29,248,2,128,6,240,2,128,7,136,2,128,19,33,0,0,0,97,0,0,0,1,0,17,1,16,143,0,0,1,15,1,16,143,0,0,17,1,185,204,0,0,1,15,1,16,143,0,0,17,1,58,138,0,0,1,1,19, 
  11,0,0,0,38,0,0,0,2,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,16,143,0,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,16,143,0,0,1,2,21,0,114,1,0,0,255,255,255,255,18,0, 
  0,0,4,0,0,0,21,168,6,128,33,128,4,128,27,80,5,128,3,0,11,128,20,56,7,128,5,112,202,126,6,16,74,130,23,24,6,128,24,176,5,130,9,8,10,128,10,120,9,128,11,232,200,125,12,88,72,128,28,232, 
  4,129,14,200,7,128,38,240,3,128,40,96,3,128,44,208,2,128,19,33,0,0,0,98,0,0,0,1,0,17,1,160,143,0,0,1,19,27,0,0,0,81,0,0,0,1,0,17,1,160,143,0,0,1,19,27,0,0,0,78, 
  0,0,0,1,0,17,1,160,143,0,0,1,15,1,160,143,0,0,17,1,19,145,0,0,1,15,1,160,143,0,0,17,1,71,133,0,0,1,19,9,0,0,0,30,0,0,0,2,0,1,15,1,160,143,0,0,17,1,232,93, 
  0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,160,143,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,160,143,0,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,160,143,0,0,1,19,27,0, 
  0,0,79,0,0,0,1,0,17,1,160,143,0,0,1,19,27,0,0,0,80,0,0,0,1,0,17,1,160,143,0,0,1,19,27,0,0,0,77,0,0,0,1,0,17,1,160,143,0,0,1,19,27,0,0,0,76,0,0,0, 
  1,0,17,1,160,143,0,0,1,1,19,9,0,0,0,31,0,0,0,2,0,1,19,27,0,0,0,75,0,0,0,1,0,17,1,160,143,0,0,1,19,27,0,0,0,74,0,0,0,1,0,17,1,160,143,0,0,1,2,21, 
  1,235,0,0,0,198,60,1,0,9,0,0,0,3,0,0,0,40,192,5,128,1,80,199,129,2,80,7,128,35,48,4,128,36,32,5,128,37,176,6,128,38,80,2,128,39,64,3,128,41,176,1,128,4,15,1,10,146,0,0, 
  15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,10,146,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0, 
  41,0,0,0,1,0,17,1,10,146,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,10,146,0,0,1,4,15,1,10,146,0,0,15,1,95,68,0,0,17,1,145,44, 
  0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,10,146,0,0,1,4,15,1,10,146,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,8,19,12,0,0,0,40, 
  0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,10,146,0,0,1,15,1,10,146,0,0, 
  17,1,73,146,0,0,1,1,2,21,1,193,0,0,0,198,60,1,0,9,0,0,0,3,0,0,0,40,240,3,128,1,176,196,129,2,176,4,128,35,176,1,128,36,184,4,128,37,152,5,128,38,112,2,128,39,48,3,128,41, 
  40,5,128,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,92,0,0, 
  0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,145,44,0,0,1,4,15,1,197,16,0, 
  0,17,1,221,8,0,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,19,9,0,0,0,30,0,0,0,2,0,1,21,1,149,0,0,0,247,57,1,0,6,0,0, 
  0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,185,147,0,0,15, 
  1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,185,147,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0, 
  0,17,1,86,9,0,0,1,4,17,1,221,133,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,240,1,128,33,240,2,128,26,232,3,128,11,128,2,128,44,88,3,128,29,144,1,128,6, 
  128,2,128,7,136,2,128,19,11,0,0,0,38,0,0,0,2,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,185,147,0,0,1,1,15,1,185,147,0,0,17,1,58,138,0,0,1,15,1,185,147,0,0,17,1,185, 
  204,0,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,185,147,0,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,185,147,0,0,1,2,21,1,235,0,0,0,198,60,1,0,9,0,0,0,3,0,0,0,40, 
  144,3,128,1,80,199,129,2,80,7,128,35,128,4,128,36,80,2,128,37,240,2,128,38,96,6,128,39,112,5,128,41,176,1,128,4,15,1,10,146,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,10,146, 
  0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,15,1,10,146,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1, 
  10,146,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,10,146,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1, 
  10,146,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,10,146,0,0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,19,7,0,0,0,25,0,0,0,2,0,1, 
  21,1,5,2,0,0,37,69,1,0,15,0,0,0,3,0,0,0,48,24,72,131,1,112,130,128,2,112,194,129,17,104,78,129,52,40,7,128,13,216,78,129,22,104,77,129,23,104,76,129,25,184,138,129,26,200,9,128,53,56, 
  6,128,54,72,5,128,55,88,4,128,56,104,3,128,57,120,2,128,8,4,19,57,0,0,0,163,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,17,1,149,159,0,0,1,4,19,57,0,0,0,162,0,0,0,1, 
  0,19,55,0,0,0,156,0,0,0,1,0,17,1,149,159,0,0,1,4,19,57,0,0,0,166,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,17,1,149,159,0,0,1,4,19,57,0,0,0,164,0,0,0,1, 
  0,19,55,0,0,0,156,0,0,0,1,0,17,1,149,159,0,0,1,4,19,57,0,0,0,160,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,17,1,149,159,0,0,1,4,19,57,0,0,0,165,0,0,0,1, 
  0,19,55,0,0,0,156,0,0,0,1,0,17,1,149,159,0,0,1,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,149,159,0,0,15,1,95,157,0,0,15,1,107,157,0,0, 
  15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,57,0,0,0,161,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,17,1,149,159,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0, 
  97,0,0,0,1,0,15,1,149,159,0,0,15,1,95,157,0,0,15,1,107,157,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,149,159,0,0,15,1,95,157,0,0,15,1,107,157,0,0,15,1,218,7, 
  0,0,17,1,12,28,1,0,1,4,15,1,149,159,0,0,15,1,95,157,0,0,15,1,107,157,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,15,1,149,159,0,0,17,1,82,151,0,0,1,4,19,20,0,0, 
  0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,15,1,149,159,0,0,15,1,95,157,0,0,17,1,107,157,0,0,1,2,21,1,183,1,0,0,202,75,1,0,14,0,0,0,3,0,0,0,48,248,7,131, 
  1,80,130,128,2,80,130,129,25,56,138,130,52,8,7,128,13,200,12,129,22,40,12,129,23,136,11,129,26,72,9,128,53,24,6,128,54,40,5,128,55,56,4,128,56,72,3,128,57,88,2,128,8,4,19,57,0,0,0,163, 
  0,0,0,1,0,19,56,0,0,0,157,0,0,0,1,0,17,1,10,153,0,0,1,4,19,57,0,0,0,162,0,0,0,1,0,19,56,0,0,0,157,0,0,0,1,0,17,1,10,153,0,0,1,4,19,57,0,0,0,166, 
  0,0,0,1,0,19,56,0,0,0,157,0,0,0,1,0,17,1,10,153,0,0,1,4,19,57,0,0,0,164,0,0,0,1,0,19,56,0,0,0,157,0,0,0,1,0,17,1,10,153,0,0,1,4,19,57,0,0,0,160, 
  0,0,0,1,0,19,56,0,0,0,157,0,0,0,1,0,17,1,10,153,0,0,1,4,19,57,0,0,0,165,0,0,0,1,0,19,56,0,0,0,157,0,0,0,1,0,17,1,10,153,0,0,1,4,19,44,0,0,0,129, 
  0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,10,153,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,57,0,0,0,161,0,0,0,1,0,19,56,0,0,0,157,0,0,0,1,0,17,1, 
  10,153,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,10,153,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,10,153,0,0,15,1,218,7,0,0, 
  17,1,12,28,1,0,1,4,15,1,10,153,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,10,153,0,0,1,2,21,0, 
  228,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,24,224,68,130,25,120,68,129,20,0,134,129,35,128,3,128,12,144,134,127,21,112,5,128,33,16,4,129,55,232,2,128,44,240,2,128,56,128,2,128,57,240,1,128, 
  19,56,0,0,0,157,0,0,0,1,0,17,1,10,153,0,0,1,15,1,10,153,0,0,17,1,91,155,0,0,1,1,19,33,0,0,0,98,0,0,0,1,0,17,1,10,153,0,0,1,19,25,0,0,0,72,0,0,0,1, 
  0,17,1,10,153,0,0,1,15,1,10,153,0,0,17,1,100,154,0,0,1,15,1,10,153,0,0,17,1,239,153,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,10,153,0,0,1,19,25,0,0,0,71,0,0, 
  0,1,0,17,1,10,153,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,10,153,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,10,153,0,0,1,2,21,7,78,0,0,0,255,255,255,255,5,0,0, 
  0,2,0,0,0,47,176,1,128,45,48,1,128,42,240,129,128,43,112,65,127,94,48,2,128,4,17,1,213,18,0,0,1,4,17,1,211,32,0,0,1,4,17,1,77,35,0,0,1,4,17,1,199,37,0,0,1,4,17,1, 
  117,11,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,57,0,0,0,159,0,0,0,1,0,1,21,1,235,0,0,0,75,56,1,0,9,0,0,0,3,0, 
  0,0,40,64,3,128,1,192,197,129,2,192,5,128,35,48,4,128,36,176,1,128,37,32,5,128,38,104,6,128,39,80,2,128,41,200,5,128,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19, 
  31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,19, 
  31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,4,15,1,200,17,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,8,4,15,1,200,17,0,0,15,1,95,68, 
  0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,200,17,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,7,48,0,0,0,255,255,255, 
  255,2,0,0,0,1,0,0,0,44,208,0,128,41,16,1,128,4,17,1,167,155,0,0,1,4,19,55,0,0,0,155,0,0,0,3,0,14,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0, 
  128,5,208,0,128,8,2,21,1,171,1,0,0,202,75,1,0,14,0,0,0,3,0,0,0,48,216,6,131,1,80,130,128,2,80,130,129,25,24,137,130,52,24,6,128,13,56,12,129,22,104,11,129,23,152,10,129,26,88,8, 
  128,53,88,5,128,54,152,4,128,55,216,3,128,56,24,3,128,57,88,2,128,8,4,19,57,0,0,0,163,0,0,0,1,0,19,56,0,0,0,158,0,0,0,3,0,1,4,19,57,0,0,0,162,0,0,0,1,0,19,56, 
  0,0,0,158,0,0,0,3,0,1,4,19,57,0,0,0,166,0,0,0,1,0,19,56,0,0,0,158,0,0,0,3,0,1,4,19,57,0,0,0,164,0,0,0,1,0,19,56,0,0,0,158,0,0,0,3,0,1,4,19, 
  57,0,0,0,160,0,0,0,1,0,19,56,0,0,0,158,0,0,0,3,0,1,4,19,57,0,0,0,165,0,0,0,1,0,19,56,0,0,0,158,0,0,0,3,0,1,4,19,44,0,0,0,129,0,0,0,1,0,19,33, 
  0,0,0,98,0,0,0,1,0,15,1,83,157,0,0,15,1,107,157,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,57,0,0,0,161,0,0,0,1,0,19,56,0,0,0,158,0,0,0,3,0,1,4,19, 
  24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,83,157,0,0,15,1,107,157,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,83,157,0,0,15,1,107,157,0,0,15, 
  1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,83,157,0,0,15,1,107,157,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1, 
  0,15,1,83,157,0,0,17,1,107,157,0,0,1,2,19,56,0,0,0,158,0,0,0,3,0,1,19,55,0,0,0,156,0,0,0,1,0,1,21,0,189,0,0,0,255,255,255,255,9,0,0,0,3,0,0,0,24,168,3, 
  128,25,64,67,129,20,200,68,129,35,72,2,128,12,88,133,127,21,56,4,128,33,216,130,128,44,184,1,128,57,176,1,128,1,19,33,0,0,0,98,0,0,0,1,0,17,1,107,157,0,0,1,19,25,0,0,0,72,0,0, 
  0,1,0,17,1,107,157,0,0,1,15,1,107,157,0,0,17,1,158,158,0,0,1,15,1,107,157,0,0,17,1,41,158,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,107,157,0,0,1,19,25,0,0,0,71, 
  0,0,0,1,0,17,1,107,157,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,107,157,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,107,157,0,0,1,2,21,7,78,0,0,0,255,255,255,255,5, 
  0,0,0,2,0,0,0,47,48,1,128,45,112,1,128,42,240,129,128,43,48,66,127,94,176,1,128,4,17,1,206,20,0,0,1,4,17,1,114,26,0,0,1,4,17,1,62,13,0,0,1,4,17,1,166,23,0,0,1,4, 
  17,1,236,28,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,57,0,0,0,159,0,0,0,1,0,1,21,1,235,0,0,0,59,55,1,0,9,0,0,0, 
  3,0,0,0,40,176,1,128,1,80,199,129,2,80,7,128,35,96,6,128,36,64,3,128,37,208,4,128,38,224,3,128,39,112,5,128,41,160,2,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0, 
  0,1,0,17,1,184,15,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,91,0,0,0, 
  1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,4,15,1,184,15,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0, 
  0,1,0,17,1,184,15,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,184,15,0,0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,21,7,30,0,0,0,255, 
  255,255,255,1,0,0,0,0,0,0,0,123,176,0,128,4,17,1,207,159,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,2,21,1,82,2,0,0,148,76,1, 
  0,17,0,0,0,4,0,0,0,48,152,10,128,1,176,130,128,2,176,2,128,17,200,15,128,52,72,9,128,53,248,7,128,6,40,82,129,23,136,206,129,8,136,209,129,25,56,205,129,26,232,11,128,22,40,79,128,54,168,6, 
  128,13,152,16,128,55,88,5,128,56,8,4,128,57,184,2,128,8,4,19,57,0,0,0,163,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,15,1,164,179,0,0,15,1,15,198,0,0,17,1,198,170,0,0,1, 
  4,19,57,0,0,0,162,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,15,1,164,179,0,0,15,1,15,198,0,0,17,1,198,170,0,0,1,4,19,57,0,0,0,166,0,0,0,1,0,19,55,0,0,0,156, 
  0,0,0,1,0,15,1,164,179,0,0,15,1,15,198,0,0,17,1,198,170,0,0,1,4,19,57,0,0,0,164,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,15,1,164,179,0,0,15,1,15,198,0,0,17, 
  1,198,170,0,0,1,4,19,57,0,0,0,160,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,15,1,164,179,0,0,15,1,15,198,0,0,17,1,198,170,0,0,1,4,19,57,0,0,0,165,0,0,0,1,0, 
  19,55,0,0,0,156,0,0,0,1,0,15,1,164,179,0,0,15,1,15,198,0,0,17,1,198,170,0,0,1,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,164,179,0,0,15, 
  1,24,85,0,0,17,1,31,33,1,0,1,4,19,57,0,0,0,161,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,15,1,164,179,0,0,15,1,15,198,0,0,17,1,198,170,0,0,1,4,19,24,0,0,0, 
  63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,164,179,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,164,179,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,164, 
  179,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,15,1,164,179,0,0,15,1,15,198,0,0,15,1,198,170,0,0,17,1,82,151,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69, 
  0,0,0,1,0,17,1,164,179,0,0,1,4,15,1,164,179,0,0,15,1,15,198,0,0,17,1,34,162,0,0,1,4,19,38,0,0,0,114,0,0,0,4,0,1,2,21,1,66,3,0,0,179,58,1,0,23,0,0,0, 
  4,0,0,0,16,88,139,133,1,112,195,131,2,112,3,128,3,200,152,131,4,128,23,128,5,56,22,128,6,208,21,128,23,184,8,128,8,0,149,130,9,48,148,130,10,232,18,128,11,160,17,128,12,88,16,128,13,184,14,128, 
  14,112,13,128,15,40,140,129,17,136,10,129,19,184,9,128,24,184,7,128,25,240,6,128,33,200,5,128,47,248,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0, 
  15,1,101,165,0,0,15,1,186,170,0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,101,165,0,0,15,1,186,170,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0, 
  0,1,0,15,1,101,165,0,0,15,1,186,170,0,0,15,1,103,92,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,101,165,0,0,17,1,232,93,0,0,1,4,15,1,101,165,0,0, 
  15,1,186,170,0,0,15,1,174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,101,165,0,0,15,1,186,170,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1, 
  101,165,0,0,15,1,186,170,0,0,15,1,162,122,0,0,17,1,195,112,0,0,1,4,15,1,101,165,0,0,15,1,186,170,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,101,165,0,0,15,1,186,170, 
  0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,54,0,0,0,153,0,0,0,1,0,17,1,101,165,0,0,1,4,19,10,0, 
  0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,54,0,0,0,153,0,0,0,1,0,17,1,101,165,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0, 
  19,27,0,0,0,74,0,0,0,1,0,19,54,0,0,0,153,0,0,0,1,0,17,1,101,165,0,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,54,0,0,0,153,0,0, 
  0,1,0,17,1,101,165,0,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,54,0,0,0,153,0,0,0,1,0,17,1,101,165,0,0,1,4,19,10,0,0,0,32,0,0, 
  0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,54,0,0,0,153,0,0,0,1,0,17,1,101,165,0,0,1,4,15,1,101,165,0,0,15,1,186,170,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15, 
  1,101,165,0,0,15,1,186,170,0,0,15,1,162,122,0,0,17,1,241,88,0,0,1,4,19,50,0,0,0,148,0,0,0,2,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0, 
  19,54,0,0,0,153,0,0,0,1,0,17,1,101,165,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,54,0,0,0,153,0,0,0,1,0,17,1,101,165,0,0,1,4, 
  19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,54,0,0,0,153,0,0,0,1,0,17,1,101,165,0,0,1,2,21,0,121,1,0,0,255,255,255,255,18,0,0,0,4,0,0,0,21, 
  72,7,128,33,240,4,128,50,56,3,128,3,56,11,128,20,216,7,128,5,168,202,126,38,96,196,130,23,184,6,128,24,80,198,129,27,192,5,128,10,24,10,128,11,136,137,127,12,248,72,128,28,88,197,128,14,104,8,128,40, 
  208,3,128,44,64,3,128,54,208,2,128,15,1,101,165,0,0,17,1,214,167,0,0,1,1,19,33,0,0,0,98,0,0,0,1,0,17,1,101,165,0,0,1,19,27,0,0,0,81,0,0,0,1,0,17,1,101,165,0,0, 
  1,19,27,0,0,0,78,0,0,0,1,0,17,1,101,165,0,0,1,15,1,101,165,0,0,17,1,223,166,0,0,1,15,1,101,165,0,0,17,1,22,9,0,0,1,19,54,0,0,0,153,0,0,0,1,0,17,1,101,165, 
  0,0,1,15,1,101,165,0,0,17,1,186,122,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,101,165,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,101,165,0,0,1,19,3,0,0,0,14,0,0, 
  0,1,0,17,1,101,165,0,0,1,19,27,0,0,0,79,0,0,0,1,0,17,1,101,165,0,0,1,19,27,0,0,0,80,0,0,0,1,0,17,1,101,165,0,0,1,19,27,0,0,0,77,0,0,0,1,0,17,1,101, 
  165,0,0,1,19,27,0,0,0,76,0,0,0,1,0,17,1,101,165,0,0,1,19,27,0,0,0,75,0,0,0,1,0,17,1,101,165,0,0,1,19,27,0,0,0,74,0,0,0,1,0,17,1,101,165,0,0,1,2,21, 
  1,235,0,0,0,198,60,1,0,9,0,0,0,3,0,0,0,40,96,6,128,1,80,199,129,2,80,7,128,35,64,3,128,36,160,2,128,37,208,4,128,38,112,5,128,39,176,1,128,41,48,4,128,4,19,31,0,0,0,92, 
  0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,18,95,0,0,1,4,15,1,18,95,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0, 
  41,0,0,0,1,0,17,1,18,95,0,0,1,4,15,1,18,95,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,18,95,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91, 
  0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,18,95,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,18,95,0,0,1,8,19,12,0,0,0,40, 
  0,0,0,1,0,1,21,1,215,2,0,0,179,58,1,0,23,0,0,0,4,0,0,0,16,56,138,133,1,112,195,131,2,112,3,128,3,160,149,131,4,136,20,128,5,112,19,128,6,0,19,128,23,40,8,128,8,96,146,130, 
  9,192,145,130,10,168,16,128,11,144,15,128,12,120,14,128,13,8,13,128,14,240,11,128,15,216,138,129,17,152,9,129,19,248,8,128,24,88,7,128,25,96,6,128,33,104,5,128,47,200,4,128,48,120,3,128,8,4,19,44, 
  0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,174,170,0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,174,170,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4, 
  19,28,0,0,0,82,0,0,0,1,0,15,1,174,170,0,0,15,1,103,92,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,174,170,0,0,15,1,42,99,0,0,17,1,247,98,0,0, 
  1,4,15,1,174,170,0,0,15,1,174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,174,170,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,174,170,0,0, 
  15,1,162,122,0,0,17,1,195,112,0,0,1,4,15,1,174,170,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,174,170,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37, 
  0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,54,0,0,0,154,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,54,0,0,0,154,0,0,0, 
  2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,54,0,0,0,154,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1, 
  0,19,27,0,0,0,76,0,0,0,1,0,19,54,0,0,0,154,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,54,0,0,0,154,0,0,0,2,0,1,4, 
  19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,54,0,0,0,154,0,0,0,2,0,1,4,15,1,174,170,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,174,170,0, 
  0,15,1,162,122,0,0,17,1,241,88,0,0,1,4,19,50,0,0,0,147,0,0,0,3,0,14,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,54,0,0,0,154,0,0,0, 
  2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,54,0,0,0,154,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1, 
  0,19,54,0,0,0,154,0,0,0,2,0,1,2,19,54,0,0,0,154,0,0,0,2,0,1,19,54,0,0,0,153,0,0,0,1,0,1,21,7,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,123,176,0,128, 
  4,17,1,0,171,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,2,21,1,66,3,0,0,179,58,1,0,23,0,0,0,4,0,0,0,16,88,139,133,1,112, 
  195,131,2,112,3,128,3,200,152,131,4,128,23,128,5,56,22,128,6,208,21,128,23,184,8,128,8,0,149,130,9,48,148,130,10,232,18,128,11,160,17,128,12,88,16,128,13,184,14,128,14,112,13,128,15,40,140,129,17,136, 
  10,129,19,184,9,128,24,184,7,128,25,240,6,128,33,200,5,128,47,248,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,67,174,0,0,15,1,152,179, 
  0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,67,174,0,0,15,1,152,179,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,67,174,0,0,15, 
  1,152,179,0,0,15,1,103,92,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,67,174,0,0,17,1,232,93,0,0,1,4,15,1,67,174,0,0,15,1,152,179,0,0,15,1,174,122, 
  0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,67,174,0,0,15,1,152,179,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,67,174,0,0,15,1,152,179,0,0, 
  15,1,162,122,0,0,17,1,195,112,0,0,1,4,15,1,67,174,0,0,15,1,152,179,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,67,174,0,0,15,1,152,179,0,0,15,1,216,102,0,0,17,1, 
  35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,52,0,0,0,150,0,0,0,1,0,17,1,67,174,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27, 
  0,0,0,76,0,0,0,1,0,19,52,0,0,0,150,0,0,0,1,0,17,1,67,174,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1, 
  0,19,52,0,0,0,150,0,0,0,1,0,17,1,67,174,0,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,52,0,0,0,150,0,0,0,1,0,17,1,67,174,0,0,1, 
  4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,52,0,0,0,150,0,0,0,1,0,17,1,67,174,0,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,27,0,0,0,76,0, 
  0,0,1,0,19,52,0,0,0,150,0,0,0,1,0,17,1,67,174,0,0,1,4,15,1,67,174,0,0,15,1,152,179,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,67,174,0,0,15,1,152,179,0, 
  0,15,1,162,122,0,0,17,1,241,88,0,0,1,4,19,50,0,0,0,146,0,0,0,3,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,52,0,0,0,150,0,0,0,1, 
  0,17,1,67,174,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,52,0,0,0,150,0,0,0,1,0,17,1,67,174,0,0,1,4,19,3,0,0,0,11,0,0,0,1, 
  0,19,27,0,0,0,74,0,0,0,1,0,19,52,0,0,0,150,0,0,0,1,0,17,1,67,174,0,0,1,2,21,0,121,1,0,0,255,255,255,255,18,0,0,0,4,0,0,0,21,72,7,128,33,240,4,128,50,56,3, 
  128,3,56,11,128,20,216,71,131,5,168,202,126,38,96,4,128,23,184,6,128,24,80,198,129,27,192,5,128,10,24,10,128,11,136,137,127,12,248,72,128,28,88,197,128,14,104,8,128,40,208,3,128,44,64,3,128,52,208,2, 
  128,15,1,67,174,0,0,17,1,180,176,0,0,1,1,19,33,0,0,0,98,0,0,0,1,0,17,1,67,174,0,0,1,19,27,0,0,0,81,0,0,0,1,0,17,1,67,174,0,0,1,19,27,0,0,0,78,0,0,0, 
  1,0,17,1,67,174,0,0,1,15,1,67,174,0,0,17,1,189,175,0,0,1,15,1,67,174,0,0,17,1,22,9,0,0,1,19,52,0,0,0,150,0,0,0,1,0,17,1,67,174,0,0,1,15,1,67,174,0,0,17, 
  1,232,93,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,67,174,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,67,174,0,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,67,174,0,0,1, 
  19,27,0,0,0,79,0,0,0,1,0,17,1,67,174,0,0,1,19,27,0,0,0,80,0,0,0,1,0,17,1,67,174,0,0,1,19,27,0,0,0,77,0,0,0,1,0,17,1,67,174,0,0,1,19,27,0,0,0,76, 
  0,0,0,1,0,17,1,67,174,0,0,1,19,27,0,0,0,75,0,0,0,1,0,17,1,67,174,0,0,1,19,27,0,0,0,74,0,0,0,1,0,17,1,67,174,0,0,1,2,21,1,235,0,0,0,198,60,1,0,9, 
  0,0,0,3,0,0,0,40,112,5,128,1,96,198,129,2,96,6,128,35,176,1,128,36,160,2,128,37,48,4,128,38,64,3,128,39,104,6,128,41,208,4,128,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0, 
  41,0,0,0,1,0,17,1,18,95,0,0,1,4,15,1,18,95,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,18,95, 
  0,0,1,4,15,1,18,95,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,18,95,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0, 
  41,0,0,0,1,0,17,1,18,95,0,0,1,8,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,18,95,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,215,2, 
  0,0,179,58,1,0,23,0,0,0,4,0,0,0,16,56,138,133,1,112,195,131,2,112,3,128,3,160,149,131,4,136,20,128,5,112,19,128,6,0,19,128,23,40,8,128,8,96,146,130,9,192,145,130,10,168,16,128,11,144, 
  15,128,12,120,14,128,13,8,13,128,14,240,11,128,15,216,138,129,17,152,9,129,19,248,8,128,24,88,7,128,25,96,6,128,33,104,5,128,47,200,4,128,48,120,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19, 
  33,0,0,0,98,0,0,0,1,0,15,1,140,179,0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,140,179,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0,0,82,0,0,0,1, 
  0,15,1,140,179,0,0,15,1,103,92,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,140,179,0,0,15,1,42,99,0,0,17,1,247,98,0,0,1,4,15,1,140,179,0,0,15,1, 
  174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,140,179,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,140,179,0,0,15,1,162,122,0,0,17,1,195,112, 
  0,0,1,4,15,1,140,179,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,140,179,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0, 
  76,0,0,0,1,0,19,52,0,0,0,151,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,52,0,0,0,151,0,0,0,2,0,1,4,19,20,0,0,0,58, 
  0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,52,0,0,0,151,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0, 
  1,0,19,52,0,0,0,151,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,52,0,0,0,151,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1, 
  0,19,27,0,0,0,76,0,0,0,1,0,19,52,0,0,0,151,0,0,0,2,0,1,4,15,1,140,179,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,140,179,0,0,15,1,162,122,0,0,17,1,241, 
  88,0,0,1,4,19,50,0,0,0,145,0,0,0,4,0,14,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,52,0,0,0,151,0,0,0,2,0,1,4,19,3,0,0,0,12, 
  0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,52,0,0,0,151,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,52,0,0,0,151,0,0,0, 
  2,0,1,2,19,52,0,0,0,151,0,0,0,2,0,1,19,52,0,0,0,150,0,0,0,1,0,1,21,0,11,1,0,0,255,255,255,255,13,0,0,0,3,0,0,0,24,24,6,128,25,176,5,130,50,40,3,128,35,184, 
  4,128,12,200,7,129,21,168,6,128,38,176,4,128,39,72,4,129,20,56,135,128,33,72,197,128,44,184,3,128,55,192,2,128,57,48,2,128,19,55,0,0,0,156,0,0,0,1,0,17,1,164,179,0,0,1,15,1,164,179, 
  0,0,17,1,198,170,0,0,1,19,39,0,0,0,115,0,0,0,1,0,17,1,164,179,0,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,164,179,0,0,1,15,1,164,179,0,0,17,1,139,195,0,0,1,1,19, 
  25,0,0,0,72,0,0,0,1,0,17,1,164,179,0,0,1,15,1,164,179,0,0,17,1,148,194,0,0,1,15,1,164,179,0,0,17,1,176,180,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,164,179,0,0, 
  1,19,25,0,0,0,71,0,0,0,1,0,17,1,164,179,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,164,179,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,164,179,0,0,1,2,21,7,78,0, 
  0,0,255,255,255,255,5,0,0,0,2,0,0,0,47,48,2,128,45,240,1,128,42,176,129,128,43,112,65,127,94,48,1,128,4,17,1,26,192,0,0,1,4,17,1,160,189,0,0,1,4,17,1,38,187,0,0,1,4,17, 
  1,172,184,0,0,1,4,17,1,37,181,0,0,1,21,9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,19,57,0,0,0,159,0,0,0,1,0,1,21,1,201,0,0,0,108, 
  49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0, 
  0,1,0,15,1,239,181,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,239,181,0,0,15,1,24,85,0,0,17,1,230, 
  7,0,0,1,4,15,1,239,181,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,239,181,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0, 
  0,69,0,0,0,1,0,17,1,239,181,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2, 
  128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,239,181,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,239,181,0,0,1,15,1,239,181,0,0,17,1,168,182,0,0,1,15,1,239,181,0,0, 
  17,1,81,22,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,239,181,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,239,181,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,239,181,0,0, 
  1,19,25,0,0,0,70,0,0,0,1,0,17,1,239,181,0,0,1,2,21,1,235,0,0,0,127,77,1,0,9,0,0,0,3,0,0,0,40,96,6,128,1,80,199,129,2,80,7,128,35,176,1,128,36,48,4,128,37,192, 
  5,128,38,208,4,128,39,160,2,128,41,144,3,128,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0, 
  0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0, 
  91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0, 
  0,41,0,0,0,1,0,17,1,159,183,0,0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0, 
  0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,15,1,159,183,0,0,17,1,222,183,0,0,1,1,2,21,1,193,0,0,0,127,77,1,0,9,0,0,0,3,0,0,0,40,160,3,128,1,0,198,129,2,0,6,128, 
  35,208,4,128,36,144,5,128,37,96,4,128,38,176,1,128,39,224,2,128,41,112,2,128,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,221,8,0, 
  0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,251, 
  60,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,145,44,0,0,1,8,19,12,0,0,0,39,0,0,0,2,0,14,1,21,1,201,0, 
  0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0, 
  98,0,0,0,1,0,15,1,118,185,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,118,185,0,0,15,1,24,85,0,0, 
  17,1,230,7,0,0,1,4,15,1,118,185,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,118,185,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19, 
  25,0,0,0,69,0,0,0,1,0,17,1,118,185,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128, 
  33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,118,185,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,118,185,0,0,1,15,1,118,185,0,0,17,1,47,186,0,0,1,15,1,118, 
  185,0,0,17,1,88,20,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,118,185,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,118,185,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,118, 
  185,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,118,185,0,0,1,2,21,1,235,0,0,0,127,77,1,0,9,0,0,0,3,0,0,0,40,48,4,128,1,80,199,129,2,80,7,128,35,80,2,128,36,176,6, 
  128,37,176,1,128,38,32,5,128,39,64,3,128,41,16,6,128,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0, 
  17,1,159,183,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0, 
  17,1,159,183,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1, 
  159,183,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128, 
  23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,240,187,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4, 
  19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,240,187,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,240,187,0,0,15,1,218,7,0,0,17,1,12,28,1,0, 
  1,4,15,1,240,187,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,240,187,0,0,1,2,21,0,184,0,0,0,255,255, 
  255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,240,187,0,0,1, 
  19,25,0,0,0,72,0,0,0,1,0,17,1,240,187,0,0,1,15,1,240,187,0,0,17,1,169,188,0,0,1,15,1,240,187,0,0,17,1,41,25,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,240,187,0, 
  0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,240,187,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,240,187,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,240,187,0,0,1,2,21,1,235, 
  0,0,0,127,77,1,0,9,0,0,0,3,0,0,0,40,48,4,128,1,96,198,129,2,96,6,128,35,64,3,128,36,32,5,128,37,192,5,128,38,80,2,128,39,104,6,128,41,176,1,128,4,15,1,159,183,0,0,15,1, 
  95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0, 
  0,0,1,0,17,1,159,183,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,145,44,0,0, 
  1,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,8,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,19,12,0,0,0,40,0,0, 
  0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0, 
  0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,106,190,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,106,190, 
  0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,106,190,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,106,190,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0, 
  0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,106,190,0,0,1,2,21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128, 
  12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1,0,17,1,106,190,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,106,190,0,0,1,15,1,106,190,0,0,17,1, 
  35,191,0,0,1,15,1,106,190,0,0,17,1,111,30,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,106,190,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,106,190,0,0,1,19,25,0,0,0,69, 
  0,0,0,1,0,17,1,106,190,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,106,190,0,0,1,2,21,1,235,0,0,0,127,77,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1,32,197,129,2,32,5, 
  128,35,48,4,128,36,160,2,128,37,40,5,128,38,104,6,128,39,64,3,128,41,200,5,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,15,1,159,183, 
  0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0, 
  0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,8,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0, 
  0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0,0,48,120,1,128, 
  1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,228,192,0,0,15,1,24,85,0,0, 
  17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,228,192,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,228,192,0,0,15,1,218,7, 
  0,0,17,1,12,28,1,0,1,4,15,1,228,192,0,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,228,192,0,0,1,2, 
  21,0,184,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,25,24,67,129,20,160,68,129,35,32,2,128,12,48,133,127,21,16,4,128,33,176,2,128,44,144,1,128,19,33,0,0,0,98,0,0,0,1, 
  0,17,1,228,192,0,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,228,192,0,0,1,15,1,228,192,0,0,17,1,157,193,0,0,1,15,1,228,192,0,0,17,1,248,12,0,0,1,19,33,0,0,0,97,0,0, 
  0,1,0,17,1,228,192,0,0,1,19,25,0,0,0,71,0,0,0,1,0,17,1,228,192,0,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,228,192,0,0,1,19,25,0,0,0,70,0,0,0,1,0,17,1,228, 
  192,0,0,1,2,21,1,235,0,0,0,127,77,1,0,9,0,0,0,3,0,0,0,40,64,3,128,1,96,198,129,2,96,6,128,35,104,6,128,36,48,4,128,37,176,1,128,38,208,4,128,39,80,2,128,41,192,5,128,4, 
  15,1,159,183,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,19,31,0,0,0,93,0,0,0,1, 
  0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0, 
  1,0,17,1,159,183,0,0,1,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,8,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1, 
  19,12,0,0,0,40,0,0,0,1,0,1,21,1,235,0,0,0,127,77,1,0,9,0,0,0,3,0,0,0,40,48,4,128,1,80,199,129,2,80,7,128,35,80,2,128,36,176,1,128,37,16,6,128,38,64,3,128,39,32, 
  5,128,41,176,6,128,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,19,31,0, 
  0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,19,31,0, 
  0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,159,183,0,0,1,4,15,1,159,183,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,159,183,0,0,15,1,95,68,0,0,17, 
  1,221,8,0,0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,21,1,119,2,0,0,148,76,1,0,17,0,0,0,4,0,0,0,48,120,9,128,1,176,130,128,2,176,2,128,17,184,16,128,52,88,8,128,53,56,7, 
  128,6,72,83,129,23,88,206,129,8,216,210,129,25,120,204,129,26,88,11,128,22,136,79,128,54,24,6,128,13,88,17,128,55,248,4,128,56,216,3,128,57,184,2,128,8,4,19,57,0,0,0,163,0,0,0,1,0,19,55, 
  0,0,0,156,0,0,0,1,0,15,1,3,198,0,0,17,1,198,170,0,0,1,4,19,57,0,0,0,162,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,15,1,3,198,0,0,17,1,198,170,0,0,1,4,19, 
  57,0,0,0,166,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,15,1,3,198,0,0,17,1,198,170,0,0,1,4,19,57,0,0,0,164,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,15,1,3, 
  198,0,0,17,1,198,170,0,0,1,4,19,57,0,0,0,160,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,15,1,3,198,0,0,17,1,198,170,0,0,1,4,19,57,0,0,0,165,0,0,0,1,0,19,55, 
  0,0,0,156,0,0,0,1,0,15,1,3,198,0,0,17,1,198,170,0,0,1,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,3,198,0,0,15,1,198,170,0,0,15,1,95, 
  157,0,0,15,1,107,157,0,0,15,1,24,85,0,0,17,1,31,33,1,0,1,4,19,57,0,0,0,161,0,0,0,1,0,19,55,0,0,0,156,0,0,0,1,0,15,1,3,198,0,0,17,1,198,170,0,0,1,4,19, 
  24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,3,198,0,0,15,1,198,170,0,0,15,1,95,157,0,0,15,1,107,157,0,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15, 
  1,3,198,0,0,15,1,198,170,0,0,15,1,95,157,0,0,15,1,107,157,0,0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,3,198,0,0,15,1,198,170,0,0,15,1,95,157,0,0,15,1,107,157,0, 
  0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,15,1,3,198,0,0,15,1,198,170,0,0,17,1,82,151,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,15,1,3, 
  198,0,0,15,1,198,170,0,0,15,1,95,157,0,0,17,1,107,157,0,0,1,4,15,1,3,198,0,0,17,1,34,162,0,0,1,4,19,38,0,0,0,113,0,0,0,5,0,14,1,2,19,39,0,0,0,116,0,0,0, 
  2,0,1,19,39,0,0,0,115,0,0,0,1,0,1,19,27,0,0,0,78,0,0,0,1,0,1,21,0,138,1,0,0,255,255,255,255,18,0,0,0,4,0,0,0,21,216,6,128,33,128,4,128,27,80,5,128,3,192,11, 
  128,20,104,7,128,5,48,203,126,6,160,74,130,23,72,6,128,24,224,5,130,9,56,10,128,10,168,9,128,11,24,201,125,12,136,72,128,28,232,4,129,14,248,7,128,38,240,3,128,40,96,3,128,44,208,2,128,19,33,0, 
  0,0,98,0,0,0,1,0,17,1,39,198,0,0,1,19,27,0,0,0,81,0,0,0,1,0,17,1,39,198,0,0,1,19,27,0,0,0,78,0,0,0,1,0,17,1,39,198,0,0,1,15,1,39,198,0,0,17,1,170, 
  203,0,0,1,15,1,39,198,0,0,17,1,132,202,0,0,1,19,9,0,0,0,28,0,0,0,1,0,17,1,39,198,0,0,1,15,1,39,198,0,0,17,1,52,92,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17, 
  1,39,198,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,39,198,0,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,39,198,0,0,1,19,27,0,0,0,79,0,0,0,1,0,17,1,39,198,0,0,1, 
  19,27,0,0,0,80,0,0,0,1,0,17,1,39,198,0,0,1,19,27,0,0,0,77,0,0,0,1,0,17,1,39,198,0,0,1,19,27,0,0,0,76,0,0,0,1,0,17,1,39,198,0,0,1,15,1,39,198,0,0, 
  17,1,178,199,0,0,1,19,9,0,0,0,29,0,0,0,1,0,17,1,39,198,0,0,1,19,27,0,0,0,75,0,0,0,1,0,17,1,39,198,0,0,1,19,27,0,0,0,74,0,0,0,1,0,17,1,39,198,0,0, 
  1,2,21,1,209,2,0,0,179,58,1,0,23,0,0,0,4,0,0,0,16,8,138,133,1,112,195,131,2,112,3,128,3,112,149,131,4,88,20,128,5,64,19,128,6,208,18,128,23,248,7,128,8,48,146,130,9,144,145,130, 
  10,120,16,128,11,96,15,128,12,72,14,128,13,216,12,128,14,192,11,128,15,168,138,129,17,104,9,129,19,200,8,128,24,40,7,128,25,48,6,128,33,104,5,128,47,200,4,128,48,120,3,128,8,4,19,44,0,0,0,129, 
  0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,23,147,0,0,15,1,173,204,0,0,17,1,31,33,1,0,1,4,15,1,23,147,0,0,15,1,27,198,0,0,17,1,76,149,0,0,1,4,19,28,0,0, 
  0,82,0,0,0,1,0,15,1,160,143,0,0,17,1,71,133,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,23,147,0,0,15,1,42,99,0,0,17,1,247,98,0,0,1,4,15,1,23,147,0,0,15,1, 
  174,122,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,23,147,0,0,15,1,174,122,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,23,147,0,0,15,1,162,122,0,0,17,1,195,112, 
  0,0,1,4,15,1,23,147,0,0,15,1,162,122,0,0,17,1,228,102,0,0,1,4,15,1,23,147,0,0,15,1,216,102,0,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,27,0,0,0, 
  76,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,20,0,0,0,58, 
  0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,27,0,0,0,76,0,0,0, 
  1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1, 
  0,19,27,0,0,0,76,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,15,1,23,147,0,0,15,1,204,102,0,0,17,1,234,219,0,0,1,4,15,1,23,147,0,0,15,1,162,122,0,0,17,1,241, 
  88,0,0,1,4,19,6,0,0,0,21,0,0,0,4,0,14,1,4,19,3,0,0,0,13,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,3,0,0,0,12, 
  0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,30,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,27,0,0,0,74,0,0,0,1,0,19,9,0,0,0,30,0,0,0, 
  2,0,1,2,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19, 
  33,0,0,0,98,0,0,0,1,0,15,1,26,203,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,26,203,0,0,15,1, 
  43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,221,133,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,96,2,128,33,144,1,128, 
  26,232,3,128,11,240,2,128,44,88,3,128,29,248,2,128,6,240,2,128,7,248,1,128,15,1,26,203,0,0,17,1,185,204,0,0,1,15,1,26,203,0,0,17,1,58,138,0,0,1,19,33,0,0,0,97,0,0,0,1, 
  0,17,1,26,203,0,0,1,1,19,11,0,0,0,38,0,0,0,2,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,26,203,0,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,26,203,0,0,1,2,21,1, 
  235,0,0,0,198,60,1,0,9,0,0,0,3,0,0,0,40,208,4,128,1,176,198,129,2,176,6,128,35,240,2,128,36,176,1,128,37,184,6,128,38,192,5,128,39,224,3,128,41,80,2,128,4,15,1,10,146,0,0,15, 
  1,95,68,0,0,17,1,145,44,0,0,1,4,15,1,10,146,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,10,146,0, 
  0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,10,146,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,10,146,0, 
  0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,10,146,0,0,1,8,4,15,1,10,146,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,19,12,0,0,0,40,0, 
  0,0,1,0,1,19,9,0,0,0,28,0,0,0,1,0,1,19,27,0,0,0,80,0,0,0,1,0,1,21,7,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,58,176,0,128,4,17,1,243,204,0,0,1,21, 
  9,27,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,2,21,1,167,0,0,0,10,78,1,0,7,0,0,0,2,0,0,0,44,248,2,128,1,112,1,129,2,112,129,128,43,184,3, 
  128,42,120,132,128,45,56,2,128,46,120,1,128,8,4,19,37,0,0,0,110,0,0,0,1,0,19,26,0,0,0,73,0,0,0,3,0,1,4,19,37,0,0,0,111,0,0,0,1,0,19,26,0,0,0,73,0,0,0,3, 
  0,1,4,19,37,0,0,0,112,0,0,0,1,0,19,26,0,0,0,73,0,0,0,3,0,1,4,19,37,0,0,0,109,0,0,0,1,0,19,26,0,0,0,73,0,0,0,3,0,1,4,19,37,0,0,0,108,0,0,0, 
  1,0,19,26,0,0,0,73,0,0,0,3,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,136,3,128,33,0,2,128,26,104,2,128,11,144,1,128,44,248,2,128,29,24,4,128,6,144,1, 
  128,7,152,1,128,1,15,1,155,205,0,0,17,1,19,124,0,0,1,15,1,155,205,0,0,17,1,185,204,0,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,155,205,0,0,1,19,33,0,0,0,98,0,0,0,1, 
  0,17,1,155,205,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,155,205,0,0,1,19,11,0,0,0,38,0,0,0,2,0,1,2,19,7,0,0,0,24,0,0,0,1,0,1,21,0,116,1,0,0,255,255,255, 
  255,17,0,0,0,4,0,0,0,16,144,6,128,33,64,3,128,2,16,11,128,3,128,138,129,20,152,5,128,5,240,9,130,6,96,9,128,23,120,4,128,24,16,4,128,19,40,6,128,10,208,8,128,11,64,8,128,12,176,199, 
  128,21,8,5,128,14,32,7,128,28,168,67,128,44,176,2,128,19,33,0,0,0,98,0,0,0,1,0,17,1,55,206,0,0,1,15,1,55,206,0,0,17,1,218,217,0,0,1,15,1,55,206,0,0,17,1,180,216,0,0, 
  1,15,1,55,206,0,0,17,1,129,216,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,55,206,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,55,206,0,0,1,19,3,0,0,0,14,0,0,0,1, 
  0,17,1,55,206,0,0,1,15,1,55,206,0,0,17,1,172,207,0,0,1,19,2,0,0,0,10,0,0,0,1,0,17,1,55,206,0,0,1,19,2,0,0,0,9,0,0,0,1,0,17,1,55,206,0,0,1,19,2,0, 
  0,0,8,0,0,0,1,0,17,1,55,206,0,0,1,19,2,0,0,0,7,0,0,0,1,0,17,1,55,206,0,0,1,19,2,0,0,0,6,0,0,0,1,0,17,1,55,206,0,0,1,19,2,0,0,0,5,0,0,0, 
  1,0,17,1,55,206,0,0,1,19,2,0,0,0,4,0,0,0,1,0,17,1,55,206,0,0,1,19,2,0,0,0,3,0,0,0,1,0,17,1,55,206,0,0,1,19,19,0,0,0,56,0,0,0,1,0,17,1,55,206, 
  0,0,1,2,21,1,191,2,0,0,110,44,1,0,22,0,0,0,4,0,0,0,16,120,73,133,1,80,195,131,2,80,3,128,3,224,148,131,4,200,19,128,5,176,18,128,6,64,18,128,23,104,7,128,8,160,145,130,9,0, 
  145,130,10,232,15,128,11,208,14,128,12,184,13,128,13,72,12,128,14,48,11,128,15,24,10,128,17,216,8,129,19,56,8,128,24,152,6,128,25,160,5,128,33,168,4,128,48,88,3,128,8,4,19,44,0,0,0,129,0,0, 
  0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,117,216,0,0,15,1,139,41,1,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,117,216,0,0,15,1,197,211,0,0,17,1,159, 
  210,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,117,216,0,0,15,1,197,211,0,0,17,1,108,210,0,0,1,4,15,1,117,216,0,0,15,1,174,31,1,0,15,1,186,31,1,0,17,1,179,28,1,0, 
  1,4,15,1,117,216,0,0,15,1,174,31,1,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,117,216,0,0,15,1,0,28,1,0,17,1,23,16,1,0,1,4,15,1,117,216,0,0,15,1,0,28,1,0, 
  17,1,46,4,1,0,1,4,15,1,117,216,0,0,15,1,34,4,1,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,19,0,0,0,57,0,0,0,2, 
  0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,19,0,0,0,57,0,0,0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0, 
  19,2,0,0,0,3,0,0,0,1,0,19,19,0,0,0,57,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,19,0,0,0,57,0,0,0,2,0,1,4,19, 
  10,0,0,0,33,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,19,0,0,0,57,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,19,0,0, 
  0,57,0,0,0,2,0,1,4,15,1,117,216,0,0,15,1,23,220,0,0,17,1,234,219,0,0,1,4,15,1,117,216,0,0,15,1,0,28,1,0,17,1,29,3,0,0,1,4,19,16,0,0,0,50,0,0,0,3,0, 
  14,1,4,19,3,0,0,0,13,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,19,0,0,0,57,0,0,0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0, 
  19,19,0,0,0,57,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,19,0,0,0,57,0,0,0,2,0,1,2,21,1,39,0,0,0,180,79,1,0,3,0, 
  0,0,1,0,0,0,2,48,1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0,0,1,8,19,33,0,0,0,97,0,0,0,1,0,1,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68, 
  129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,53,211,0,0,15,1,43,206,0,0,17,1,185, 
  204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,53,211,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1, 
  4,17,1,186,85,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,0,2,128,33,152,1,128,26,248,2,128,11,144,1,128,44,136,3,128,29,24,4,128,6,144,1,128,7,144,2,128,1, 
  15,1,53,211,0,0,17,1,185,204,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,53,211,0,0,1,15,1,53,211,0,0,17,1,19,124,0,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,53,211,0, 
  0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,53,211,0,0,1,19,11,0,0,0,38,0,0,0,2,0,1,2,21,0,82,1,0,0,255,255,255,255,16,0,0,0,4,0,0,0,16,8,6,128,33,32,3,128,2, 
  136,10,128,3,248,9,128,20,120,5,128,5,104,9,129,6,216,8,128,23,88,4,128,24,240,3,128,21,232,4,128,10,72,8,128,11,184,7,128,12,40,71,128,28,136,131,128,14,152,6,128,44,144,2,128,19,33,0,0,0, 
  98,0,0,0,1,0,17,1,197,211,0,0,1,15,1,197,211,0,0,17,1,113,214,0,0,1,15,1,197,211,0,0,17,1,75,213,0,0,1,15,1,197,211,0,0,17,1,24,213,0,0,1,19,3,0,0,0,16,0,0, 
  0,1,0,17,1,197,211,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,197,211,0,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,197,211,0,0,1,19,2,0,0,0,10,0,0,0,1,0,17,1,197, 
  211,0,0,1,19,2,0,0,0,9,0,0,0,1,0,17,1,197,211,0,0,1,19,2,0,0,0,8,0,0,0,1,0,17,1,197,211,0,0,1,19,2,0,0,0,7,0,0,0,1,0,17,1,197,211,0,0,1,19,2, 
  0,0,0,6,0,0,0,1,0,17,1,197,211,0,0,1,19,2,0,0,0,5,0,0,0,1,0,17,1,197,211,0,0,1,19,2,0,0,0,4,0,0,0,1,0,17,1,197,211,0,0,1,19,2,0,0,0,3,0,0, 
  0,1,0,17,1,197,211,0,0,1,1,2,21,1,39,0,0,0,180,79,1,0,3,0,0,0,1,0,0,0,2,48,1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0,0,1,8,19,33,0,0,0,97,0,0,0, 
  1,0,1,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33, 
  0,0,0,98,0,0,0,1,0,15,1,225,213,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,225,213,0,0,15,1,43, 
  206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,221,133,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,88,3,128,33,144,2,128,26, 
  232,3,128,11,136,2,128,44,248,1,128,29,248,2,128,6,136,2,128,7,144,1,128,15,1,225,213,0,0,17,1,58,138,0,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,225,213,0,0,1,1,15,1,225,213,0, 
  0,17,1,185,204,0,0,1,19,11,0,0,0,38,0,0,0,2,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,225,213,0,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,225,213,0,0,1,2,21,1,235, 
  0,0,0,163,80,1,0,9,0,0,0,3,0,0,0,40,240,2,128,1,208,196,129,2,208,4,128,35,200,5,128,36,80,2,128,37,184,6,128,38,216,4,128,39,224,3,128,41,176,1,128,4,15,1,104,215,0,0,15,1, 
  95,68,0,0,17,1,221,8,0,0,1,4,15,1,104,215,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,104,215,0,0, 
  1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,104,215,0,0,1,8,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,104,215,0, 
  0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,104,215,0,0,1,4,15,1,104,215,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,19,12,0,0,0,40,0,0, 
  0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,104,215,0,0,1,15,1,104,215,0,0,17,1, 
  167,215,0,0,1,1,2,21,1,193,0,0,0,163,80,1,0,9,0,0,0,3,0,0,0,40,224,2,128,1,64,197,129,2,64,5,128,35,32,2,128,36,208,4,128,37,176,1,128,38,16,4,128,39,72,5,128,41,160,3, 
  128,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0, 
  2,0,1,4,15,1,197,16,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,145,44,0,0,1,8,4,19,31,0, 
  0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,19,19,0,0,0,57,0,0,0,2,0,1,21,1,39,0,0,0,58,45,1,0,3,0,0,0,1, 
  0,0,0,2,48,1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0,0,1,8,19,33,0,0,0,97,0,0,0,1,0,1,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80, 
  129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,74,217,0,0,15,1,43,206,0,0,17,1,185,204,0,0, 
  1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,74,217,0,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1, 
  221,133,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,128,3,128,33,24,3,128,26,144,1,128,11,32,2,128,44,40,2,128,29,184,2,128,6,32,2,128,7,16,4,128,19,7,0,0, 
  0,24,0,0,0,1,0,17,1,74,217,0,0,1,1,19,33,0,0,0,98,0,0,0,1,0,17,1,74,217,0,0,1,19,11,0,0,0,38,0,0,0,2,0,1,15,1,74,217,0,0,17,1,185,204,0,0,1,19,33, 
  0,0,0,97,0,0,0,1,0,17,1,74,217,0,0,1,15,1,74,217,0,0,17,1,58,138,0,0,1,2,21,1,235,0,0,0,134,81,1,0,9,0,0,0,3,0,0,0,40,112,5,128,1,96,198,129,2,96,6,128, 
  35,128,4,128,36,224,3,128,37,80,2,128,38,240,2,128,39,104,6,128,41,176,1,128,4,15,1,209,218,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,15,1,209,218,0,0,15,1,95,68,0,0,17,1,251, 
  60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,209,218,0,0,1,4,15,1,209,218,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0, 
  87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,209,218,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,209,218,0,0,1,8,4,19,31,0,0, 
  0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,209,218,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13, 
  128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,209,218,0,0,1,15,1,209,218,0,0,17,1,16,219,0,0,1,1,2,21,1,193,0,0,0,134,81,1,0,9,0,0,0,3,0,0,0,40,240, 
  3,128,1,176,196,129,2,176,4,128,35,176,1,128,36,184,4,128,37,152,5,128,38,112,2,128,39,48,3,128,41,40,5,128,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19, 
  31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13, 
  0,0,0,42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,145,44,0,0,1,4,15,1,197,16,0,0,17,1,221,8,0,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,19,12,0,0,0,39,0, 
  0,0,2,0,14,1,19,19,0,0,0,56,0,0,0,1,0,1,21,1,44,0,0,0,83,82,1,0,3,0,0,0,1,0,0,0,2,240,0,128,1,240,64,128,13,248,0,128,8,4,19,10,0,0,0,35,0,0,0,2, 
  0,1,2,19,2,0,0,0,6,0,0,0,1,0,1,21,1,147,3,0,0,136,82,1,0,23,0,0,0,4,0,0,0,48,80,7,128,1,112,67,129,2,112,195,132,3,80,91,131,4,8,26,128,5,192,24,128,17,72,78, 
  131,23,120,12,128,8,240,87,130,9,32,87,130,10,216,21,128,11,144,20,128,12,72,19,128,13,168,17,128,14,96,16,128,15,24,15,128,19,120,141,129,24,120,11,128,25,248,9,128,33,208,72,128,49,8,6,128,50,192,4, 
  128,51,120,3,128,8,4,19,45,0,0,0,132,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,15,0,0,0,44,0,0,0,1,0,17,1,164,254,0,0,1,4,19,45,0,0,0,131,0,0,0,1,0,19, 
  34,0,0,0,101,0,0,0,1,0,19,15,0,0,0,44,0,0,0,1,0,17,1,164,254,0,0,1,4,19,45,0,0,0,130,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,15,0,0,0,44,0,0,0, 
  1,0,17,1,164,254,0,0,1,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,164,254,0,0,15,1,140,254,0,0,15,1,152,254,0,0,17,1,31,33,1,0,1,4,19,28, 
  0,0,0,82,0,0,0,1,0,15,1,164,254,0,0,15,1,140,254,0,0,15,1,128,254,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,164, 
  254,0,0,15,1,140,254,0,0,15,1,152,254,0,0,17,1,230,7,0,0,1,4,15,1,164,254,0,0,15,1,140,254,0,0,15,1,116,254,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,164,254,0, 
  0,15,1,140,254,0,0,15,1,116,254,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,164,254,0,0,15,1,140,254,0,0,15,1,104,254,0,0,17,1,49,244,0,0,1,4,15,1,164,254,0,0,15, 
  1,140,254,0,0,15,1,104,254,0,0,17,1,250,233,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,15,0,0,0,44,0,0,0,1,0,17,1,164,254,0,0,1,4, 
  19,10,0,0,0,36,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,15,0,0,0,44,0,0,0,1,0,17,1,164,254,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0, 
  0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,15,0,0,0,44,0,0,0,1,0,17,1,164,254,0,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,15,0,0,0, 
  44,0,0,0,1,0,17,1,164,254,0,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,15,0,0,0,44,0,0,0,1,0,17,1,164,254,0,0,1,4,19,10,0,0,0, 
  32,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,15,0,0,0,44,0,0,0,1,0,17,1,164,254,0,0,1,4,15,1,164,254,0,0,15,1,140,254,0,0,15,1,238,233,0,0,17,1,234,219,0,0, 
  1,4,15,1,164,254,0,0,15,1,140,254,0,0,15,1,104,254,0,0,17,1,183,223,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,15,0,0,0,44,0,0,0,1, 
  0,17,1,164,254,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,15,0,0,0,44,0,0,0,1,0,17,1,164,254,0,0,1,4,19,3,0,0,0,11,0,0,0,1, 
  0,19,34,0,0,0,99,0,0,0,1,0,19,15,0,0,0,44,0,0,0,1,0,17,1,164,254,0,0,1,2,21,1,164,3,0,0,217,83,1,0,24,0,0,0,4,0,0,0,48,112,7,128,1,144,195,131,2,144,3, 
  133,3,216,155,131,4,144,26,128,5,72,25,128,6,224,24,128,23,152,12,128,8,16,152,130,9,64,151,130,10,248,21,128,11,176,20,128,12,104,19,128,13,200,17,128,14,128,16,128,15,56,15,128,17,104,14,129,19,152,141, 
  129,24,152,11,128,25,24,10,128,33,240,72,128,49,40,6,128,50,224,4,128,51,152,3,128,8,4,19,45,0,0,0,132,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,49,0,0,0,143,0,0,0,1,0, 
  17,1,104,227,0,0,1,4,19,45,0,0,0,131,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227,0,0,1,4,19,45,0,0,0,130,0,0,0,1,0, 
  19,34,0,0,0,101,0,0,0,1,0,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227,0,0,1,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,104,227,0,0,15,1, 
  92,227,0,0,15,1,152,254,0,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,104,227,0,0,15,1,92,227,0,0,15,1,128,254,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0, 
  63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,104,227,0,0,15,1,92,227,0,0,15,1,152,254,0,0,17,1,230,7,0,0,1,4,15,1,104,227,0,0,15,1,92,227,0,0,15,1,116,254,0, 
  0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,104,227,0,0,15,1,92,227,0,0,15,1,116,254,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,104,227,0,0,15,1,92,227,0,0,15, 
  1,104,254,0,0,17,1,49,244,0,0,1,4,15,1,104,227,0,0,15,1,92,227,0,0,15,1,104,254,0,0,17,1,250,233,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1, 
  0,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227,0,0,1, 
  4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227,0,0,1,4,19,10,0,0,0,34,0, 
  0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227,0,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,49,0,0, 
  0,143,0,0,0,1,0,17,1,104,227,0,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227,0,0,1,4,15,1,104,227, 
  0,0,15,1,92,227,0,0,15,1,238,233,0,0,17,1,234,219,0,0,1,4,15,1,104,227,0,0,15,1,92,227,0,0,15,1,104,254,0,0,17,1,183,223,0,0,1,4,19,46,0,0,0,138,0,0,0,2,0,1, 
  4,19,3,0,0,0,13,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,34,0,0,0,99,0, 
  0,0,1,0,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227, 
  0,0,1,2,19,49,0,0,0,143,0,0,0,1,0,1,21,0,77,1,0,0,255,255,255,255,15,0,0,0,3,0,0,0,24,232,5,128,33,24,69,131,10,72,73,130,3,216,73,129,12,40,72,129,21,8,7,130,46,216, 
  2,128,23,120,6,128,11,184,8,128,20,152,71,128,28,128,133,128,34,136,4,128,44,248,3,128,45,104,3,128,49,112,2,128,15,1,104,227,0,0,17,1,186,230,0,0,1,19,34,0,0,0,104,0,0,0,1,0,17,1, 
  104,227,0,0,1,19,34,0,0,0,101,0,0,0,1,0,17,1,104,227,0,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,104,227,0,0,1,19,49,0,0,0,143,0,0,0,1,0,17,1,104,227,0,0,1,15, 
  1,104,227,0,0,17,1,182,228,0,0,1,15,1,104,227,0,0,17,1,22,9,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,104,227,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,104,227,0,0, 
  1,19,3,0,0,0,15,0,0,0,1,0,17,1,104,227,0,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,104,227,0,0,1,19,34,0,0,0,103,0,0,0,1,0,17,1,104,227,0,0,1,19,34,0,0,0, 
  102,0,0,0,1,0,17,1,104,227,0,0,1,19,34,0,0,0,100,0,0,0,1,0,17,1,104,227,0,0,1,19,34,0,0,0,99,0,0,0,1,0,17,1,104,227,0,0,1,2,21,1,235,0,0,0,188,84,1,0, 
  9,0,0,0,3,0,0,0,40,112,5,128,1,96,198,129,2,96,6,128,35,104,6,128,36,64,3,128,37,176,1,128,38,80,2,128,39,224,3,128,41,208,4,128,4,15,1,173,229,0,0,15,1,95,68,0,0,17,1,251, 
  60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,173,229,0,0,1,4,15,1,173,229,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0, 
  92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,173,229,0,0,1,4,15,1,173,229,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0, 
  0,41,0,0,0,1,0,17,1,173,229,0,0,1,8,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,173,229,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62, 
  0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,173,229,0,0,1,15,1,173,229,0,0,17,1,236,229,0,0,1,1,2, 
  21,1,193,0,0,0,188,84,1,0,9,0,0,0,3,0,0,0,40,16,4,128,1,144,197,129,2,144,5,128,35,208,4,128,36,152,5,128,37,112,2,128,38,224,2,128,39,176,1,128,41,160,3,128,4,19,31,0,0,0, 
  92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197, 
  16,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4, 
  15,1,197,16,0,0,17,1,145,44,0,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,21,1,39,3,0,0,217,83,1,0,24,0,0,0,4,0,0,0,48,224,6,128,1,144,195,131,2,144,3,133,3,32,152,131, 
  4,8,23,128,5,240,21,128,6,128,21,128,23,72,11,128,8,224,148,130,9,64,148,130,10,40,19,128,11,16,18,128,12,248,16,128,13,136,15,128,14,112,14,128,15,88,13,128,17,184,12,129,19,24,140,129,24,120,10,128, 
  25,40,9,128,33,48,72,128,49,200,5,128,50,176,4,128,51,152,3,128,8,4,19,45,0,0,0,132,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,49,0,0,0,144,0,0,0,2,0,1,4,19,45,0, 
  0,0,131,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,49,0,0,0,144,0,0,0,2,0,1,4,19,45,0,0,0,130,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,49,0,0,0,144, 
  0,0,0,2,0,1,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,226,233,0,0,15,1,152,254,0,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0, 
  15,1,226,233,0,0,15,1,128,254,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,226,233,0,0,15,1,152,254,0,0,17,1,230,7,0,0, 
  1,4,15,1,226,233,0,0,15,1,116,254,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,226,233,0,0,15,1,116,254,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,226,233,0,0, 
  15,1,104,254,0,0,17,1,49,244,0,0,1,4,15,1,226,233,0,0,15,1,104,254,0,0,17,1,250,233,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,49,0,0, 
  0,144,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,49,0,0,0,144,0,0,0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0, 
  14,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,49,0,0,0,144,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,49,0,0,0,144,0,0, 
  0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,49,0,0,0,144,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,34,0,0,0,100,0,0,0, 
  1,0,19,49,0,0,0,144,0,0,0,2,0,1,4,15,1,226,233,0,0,15,1,238,233,0,0,17,1,234,219,0,0,1,4,15,1,226,233,0,0,15,1,104,254,0,0,17,1,183,223,0,0,1,4,19,46,0,0,0, 
  137,0,0,0,3,0,14,1,4,19,3,0,0,0,13,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,49,0,0,0,144,0,0,0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,34,0,0,0, 
  99,0,0,0,1,0,19,49,0,0,0,144,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,49,0,0,0,144,0,0,0,2,0,1,2,19,49,0,0,0,144, 
  0,0,0,2,0,1,19,34,0,0,0,100,0,0,0,1,0,1,21,1,164,3,0,0,160,85,1,0,24,0,0,0,4,0,0,0,48,112,7,128,1,144,67,129,2,144,131,131,3,216,155,131,4,144,26,128,5,72,25,128, 
  17,208,142,131,23,152,12,128,8,120,152,130,9,168,151,130,10,96,22,128,11,24,21,128,12,208,19,128,13,48,18,128,14,232,16,128,15,160,15,128,18,104,142,129,19,152,141,129,24,152,11,128,25,24,10,128,33,240,72,128, 
  49,40,6,128,50,224,4,128,51,152,3,128,8,4,19,45,0,0,0,132,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,48,0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1,4,19,45,0,0,0, 
  131,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,48,0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1,4,19,45,0,0,0,130,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,48, 
  0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,171,237,0,0,15,1,159,237,0,0,15,1,152,254,0,0,17,1,31, 
  33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,171,237,0,0,15,1,159,237,0,0,15,1,128,254,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0, 
  0,0,1,0,15,1,171,237,0,0,15,1,159,237,0,0,15,1,152,254,0,0,17,1,230,7,0,0,1,4,15,1,171,237,0,0,15,1,159,237,0,0,15,1,116,254,0,0,15,1,186,31,1,0,17,1,179,28,1,0, 
  1,4,15,1,171,237,0,0,15,1,159,237,0,0,15,1,116,254,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,171,237,0,0,15,1,159,237,0,0,15,1,104,254,0,0,17,1,49,244,0,0,1,4, 
  19,46,0,0,0,136,0,0,0,2,0,1,4,15,1,171,237,0,0,15,1,159,237,0,0,15,1,104,254,0,0,17,1,250,233,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1, 
  0,19,48,0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,48,0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1, 
  4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,48,0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1,4,19,10,0,0,0,34,0, 
  0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,48,0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,48,0,0, 
  0,141,0,0,0,1,0,17,1,171,237,0,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,48,0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1,4,15,1,171,237, 
  0,0,15,1,159,237,0,0,15,1,238,233,0,0,17,1,234,219,0,0,1,4,15,1,171,237,0,0,15,1,159,237,0,0,15,1,104,254,0,0,17,1,183,223,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19, 
  34,0,0,0,99,0,0,0,1,0,19,48,0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,48,0,0,0,141,0,0,0, 
  1,0,17,1,171,237,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,48,0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1,2,19,48,0,0,0,141,0,0,0, 
  1,0,1,21,0,77,1,0,0,255,255,255,255,15,0,0,0,3,0,0,0,24,232,133,131,33,24,5,128,10,72,73,130,3,216,73,129,12,40,72,129,21,8,7,130,46,216,2,128,23,120,6,128,11,184,8,128,20,152,71, 
  128,28,128,133,128,34,136,4,128,44,248,3,128,45,104,3,128,48,112,2,128,15,1,171,237,0,0,17,1,253,240,0,0,1,19,34,0,0,0,104,0,0,0,1,0,17,1,171,237,0,0,1,19,34,0,0,0,101,0,0, 
  0,1,0,17,1,171,237,0,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,171,237,0,0,1,19,48,0,0,0,141,0,0,0,1,0,17,1,171,237,0,0,1,15,1,171,237,0,0,17,1,249,238,0,0,1,15, 
  1,171,237,0,0,17,1,22,9,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,171,237,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,171,237,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17, 
  1,171,237,0,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,171,237,0,0,1,19,34,0,0,0,103,0,0,0,1,0,17,1,171,237,0,0,1,19,34,0,0,0,102,0,0,0,1,0,17,1,171,237,0,0,1, 
  19,34,0,0,0,100,0,0,0,1,0,17,1,171,237,0,0,1,19,34,0,0,0,99,0,0,0,1,0,17,1,171,237,0,0,1,2,21,1,235,0,0,0,131,86,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1, 
  80,199,129,2,80,7,128,35,208,4,128,36,192,5,128,37,160,2,128,38,224,3,128,39,96,6,128,41,64,3,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,240,239,0,0, 
  1,4,15,1,240,239,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,240,239,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0, 
  0,0,1,0,17,1,240,239,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,240,239,0,0,1,4,15,1,240,239,0,0,15,1,95,68,0,0,17,1,145,44,0,0, 
  1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,240,239,0,0,1,8,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0, 
  0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,240,239,0,0,1,15,1,240,239,0,0,17,1,47,240,0,0,1,1,2,21,1,193,0,0,0,131,86,1,0,9,0,0, 
  0,3,0,0,0,40,16,4,128,1,144,197,129,2,144,5,128,35,208,4,128,36,152,5,128,37,112,2,128,38,224,2,128,39,176,1,128,41,160,3,128,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0, 
  0,0,2,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,221,8,0,0,1,4,19,31, 
  0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,145,44,0,0,1, 
  19,12,0,0,0,39,0,0,0,2,0,14,1,21,1,39,3,0,0,160,85,1,0,24,0,0,0,4,0,0,0,48,224,6,128,1,144,67,129,2,144,131,131,3,32,152,131,4,8,23,128,5,240,21,128,17,40,141,131,23, 
  72,11,128,8,80,149,130,9,176,148,130,10,152,19,128,11,128,18,128,12,104,17,128,13,248,15,128,14,224,14,128,15,200,13,128,18,184,140,129,19,24,140,129,24,120,10,128,25,40,9,128,33,48,72,128,49,200,5,128,50, 
  176,4,128,51,152,3,128,8,4,19,45,0,0,0,132,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,48,0,0,0,142,0,0,0,2,0,1,4,19,45,0,0,0,131,0,0,0,1,0,19,34,0,0,0, 
  101,0,0,0,1,0,19,48,0,0,0,142,0,0,0,2,0,1,4,19,45,0,0,0,130,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,48,0,0,0,142,0,0,0,2,0,1,4,19,44,0,0,0,129, 
  0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,37,244,0,0,15,1,152,254,0,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,37,244,0,0,15,1,128,254,0,0,17, 
  1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,37,244,0,0,15,1,152,254,0,0,17,1,230,7,0,0,1,4,15,1,37,244,0,0,15,1,116,254,0, 
  0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,37,244,0,0,15,1,116,254,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,37,244,0,0,15,1,104,254,0,0,17,1,49,244,0,0,1, 
  4,19,46,0,0,0,135,0,0,0,3,0,14,1,4,15,1,37,244,0,0,15,1,104,254,0,0,17,1,250,233,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,48,0, 
  0,0,142,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,48,0,0,0,142,0,0,0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0, 
  0,14,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,48,0,0,0,142,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,48,0,0,0,142,0, 
  0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,48,0,0,0,142,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,34,0,0,0,100,0,0, 
  0,1,0,19,48,0,0,0,142,0,0,0,2,0,1,4,15,1,37,244,0,0,15,1,238,233,0,0,17,1,234,219,0,0,1,4,15,1,37,244,0,0,15,1,104,254,0,0,17,1,183,223,0,0,1,4,19,3,0,0, 
  0,13,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,48,0,0,0,142,0,0,0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,48,0,0,0,142,0, 
  0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,48,0,0,0,142,0,0,0,2,0,1,2,19,48,0,0,0,142,0,0,0,2,0,1,21,1,164,3,0,0,103, 
  87,1,0,24,0,0,0,4,0,0,0,48,112,7,128,1,144,67,129,2,144,3,133,3,216,91,131,4,144,90,131,5,72,25,128,17,208,142,131,23,152,12,128,8,120,152,130,9,168,151,130,10,96,22,128,11,24,21,128,12, 
  208,19,128,13,48,18,128,14,232,16,128,15,160,15,128,19,0,206,129,20,152,13,128,24,152,11,128,25,24,10,128,33,240,72,128,49,40,6,128,50,224,4,128,51,152,3,128,8,4,19,45,0,0,0,132,0,0,0,1,0, 
  19,34,0,0,0,101,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247,0,0,1,4,19,45,0,0,0,131,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,47,0,0,0,139,0,0, 
  0,1,0,17,1,226,247,0,0,1,4,19,45,0,0,0,130,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247,0,0,1,4,19,44,0,0,0,129,0,0, 
  0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,226,247,0,0,15,1,214,247,0,0,15,1,152,254,0,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,226,247,0,0,15,1,214, 
  247,0,0,15,1,128,254,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,226,247,0,0,15,1,214,247,0,0,15,1,152,254,0,0,17,1,230, 
  7,0,0,1,4,15,1,226,247,0,0,15,1,214,247,0,0,15,1,116,254,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,226,247,0,0,15,1,214,247,0,0,15,1,116,254,0,0,15,1,167,28,1, 
  0,17,1,12,28,1,0,1,4,19,46,0,0,0,134,0,0,0,2,0,1,4,15,1,226,247,0,0,15,1,214,247,0,0,15,1,104,254,0,0,17,1,49,244,0,0,1,4,15,1,226,247,0,0,15,1,214,247,0,0, 
  15,1,104,254,0,0,17,1,250,233,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247,0,0,1,4,19,10,0,0,0, 
  36,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,34, 
  0,0,0,99,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247,0,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1, 
  0,17,1,226,247,0,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247,0,0,1,4,19,10,0,0,0,32,0,0,0,1, 
  0,19,34,0,0,0,100,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247,0,0,1,4,15,1,226,247,0,0,15,1,214,247,0,0,15,1,238,233,0,0,17,1,234,219,0,0,1,4,15,1,226, 
  247,0,0,15,1,214,247,0,0,15,1,104,254,0,0,17,1,183,223,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247, 
  0,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247,0,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,34,0,0, 
  0,99,0,0,0,1,0,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247,0,0,1,2,19,47,0,0,0,139,0,0,0,1,0,1,21,0,77,1,0,0,255,255,255,255,15,0,0,0,3,0,0,0,24,232,5,128, 
  33,24,5,128,10,72,73,130,3,216,73,129,12,40,72,129,21,8,7,130,46,216,2,128,23,120,198,129,11,184,8,128,20,152,71,128,28,128,133,128,34,136,4,128,44,248,3,128,45,104,3,128,47,112,2,128,15,1,226,247, 
  0,0,17,1,52,251,0,0,1,19,34,0,0,0,104,0,0,0,1,0,17,1,226,247,0,0,1,19,34,0,0,0,101,0,0,0,1,0,17,1,226,247,0,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,226,247, 
  0,0,1,19,47,0,0,0,139,0,0,0,1,0,17,1,226,247,0,0,1,15,1,226,247,0,0,17,1,48,249,0,0,1,15,1,226,247,0,0,17,1,22,9,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1, 
  226,247,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,226,247,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,226,247,0,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,226,247,0,0,1,19, 
  34,0,0,0,103,0,0,0,1,0,17,1,226,247,0,0,1,19,34,0,0,0,102,0,0,0,1,0,17,1,226,247,0,0,1,19,34,0,0,0,100,0,0,0,1,0,17,1,226,247,0,0,1,19,34,0,0,0,99,0, 
  0,0,1,0,17,1,226,247,0,0,1,2,21,1,235,0,0,0,74,88,1,0,9,0,0,0,3,0,0,0,40,40,5,128,1,32,197,129,2,32,5,128,35,64,3,128,36,24,6,128,37,176,1,128,38,48,4,128,39,80, 
  2,128,41,184,6,128,4,15,1,39,250,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,39,250,0,0,1,4,19,31,0, 
  0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,39,250,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,39,250,0,0,1,8,4,19,31, 
  0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,39,250,0,0,1,4,15,1,39,250,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,15,1,39,250,0,0,15,1,95,68,0,0, 
  17,1,221,8,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0, 
  17,1,39,250,0,0,1,15,1,39,250,0,0,17,1,102,250,0,0,1,1,2,21,1,193,0,0,0,74,88,1,0,9,0,0,0,3,0,0,0,40,16,4,128,1,144,197,129,2,144,5,128,35,208,4,128,36,152,5,128, 
  37,112,2,128,38,224,2,128,39,176,1,128,41,160,3,128,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0, 
  91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0, 
  0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,145,44,0,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,21,1,39,3,0,0,103,87,1,0,24,0, 
  0,0,4,0,0,0,48,224,6,128,1,144,67,129,2,144,3,133,3,32,88,131,4,8,87,131,5,240,21,128,17,40,141,131,23,72,11,128,8,80,149,130,9,176,148,130,10,152,19,128,11,128,18,128,12,104,17,128,13,248, 
  15,128,14,224,14,128,15,200,13,128,19,136,204,129,20,24,12,128,24,120,10,128,25,40,9,128,33,48,72,128,49,200,5,128,50,176,4,128,51,152,3,128,8,4,19,45,0,0,0,132,0,0,0,1,0,19,34,0,0,0, 
  101,0,0,0,1,0,19,47,0,0,0,140,0,0,0,2,0,1,4,19,45,0,0,0,131,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,47,0,0,0,140,0,0,0,2,0,1,4,19,45,0,0,0,130, 
  0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,47,0,0,0,140,0,0,0,2,0,1,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,92,254,0,0,15,1,152, 
  254,0,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,92,254,0,0,15,1,128,254,0,0,17,1,22,9,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0, 
  0,0,1,0,15,1,92,254,0,0,15,1,152,254,0,0,17,1,230,7,0,0,1,4,15,1,92,254,0,0,15,1,116,254,0,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,92,254,0,0,15,1,116,254, 
  0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,19,46,0,0,0,133,0,0,0,3,0,14,1,4,15,1,92,254,0,0,15,1,104,254,0,0,17,1,49,244,0,0,1,4,15,1,92,254,0,0,15,1,104,254, 
  0,0,17,1,250,233,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,47,0,0,0,140,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,34,0,0, 
  0,100,0,0,0,1,0,19,47,0,0,0,140,0,0,0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,47,0,0,0,140,0, 
  0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,47,0,0,0,140,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,34,0,0,0,100,0,0, 
  0,1,0,19,47,0,0,0,140,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,47,0,0,0,140,0,0,0,2,0,1,4,15,1,92,254,0,0,15,1,238, 
  233,0,0,17,1,234,219,0,0,1,4,15,1,92,254,0,0,15,1,104,254,0,0,17,1,183,223,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,47,0,0,0,140,0, 
  0,0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,47,0,0,0,140,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,34,0,0,0,99,0,0, 
  0,1,0,19,47,0,0,0,140,0,0,0,2,0,1,2,19,47,0,0,0,140,0,0,0,2,0,1,19,34,0,0,0,104,0,0,0,1,0,1,19,34,0,0,0,99,0,0,0,1,0,1,19,34,0,0,0,102,0,0, 
  0,1,0,1,19,15,0,0,0,44,0,0,0,1,0,1,19,34,0,0,0,103,0,0,0,1,0,1,21,0,82,1,0,0,255,255,255,255,16,0,0,0,4,0,0,0,28,56,133,129,33,208,4,128,34,64,4,128,3,0, 
  10,128,20,80,7,128,21,192,6,128,44,176,3,128,23,48,6,128,24,160,5,128,46,144,2,128,10,112,9,128,11,224,8,128,12,80,8,125,45,32,3,128,14,72,200,126,15,224,7,128,19,34,0,0,0,104,0,0,0,1, 
  0,17,1,164,254,0,0,1,19,34,0,0,0,101,0,0,0,1,0,17,1,164,254,0,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,164,254,0,0,1,19,15,0,0,0,44,0,0,0,1,0,17,1,164,254,0, 
  0,1,15,1,164,254,0,0,17,1,43,3,1,0,1,15,1,164,254,0,0,17,1,22,9,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,164,254,0,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,164, 
  254,0,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,164,254,0,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,164,254,0,0,1,15,1,164,254,0,0,17,1,247,255,0,0,1,1,19,34,0,0,0,103, 
  0,0,0,1,0,17,1,164,254,0,0,1,19,34,0,0,0,102,0,0,0,1,0,17,1,164,254,0,0,1,19,34,0,0,0,100,0,0,0,1,0,17,1,164,254,0,0,1,19,34,0,0,0,99,0,0,0,1,0,17, 
  1,164,254,0,0,1,2,21,1,39,3,0,0,217,83,1,0,24,0,0,0,4,0,0,0,48,224,6,128,1,144,195,131,2,144,3,133,3,32,152,131,4,8,23,128,5,240,21,128,6,128,21,128,23,72,11,128,8,224,148, 
  130,9,64,148,130,10,40,19,128,11,16,18,128,12,248,16,128,13,136,15,128,14,112,14,128,15,88,13,128,17,184,12,129,19,24,140,129,24,120,10,128,25,40,9,128,33,48,72,128,49,200,5,128,50,176,4,128,51,152,3, 
  128,8,4,19,45,0,0,0,132,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,15,0,0,0,45,0,0,0,2,0,1,4,19,45,0,0,0,131,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0, 
  19,15,0,0,0,45,0,0,0,2,0,1,4,19,45,0,0,0,130,0,0,0,1,0,19,34,0,0,0,101,0,0,0,1,0,19,15,0,0,0,45,0,0,0,2,0,1,4,19,44,0,0,0,129,0,0,0,1,0,19, 
  33,0,0,0,98,0,0,0,1,0,15,1,31,3,1,0,15,1,152,254,0,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,31,3,1,0,15,1,128,254,0,0,17,1,22,9,0,0,1, 
  4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,31,3,1,0,15,1,152,254,0,0,17,1,230,7,0,0,1,4,15,1,31,3,1,0,15,1,116,254,0,0,15,1,186,31,1, 
  0,17,1,179,28,1,0,1,4,15,1,31,3,1,0,15,1,116,254,0,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,31,3,1,0,15,1,104,254,0,0,17,1,49,244,0,0,1,4,15,1,31,3,1, 
  0,15,1,104,254,0,0,17,1,250,233,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,15,0,0,0,45,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1, 
  0,19,34,0,0,0,100,0,0,0,1,0,19,15,0,0,0,45,0,0,0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,15, 
  0,0,0,45,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,15,0,0,0,45,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,34,0, 
  0,0,100,0,0,0,1,0,19,15,0,0,0,45,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,34,0,0,0,100,0,0,0,1,0,19,15,0,0,0,45,0,0,0,2,0,1,4,15,1,31,3, 
  1,0,15,1,238,233,0,0,17,1,234,219,0,0,1,4,15,1,31,3,1,0,15,1,104,254,0,0,17,1,183,223,0,0,1,4,19,14,0,0,0,43,0,0,0,3,0,14,1,4,19,3,0,0,0,13,0,0,0,1, 
  0,19,34,0,0,0,99,0,0,0,1,0,19,15,0,0,0,45,0,0,0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,15,0,0,0,45,0,0,0,2,0,1,4, 
  19,3,0,0,0,11,0,0,0,1,0,19,34,0,0,0,99,0,0,0,1,0,19,15,0,0,0,45,0,0,0,2,0,1,2,19,15,0,0,0,45,0,0,0,2,0,1,21,1,235,0,0,0,188,84,1,0,9,0,0, 
  0,3,0,0,0,40,104,6,128,1,192,197,129,2,192,5,128,35,224,3,128,36,200,5,128,37,64,3,128,38,80,2,128,39,208,4,128,41,176,1,128,4,15,1,173,229,0,0,15,1,95,68,0,0,17,1,221,8,0,0, 
  1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,173,229,0,0,1,4,15,1,173,229,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,19,31,0,0,0,87,0,0, 
  0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,173,229,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,173,229,0,0,1,8,4,15,1,173,229,0,0,15, 
  1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,173,229,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,19,2,0,0,0,9, 
  0,0,0,1,0,1,21,1,24,3,0,0,46,89,1,0,22,0,0,0,4,0,0,0,16,112,74,133,1,80,67,129,2,80,131,131,3,120,151,131,4,48,22,128,5,232,20,128,17,160,137,131,23,104,7,128,8,24,148,130, 
  9,72,147,130,10,0,18,128,11,184,16,128,12,112,15,128,13,208,13,128,14,136,12,128,15,64,11,128,18,56,9,128,19,104,8,128,24,104,6,128,25,160,5,128,33,216,4,128,48,88,3,128,8,4,19,44,0,0,0,129, 
  0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,160,8,1,0,15,1,11,16,1,0,15,1,139,41,1,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,160,8,1,0,17, 
  1,122,7,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,160,8,1,0,17,1,71,7,1,0,1,4,15,1,160,8,1,0,15,1,11,16,1,0,15,1,174,31,1,0,15,1,186,31,1,0,17,1,179,28, 
  1,0,1,4,15,1,160,8,1,0,15,1,11,16,1,0,15,1,174,31,1,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,160,8,1,0,15,1,11,16,1,0,15,1,0,28,1,0,17,1,23,16,1,0, 
  1,4,19,16,0,0,0,49,0,0,0,2,0,1,4,15,1,160,8,1,0,15,1,11,16,1,0,15,1,0,28,1,0,17,1,46,4,1,0,1,4,15,1,160,8,1,0,15,1,11,16,1,0,15,1,34,4,1,0,17, 
  1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,18,0,0,0,54,0,0,0,1,0,17,1,160,8,1,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19, 
  2,0,0,0,6,0,0,0,1,0,19,18,0,0,0,54,0,0,0,1,0,17,1,160,8,1,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,2,0,0,0,3,0,0,0, 
  1,0,19,18,0,0,0,54,0,0,0,1,0,17,1,160,8,1,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,18,0,0,0,54,0,0,0,1,0,17,1,160,8,1,0, 
  1,4,19,10,0,0,0,33,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,18,0,0,0,54,0,0,0,1,0,17,1,160,8,1,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,2,0,0,0,6, 
  0,0,0,1,0,19,18,0,0,0,54,0,0,0,1,0,17,1,160,8,1,0,1,4,15,1,160,8,1,0,15,1,11,16,1,0,15,1,23,220,0,0,17,1,234,219,0,0,1,4,15,1,160,8,1,0,15,1,11,16, 
  1,0,15,1,0,28,1,0,17,1,29,3,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,18,0,0,0,54,0,0,0,1,0,17,1,160,8,1,0,1,4,19,3,0, 
  0,0,12,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,18,0,0,0,54,0,0,0,1,0,17,1,160,8,1,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0, 
  19,18,0,0,0,54,0,0,0,1,0,17,1,160,8,1,0,1,2,21,1,39,0,0,0,250,89,1,0,3,0,0,0,1,0,0,0,2,48,1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0,0,1,8,19,33, 
  0,0,0,97,0,0,0,1,0,1,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129, 
  0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,16,8,1,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1, 
  16,8,1,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,186,85,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,88, 
  3,128,33,240,2,128,26,88,2,128,11,232,2,128,44,232,3,128,29,144,1,128,6,232,2,128,7,240,1,128,19,11,0,0,0,38,0,0,0,2,0,1,15,1,16,8,1,0,17,1,19,124,0,0,1,19,7,0,0,0, 
  24,0,0,0,1,0,17,1,16,8,1,0,1,1,15,1,16,8,1,0,17,1,185,204,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,16,8,1,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,16,8, 
  1,0,1,2,21,0,116,1,0,0,255,255,255,255,17,0,0,0,4,0,0,0,16,144,6,128,33,64,3,128,2,16,203,129,3,128,10,128,20,152,5,128,5,240,9,130,6,96,9,128,23,120,4,128,24,16,4,128,18,40, 
  6,128,10,208,8,128,11,64,8,128,12,176,199,128,21,8,5,128,14,32,7,128,28,168,67,128,44,176,2,128,19,33,0,0,0,98,0,0,0,1,0,17,1,160,8,1,0,1,15,1,160,8,1,0,17,1,7,14,1,0, 
  1,15,1,160,8,1,0,17,1,225,12,1,0,1,15,1,160,8,1,0,17,1,71,7,1,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,160,8,1,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,160,8, 
  1,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,160,8,1,0,1,15,1,160,8,1,0,17,1,21,10,1,0,1,19,2,0,0,0,10,0,0,0,1,0,17,1,160,8,1,0,1,19,2,0,0,0,9,0,0, 
  0,1,0,17,1,160,8,1,0,1,19,2,0,0,0,8,0,0,0,1,0,17,1,160,8,1,0,1,19,2,0,0,0,7,0,0,0,1,0,17,1,160,8,1,0,1,19,2,0,0,0,6,0,0,0,1,0,17,1,160, 
  8,1,0,1,19,2,0,0,0,5,0,0,0,1,0,17,1,160,8,1,0,1,19,2,0,0,0,4,0,0,0,1,0,17,1,160,8,1,0,1,19,2,0,0,0,3,0,0,0,1,0,17,1,160,8,1,0,1,19,18, 
  0,0,0,54,0,0,0,1,0,17,1,160,8,1,0,1,2,21,1,191,2,0,0,46,89,1,0,22,0,0,0,4,0,0,0,16,232,73,133,1,80,67,129,2,80,131,131,3,224,148,131,4,200,19,128,5,176,18,128,17, 
  72,137,131,23,104,7,128,8,16,146,130,9,112,145,130,10,88,16,128,11,64,15,128,12,40,14,128,13,184,12,128,14,160,11,128,15,136,10,128,18,216,8,128,19,56,8,128,24,152,6,128,25,160,5,128,33,168,4,128,48, 
  88,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,213,12,1,0,15,1,139,41,1,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1, 
  213,12,1,0,15,1,197,211,0,0,17,1,159,210,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,213,12,1,0,15,1,197,211,0,0,17,1,108,210,0,0,1,4,15,1,213,12,1,0,15,1,174,31,1, 
  0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,213,12,1,0,15,1,174,31,1,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,213,12,1,0,15,1,0,28,1,0,17,1,23,16,1,0,1, 
  4,19,16,0,0,0,48,0,0,0,3,0,14,1,4,15,1,213,12,1,0,15,1,0,28,1,0,17,1,46,4,1,0,1,4,15,1,213,12,1,0,15,1,34,4,1,0,17,1,35,220,0,0,1,4,19,10,0,0,0, 
  37,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,18,0,0,0,55,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,18,0,0,0,55,0,0, 
  0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,18,0,0,0,55,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0, 
  1,0,19,2,0,0,0,6,0,0,0,1,0,19,18,0,0,0,55,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,18,0,0,0,55,0,0,0,2,0,1, 
  4,19,10,0,0,0,32,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,18,0,0,0,55,0,0,0,2,0,1,4,15,1,213,12,1,0,15,1,23,220,0,0,17,1,234,219,0,0,1,4,15,1,213,12, 
  1,0,15,1,0,28,1,0,17,1,29,3,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,18,0,0,0,55,0,0,0,2,0,1,4,19,3,0,0,0,12,0,0,0, 
  1,0,19,2,0,0,0,3,0,0,0,1,0,19,18,0,0,0,55,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,18,0,0,0,55,0,0,0,2,0,1, 
  2,19,18,0,0,0,55,0,0,0,2,0,1,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0, 
  0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,119,13,1,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1, 
  0,15,1,119,13,1,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,221,133,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0, 
  0,24,240,2,128,33,136,2,128,26,144,1,128,11,128,2,128,44,128,3,128,29,32,2,128,6,128,2,128,7,16,4,128,19,7,0,0,0,24,0,0,0,1,0,17,1,119,13,1,0,1,19,11,0,0,0,38,0,0,0, 
  2,0,1,1,15,1,119,13,1,0,17,1,185,204,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,119,13,1,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,119,13,1,0,1,15,1,119,13,1,0,17, 
  1,58,138,0,0,1,2,21,1,235,0,0,0,211,90,1,0,9,0,0,0,3,0,0,0,40,240,2,128,1,96,198,129,2,96,6,128,35,112,5,128,36,80,2,128,37,176,1,128,38,104,6,128,39,128,4,128,41,224,3, 
  128,4,15,1,254,14,1,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,254,14,1,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0, 
  0,0,1,0,17,1,254,14,1,0,1,4,15,1,254,14,1,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,254,14,1,0, 
  1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,254,14,1,0,1,8,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,254,14,1, 
  0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,254,14,1, 
  0,1,15,1,254,14,1,0,17,1,61,15,1,0,1,1,2,21,1,193,0,0,0,211,90,1,0,9,0,0,0,3,0,0,0,40,240,3,128,1,176,196,129,2,176,4,128,35,176,1,128,36,184,4,128,37,152,5,128,38, 
  112,2,128,39,48,3,128,41,40,5,128,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4, 
  19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,145,44,0, 
  0,1,4,15,1,197,16,0,0,17,1,221,8,0,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,19,18,0,0,0,54,0,0,0,1,0,1,21,1,24,3,0, 
  0,160,91,1,0,22,0,0,0,4,0,0,0,16,112,74,133,1,80,67,129,2,80,3,128,3,120,87,131,4,48,86,131,5,232,20,128,17,160,137,131,23,104,7,128,8,24,148,130,9,72,147,130,10,0,18,128,11,184,16, 
  128,12,112,15,128,13,208,13,128,14,136,12,128,15,64,11,128,19,208,8,128,20,104,8,128,24,104,6,128,25,160,5,128,33,216,4,128,48,88,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98, 
  0,0,0,1,0,15,1,137,20,1,0,15,1,244,27,1,0,15,1,139,41,1,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,137,20,1,0,17,1,99,19,1,0,1,4,19,24,0,0, 
  0,63,0,0,0,1,0,15,1,137,20,1,0,17,1,48,19,1,0,1,4,15,1,137,20,1,0,15,1,244,27,1,0,15,1,174,31,1,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,137,20,1,0,15, 
  1,244,27,1,0,15,1,174,31,1,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,19,16,0,0,0,47,0,0,0,2,0,1,4,15,1,137,20,1,0,15,1,244,27,1,0,15,1,0,28,1,0,17,1,23,16, 
  1,0,1,4,15,1,137,20,1,0,15,1,244,27,1,0,15,1,0,28,1,0,17,1,46,4,1,0,1,4,15,1,137,20,1,0,15,1,244,27,1,0,15,1,34,4,1,0,17,1,35,220,0,0,1,4,19,10,0,0, 
  0,37,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,17,0,0,0,52,0,0,0,1,0,17,1,137,20,1,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19, 
  17,0,0,0,52,0,0,0,1,0,17,1,137,20,1,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,17,0,0,0,52,0,0,0, 
  1,0,17,1,137,20,1,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,17,0,0,0,52,0,0,0,1,0,17,1,137,20,1,0,1,4,19,10,0,0,0,33,0,0,0, 
  1,0,19,2,0,0,0,6,0,0,0,1,0,19,17,0,0,0,52,0,0,0,1,0,17,1,137,20,1,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,17,0,0,0,52, 
  0,0,0,1,0,17,1,137,20,1,0,1,4,15,1,137,20,1,0,15,1,244,27,1,0,15,1,23,220,0,0,17,1,234,219,0,0,1,4,15,1,137,20,1,0,15,1,244,27,1,0,15,1,0,28,1,0,17,1,29, 
  3,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,17,0,0,0,52,0,0,0,1,0,17,1,137,20,1,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,2,0, 
  0,0,3,0,0,0,1,0,19,17,0,0,0,52,0,0,0,1,0,17,1,137,20,1,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,17,0,0,0,52,0,0,0,1,0, 
  17,1,137,20,1,0,1,2,21,1,39,0,0,0,108,92,1,0,3,0,0,0,1,0,0,0,2,48,1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0,0,1,8,19,33,0,0,0,97,0,0,0,1,0,1,21, 
  1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98, 
  0,0,0,1,0,15,1,249,19,1,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,249,19,1,0,15,1,43,206,0,0,17, 
  1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,186,85,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,144,2,128,33,16,4,128,26,0,2,128,11, 
  144,1,128,44,128,3,128,29,32,3,128,6,144,1,128,7,152,1,128,1,15,1,249,19,1,0,17,1,19,124,0,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,249,19,1,0,1,19,33,0,0,0,97,0,0,0, 
  1,0,17,1,249,19,1,0,1,19,11,0,0,0,38,0,0,0,2,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,249,19,1,0,1,15,1,249,19,1,0,17,1,185,204,0,0,1,2,21,0,116,1,0,0,255, 
  255,255,255,17,0,0,0,4,0,0,0,16,144,6,128,17,40,134,131,2,16,11,128,3,128,10,128,20,152,5,128,5,240,9,129,6,96,9,128,23,120,4,128,24,16,4,128,21,8,5,128,10,208,8,128,11,64,8,128,12, 
  176,71,128,28,168,195,128,14,32,7,128,33,64,3,128,44,176,2,128,19,33,0,0,0,98,0,0,0,1,0,17,1,137,20,1,0,1,15,1,137,20,1,0,17,1,240,25,1,0,1,15,1,137,20,1,0,17,1,202,24, 
  1,0,1,15,1,137,20,1,0,17,1,48,19,1,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,137,20,1,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,137,20,1,0,1,19,3,0,0,0,14,0,0, 
  0,1,0,17,1,137,20,1,0,1,15,1,137,20,1,0,17,1,254,21,1,0,1,19,2,0,0,0,10,0,0,0,1,0,17,1,137,20,1,0,1,19,2,0,0,0,9,0,0,0,1,0,17,1,137,20,1,0,1,19, 
  2,0,0,0,8,0,0,0,1,0,17,1,137,20,1,0,1,19,2,0,0,0,7,0,0,0,1,0,17,1,137,20,1,0,1,19,2,0,0,0,6,0,0,0,1,0,17,1,137,20,1,0,1,19,2,0,0,0,5,0, 
  0,0,1,0,17,1,137,20,1,0,1,19,2,0,0,0,4,0,0,0,1,0,17,1,137,20,1,0,1,19,2,0,0,0,3,0,0,0,1,0,17,1,137,20,1,0,1,19,17,0,0,0,52,0,0,0,1,0,17,1, 
  137,20,1,0,1,2,21,1,191,2,0,0,160,91,1,0,22,0,0,0,4,0,0,0,16,232,73,133,1,80,67,129,2,80,3,128,3,224,84,131,4,200,83,131,5,176,18,128,17,72,137,131,23,104,7,128,8,16,146,130, 
  9,112,145,130,10,88,16,128,11,64,15,128,12,40,14,128,13,184,12,128,14,160,11,128,15,136,10,128,19,168,8,128,20,56,8,128,24,152,6,128,25,160,5,128,33,168,4,128,48,88,3,128,8,4,19,44,0,0,0,129, 
  0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,190,24,1,0,15,1,139,41,1,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,190,24,1,0,15,1,197,211,0,0,17, 
  1,159,210,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,15,1,190,24,1,0,15,1,197,211,0,0,17,1,108,210,0,0,1,4,15,1,190,24,1,0,15,1,174,31,1,0,15,1,186,31,1,0,17,1,179,28, 
  1,0,1,4,15,1,190,24,1,0,15,1,174,31,1,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,19,16,0,0,0,46,0,0,0,3,0,14,1,4,15,1,190,24,1,0,15,1,0,28,1,0,17,1,23,16, 
  1,0,1,4,15,1,190,24,1,0,15,1,0,28,1,0,17,1,46,4,1,0,1,4,15,1,190,24,1,0,15,1,34,4,1,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,2,0,0,0, 
  6,0,0,0,1,0,19,17,0,0,0,53,0,0,0,2,0,1,4,19,10,0,0,0,36,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,17,0,0,0,53,0,0,0,2,0,1,4,19,20,0,0,0,58, 
  0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,17,0,0,0,53,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,2,0,0,0,6,0,0,0, 
  1,0,19,17,0,0,0,53,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,17,0,0,0,53,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1, 
  0,19,2,0,0,0,6,0,0,0,1,0,19,17,0,0,0,53,0,0,0,2,0,1,4,15,1,190,24,1,0,15,1,23,220,0,0,17,1,234,219,0,0,1,4,15,1,190,24,1,0,15,1,0,28,1,0,17,1,29, 
  3,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,17,0,0,0,53,0,0,0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,2,0,0,0,3,0,0,0, 
  1,0,19,17,0,0,0,53,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,17,0,0,0,53,0,0,0,2,0,1,2,19,17,0,0,0,53,0,0,0,2, 
  0,1,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0, 
  0,0,98,0,0,0,1,0,15,1,96,25,1,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,96,25,1,0,15,1,43,206, 
  0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0,0,1,4,17,1,221,133,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,240,1,128,33,136,2,128,26,232, 
  3,128,11,128,2,128,44,88,3,128,29,144,1,128,6,128,2,128,7,240,2,128,19,11,0,0,0,38,0,0,0,2,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,96,25,1,0,1,1,15,1,96,25,1,0,17, 
  1,185,204,0,0,1,15,1,96,25,1,0,17,1,58,138,0,0,1,19,33,0,0,0,98,0,0,0,1,0,17,1,96,25,1,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,96,25,1,0,1,2,21,1,235,0, 
  0,0,69,93,1,0,9,0,0,0,3,0,0,0,40,200,5,128,1,192,197,129,2,192,5,128,35,48,4,128,36,184,6,128,37,32,5,128,38,80,2,128,39,64,3,128,41,176,1,128,4,15,1,231,26,1,0,15,1,95, 
  68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,231,26,1,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0, 
  0,1,0,17,1,231,26,1,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,231,26,1,0,1,4,15,1,231,26,1,0,15,1,95,68,0,0,17,1,251,60,0,0,1, 
  8,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,231,26,1,0,1,4,15,1,231,26,1,0,15,1,95,68,0,0,17,1,145,44,0,0,1,19,12,0,0,0,40,0,0,0, 
  1,0,1,21,0,62,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,231,26,1,0,1,15,1,231,26,1,0,17,1,38, 
  27,1,0,1,1,2,21,1,193,0,0,0,69,93,1,0,9,0,0,0,3,0,0,0,40,240,3,128,1,176,196,129,2,176,4,128,35,176,1,128,36,184,4,128,37,152,5,128,38,112,2,128,39,48,3,128,41,40,5,128, 
  4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,92,0,0,0,1,0, 
  19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,8,4,15,1,197,16,0,0,17,1,145,44,0,0,1,4,15,1,197,16,0,0,17,1, 
  221,8,0,0,1,4,15,1,197,16,0,0,17,1,251,60,0,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,19,17,0,0,0,52,0,0,0,1,0,1,19,2,0,0,0,10,0,0,0,1,0,1,21,1,50,0, 
  0,0,52,50,1,0,3,0,0,0,1,0,0,0,2,240,0,128,1,240,64,128,21,248,0,128,8,4,19,22,0,0,0,60,0,0,0,1,0,17,1,63,28,1,0,1,2,21,0,40,0,0,0,255,255,255,255,2,0,0, 
  0,1,0,0,0,22,208,0,128,21,56,1,128,15,1,63,28,1,0,17,1,104,28,1,0,1,1,2,21,1,62,0,0,0,146,50,1,0,4,0,0,0,2,0,0,0,21,136,1,128,1,16,193,127,2,16,65,128,22,24, 
  1,128,8,4,19,21,0,0,0,59,0,0,0,3,0,14,1,4,19,22,0,0,0,61,0,0,0,2,0,1,2,19,3,0,0,0,15,0,0,0,1,0,1,21,1,201,0,0,0,108,49,1,0,7,0,0,0,2,0,0, 
  0,48,120,1,128,1,112,193,128,2,112,193,128,23,24,4,128,13,88,133,128,22,184,4,128,25,200,2,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,125,29,1,0,15, 
  1,24,85,0,0,17,1,31,33,1,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,125,29,1,0,15,1,24,85,0,0,17,1,230,7,0,0,1,4,15,1,125,29,1, 
  0,15,1,218,7,0,0,17,1,12,28,1,0,1,4,15,1,125,29,1,0,15,1,206,7,0,0,17,1,51,7,0,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,25,0,0,0,69,0,0,0,1,0,17,1,125, 
  29,1,0,1,2,21,0,189,0,0,0,255,255,255,255,9,0,0,0,3,0,0,0,24,160,3,128,25,56,67,129,20,200,132,129,35,64,2,128,12,88,133,127,21,56,4,128,33,208,2,128,23,48,4,128,44,176,1,128,19, 
  33,0,0,0,98,0,0,0,1,0,17,1,125,29,1,0,1,19,25,0,0,0,72,0,0,0,1,0,17,1,125,29,1,0,1,15,1,125,29,1,0,17,1,183,30,1,0,1,15,1,125,29,1,0,17,1,59,30,1,0, 
  1,19,33,0,0,0,97,0,0,0,1,0,17,1,125,29,1,0,1,1,19,25,0,0,0,71,0,0,0,1,0,17,1,125,29,1,0,1,19,25,0,0,0,69,0,0,0,1,0,17,1,125,29,1,0,1,19,25,0,0, 
  0,70,0,0,0,1,0,17,1,125,29,1,0,1,2,21,7,96,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,47,144,1,128,45,208,193,128,42,80,194,128,43,16,66,127,93,144,2,128,94,80,1,128,4,17,1, 
  138,57,0,0,1,4,17,1,16,55,0,0,1,4,17,1,150,52,0,0,1,4,17,1,28,50,0,0,1,4,17,1,149,46,0,0,1,4,19,23,0,0,0,62,0,0,0,3,0,14,1,21,9,27,0,0,0,255,255,255, 
  255,2,0,0,0,1,0,0,0,6,208,0,128,5,208,0,128,8,2,21,1,235,0,0,0,225,56,1,0,9,0,0,0,3,0,0,0,40,176,1,128,1,112,197,129,2,112,5,128,35,104,6,128,36,208,4,128,37,48,4, 
  128,38,64,3,128,39,120,5,128,41,160,2,128,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,221,8, 
  0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,15,1,15,49,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,15,49,0,0, 
  15,1,95,68,0,0,17,1,145,44,0,0,1,8,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,15,49,0,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0, 
  0,41,0,0,0,1,0,17,1,15,49,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,19,2,0,0,0,3,0,0,0,1,0,1,19,3,0,0,0,16,0,0,0,1,0,1,21,1,39,0,0,0,18,94,1,0, 
  3,0,0,0,1,0,0,0,2,48,1,128,1,48,65,128,7,240,0,128,4,17,1,105,6,0,0,1,8,19,33,0,0,0,97,0,0,0,1,0,1,21,1,149,0,0,0,247,57,1,0,6,0,0,0,2,0,0,0,8, 
  104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,143,32,1,0,15,1,43,206,0,0,17, 
  1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,143,32,1,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121,44,0,0,17,1,86,9,0, 
  0,1,4,17,1,186,85,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,144,1,128,33,40,2,128,26,232,3,128,11,32,2,128,44,88,3,128,29,144,2,128,6,32,2,128,7,240,2, 
  128,19,33,0,0,0,97,0,0,0,1,0,17,1,143,32,1,0,1,1,15,1,143,32,1,0,17,1,185,204,0,0,1,19,11,0,0,0,38,0,0,0,2,0,1,15,1,143,32,1,0,17,1,19,124,0,0,1,19,33, 
  0,0,0,98,0,0,0,1,0,17,1,143,32,1,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,143,32,1,0,1,2,21,1,235,0,0,0,237,50,1,0,9,0,0,0,3,0,0,0,40,136,4,128,1,224,195, 
  129,2,224,3,128,35,176,1,128,36,232,3,128,37,160,2,128,38,104,6,128,39,120,5,128,41,64,3,128,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,107,68,0,0,1,4, 
  15,1,107,68,0,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,107,68,0,0,15,1,95,68,0,0,17,1,221,8,0,0,1,8,4,15,1,107,68,0,0,15,1,95,68,0,0,17,1,145,44,0,0,1, 
  4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,107,68,0,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,107,68,0,0,1, 
  4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,107,68,0,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,0,121,1,0,0,255,255,255,255,18,0,0,0,4,0,0,0, 
  0,192,75,130,1,88,203,131,2,200,10,128,3,56,10,128,20,184,5,128,5,168,9,130,6,24,9,128,23,152,4,128,24,48,4,128,16,72,6,128,10,136,8,128,11,248,7,128,12,104,199,128,21,40,5,128,14,216,6,128, 
  28,200,131,128,33,96,3,128,44,208,2,128,19,33,0,0,0,98,0,0,0,1,0,17,1,22,34,1,0,1,15,1,22,34,1,0,17,1,123,39,1,0,1,15,1,22,34,1,0,17,1,85,38,1,0,1,15,1,22,34, 
  1,0,17,1,198,31,1,0,1,19,3,0,0,0,16,0,0,0,1,0,17,1,22,34,1,0,1,19,3,0,0,0,15,0,0,0,1,0,17,1,22,34,1,0,1,19,3,0,0,0,14,0,0,0,1,0,17,1,22,34, 
  1,0,1,19,2,0,0,0,10,0,0,0,1,0,17,1,22,34,1,0,1,19,2,0,0,0,9,0,0,0,1,0,17,1,22,34,1,0,1,19,2,0,0,0,8,0,0,0,1,0,17,1,22,34,1,0,1,19,2,0, 
  0,0,7,0,0,0,1,0,17,1,22,34,1,0,1,19,2,0,0,0,6,0,0,0,1,0,17,1,22,34,1,0,1,19,2,0,0,0,5,0,0,0,1,0,17,1,22,34,1,0,1,19,2,0,0,0,4,0,0,0, 
  1,0,17,1,22,34,1,0,1,19,2,0,0,0,3,0,0,0,1,0,17,1,22,34,1,0,1,19,1,0,0,0,1,0,0,0,1,0,17,1,22,34,1,0,1,15,1,22,34,1,0,17,1,144,35,1,0,1,1,2, 
  21,1,173,2,0,0,152,41,1,0,21,0,0,0,4,0,0,0,16,88,9,133,1,48,67,129,2,48,3,128,3,80,84,131,4,56,19,128,5,32,18,128,17,184,72,131,23,72,7,128,8,128,81,130,9,224,80,130,10,200, 
  15,128,11,176,14,128,12,152,13,128,13,40,12,128,14,16,11,128,15,248,9,128,19,24,8,128,24,120,6,128,25,128,5,128,33,136,4,128,48,56,3,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0, 
  98,0,0,0,1,0,15,1,73,38,1,0,15,1,139,41,1,0,17,1,31,33,1,0,1,4,19,28,0,0,0,82,0,0,0,1,0,15,1,73,38,1,0,15,1,197,211,0,0,17,1,159,210,0,0,1,4,19,24,0, 
  0,0,63,0,0,0,1,0,15,1,73,38,1,0,15,1,197,211,0,0,17,1,108,210,0,0,1,4,15,1,73,38,1,0,15,1,174,31,1,0,15,1,186,31,1,0,17,1,179,28,1,0,1,4,15,1,73,38,1,0, 
  15,1,174,31,1,0,15,1,167,28,1,0,17,1,12,28,1,0,1,4,15,1,73,38,1,0,15,1,0,28,1,0,17,1,23,16,1,0,1,4,15,1,73,38,1,0,15,1,0,28,1,0,17,1,46,4,1,0,1,4, 
  15,1,73,38,1,0,15,1,34,4,1,0,17,1,35,220,0,0,1,4,19,10,0,0,0,37,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,1,0,0,0,2,0,0,0,2,0,1,4,19,10,0,0,0, 
  36,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,1,0,0,0,2,0,0,0,2,0,1,4,19,20,0,0,0,58,0,0,0,1,0,19,3,0,0,0,14,0,0,0,1,0,19,2,0,0,0,3,0,0, 
  0,1,0,19,1,0,0,0,2,0,0,0,2,0,1,4,19,10,0,0,0,34,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,1,0,0,0,2,0,0,0,2,0,1,4,19,10,0,0,0,33,0,0,0, 
  1,0,19,2,0,0,0,6,0,0,0,1,0,19,1,0,0,0,2,0,0,0,2,0,1,4,19,10,0,0,0,32,0,0,0,1,0,19,2,0,0,0,6,0,0,0,1,0,19,1,0,0,0,2,0,0,0,2,0,1, 
  4,15,1,73,38,1,0,15,1,23,220,0,0,17,1,234,219,0,0,1,4,15,1,73,38,1,0,15,1,0,28,1,0,17,1,29,3,0,0,1,4,19,3,0,0,0,13,0,0,0,1,0,19,2,0,0,0,3,0,0, 
  0,1,0,19,1,0,0,0,2,0,0,0,2,0,1,4,19,3,0,0,0,12,0,0,0,1,0,19,2,0,0,0,3,0,0,0,1,0,19,1,0,0,0,2,0,0,0,2,0,1,4,19,3,0,0,0,11,0,0,0, 
  1,0,19,2,0,0,0,3,0,0,0,1,0,19,1,0,0,0,2,0,0,0,2,0,1,19,0,0,0,0,0,0,0,0,1,0,1,19,1,0,0,0,2,0,0,0,2,0,1,21,1,149,0,0,0,247,57,1,0,6, 
  0,0,0,2,0,0,0,8,104,68,129,1,80,129,128,2,80,1,128,17,248,67,128,25,168,2,128,48,88,1,128,8,4,19,44,0,0,0,129,0,0,0,1,0,19,33,0,0,0,98,0,0,0,1,0,15,1,235,38,1, 
  0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,19,24,0,0,0,63,0,0,0,1,0,19,33,0,0,0,97,0,0,0,1,0,15,1,235,38,1,0,15,1,43,206,0,0,17,1,185,204,0,0,1,4,15,1,121, 
  44,0,0,17,1,86,9,0,0,1,4,17,1,221,133,0,0,1,2,21,0,143,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,24,88,3,128,33,144,1,128,26,232,3,128,11,88,2,128,44,96,2,128,29,248,1, 
  128,6,88,2,128,7,240,2,128,15,1,235,38,1,0,17,1,185,204,0,0,1,19,11,0,0,0,38,0,0,0,2,0,1,1,19,33,0,0,0,98,0,0,0,1,0,17,1,235,38,1,0,1,15,1,235,38,1,0,17, 
  1,58,138,0,0,1,19,33,0,0,0,97,0,0,0,1,0,17,1,235,38,1,0,1,19,7,0,0,0,24,0,0,0,1,0,17,1,235,38,1,0,1,2,21,1,235,0,0,0,224,94,1,0,9,0,0,0,3,0,0, 
  0,40,224,3,128,1,96,198,129,2,96,6,128,35,104,6,128,36,208,4,128,37,160,2,128,38,112,5,128,39,176,1,128,41,64,3,128,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0, 
  17,1,114,40,1,0,1,4,15,1,114,40,1,0,15,1,95,68,0,0,17,1,251,60,0,0,1,4,15,1,114,40,1,0,15,1,95,68,0,0,17,1,221,8,0,0,1,4,19,31,0,0,0,93,0,0,0,1,0,19, 
  13,0,0,0,41,0,0,0,1,0,17,1,114,40,1,0,1,4,15,1,114,40,1,0,15,1,95,68,0,0,17,1,145,44,0,0,1,4,19,31,0,0,0,91,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0, 
  17,1,114,40,1,0,1,8,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,41,0,0,0,1,0,17,1,114,40,1,0,1,19,12,0,0,0,40,0,0,0,1,0,1,21,0,62,0,0,0,255,255,255,255, 
  3,0,0,0,1,0,0,0,12,232,1,128,13,128,65,128,31,240,0,128,19,13,0,0,0,41,0,0,0,1,0,17,1,114,40,1,0,1,15,1,114,40,1,0,17,1,177,40,1,0,1,1,2,21,1,193,0,0,0,224, 
  94,1,0,9,0,0,0,3,0,0,0,40,32,2,128,1,144,197,129,2,144,5,128,35,224,2,128,36,176,1,128,37,152,5,128,38,160,3,128,39,96,4,128,41,32,5,128,4,15,1,197,16,0,0,17,1,145,44,0,0, 
  1,4,19,31,0,0,0,93,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,87,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,91,0,0,0,1, 
  0,19,13,0,0,0,42,0,0,0,2,0,1,4,19,31,0,0,0,92,0,0,0,1,0,19,13,0,0,0,42,0,0,0,2,0,1,4,15,1,197,16,0,0,17,1,221,8,0,0,1,8,4,15,1,197,16,0,0,17, 
  1,251,60,0,0,1,19,12,0,0,0,39,0,0,0,2,0,14,1,19,1,0,0,0,1,0,0,0,1,0,1,19,2,0,0,0,8,0,0,0,1,0,1,13,21,4,114,0,0,0,255,255,255,255,9,0,0,0,3,0, 
  0,0,40,240,65,128,64,176,1,128,91,88,131,129,35,160,194,127,92,96,2,128,45,224,2,128,46,32,3,128,95,224,2,128,123,40,2,128,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,8,0,0,0,1, 
  3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128, 
  5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,18,3,0, 
  0,0,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192,127,3,17,1,159,42,1,0,1,21,2,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,3,176,0,128,3,17, 
  1,159,42,1,0,1,1,18,33,0,0,0,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192,127,3,17,1,159,42,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0, 
  1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,159,42,1,0,1,1,21,4,40,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,32,8,65,128,110,208,0,128,3,18,15,0,0,0,1,3,18,14,0,0,0, 
  1,24,3,17,1,26,43,1,0,1,18,5,0,0,0,1,18,3,0,0,0,1,18,48,0,0,0,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192,127,3,17,1,38,43,1,0, 
  1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,13,0,0,0,21,2,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,4, 
  176,0,128,3,17,1,112,43,1,0,1,1,18,2,0,0,0,1,18,1,0,0,0,1,18,9,0,0,0,21,4,96,0,0,0,255,255,255,255,7,0,0,0,2,0,0,0,59,32,2,129,45,88,2,128,34,200,2,128,43, 
  144,66,127,91,232,65,128,95,168,65,128,123,112,1,128,3,18,16,0,0,0,1,3,17,1,36,44,1,0,1,3,18,24,0,0,0,1,3,18,12,0,0,0,1,3,18,25,0,0,0,1,3,18,10,0,0,0,1,3,18, 
  23,0,0,0,1,21,2,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,3,176,0,128,3,17,1,36,44,1,0,1,1,18,25,0,0,0,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95, 
  208,0,128,45,208,192,127,3,17,1,36,44,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,36,44,1,0,1,1,21,4,125,0,0,0,255,255,255,255, 
  10,0,0,0,3,0,0,0,40,16,66,128,64,208,1,128,91,176,131,129,35,192,194,127,92,128,2,128,45,0,3,129,46,120,3,128,95,0,3,128,123,72,2,128,125,64,3,128,3,17,1,160,43,1,0,1,3,18,17,0, 
  0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1,3,18,6,0,0,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0, 
  0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1, 
  38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,138,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,40,112,130,128,61,96,67,130,64,48,2,128,35,32,67,129,92,224,2,128,45,160,3,127,46,240,1,128, 
  95,160,3,128,91,24,68,128,123,168,2,128,125,224,3,128,3,17,1,54,46,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1, 
  0,1,3,17,1,19,46,1,0,1,3,17,1,38,43,1,0,1,3,18,6,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48, 
  130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,18,3,0,0,0,21,4,29,0, 
  0,0,255,255,255,255,1,0,0,0,0,0,0,0,123,176,0,128,3,18,7,0,0,0,1,1,18,4,0,0,0,21,4,80,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,40,208,1,128,45,144,1,129,91,8,130, 
  128,35,80,193,127,95,144,1,128,105,64,2,128,3,17,1,64,48,1,0,1,3,17,1,246,47,1,0,1,3,18,37,0,0,0,1,3,18,36,0,0,0,1,3,17,1,170,46,1,0,1,21,2,30,0,0,0,255,255,255, 
  255,1,0,0,0,0,0,0,0,3,176,0,128,3,17,1,246,47,1,0,1,1,18,35,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,116,48,1,128,45,240,64,128,95,240,0,128,3,17,1, 
  246,47,1,0,1,3,17,1,0,47,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,246,47,1,0,1,1,18,35,0,0,0,21,4,46,0,0,0,255, 
  255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45,48,193,127,101,240,0,128,3,17,1,86,47,1,0,1,3,17,1,246,47,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0, 
  128,3,208,0,128,3,17,1,246,47,1,0,1,1,18,35,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,114,240,0,128,45,48,65,128,95,48,1,128,3,17,1,172,47,1,0,1,3,17,1, 
  246,47,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,246,47,1,0,1,1,18,41,0,0,0,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1, 
  0,0,0,95,208,0,128,45,208,192,127,3,17,1,246,47,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,246,47,1,0,1,1,18,35,0,0,0,21, 
  4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192,127,3,17,1,246,47,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3, 
  17,1,246,47,1,0,1,1,21,4,54,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,108,48,129,128,107,112,1,128,116,240,0,128,3,17,1,16,49,1,0,1,3,17,1,211,48,1,0,1,3,17,1,119,48,1, 
  0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,101,176,0,128,3,17,1,150,48,1,0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,121,176,0,128,3,17,1,181, 
  48,1,0,1,2,21,4,29,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,115,176,0,128,3,18,39,0,0,0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,101,176,0,128,3,17,1, 
  242,48,1,0,1,2,21,4,29,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,110,176,0,128,3,18,40,0,0,0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,121,176,0,128,3,17, 
  1,47,49,1,0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,112,176,0,128,3,17,1,78,49,1,0,1,2,21,4,29,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,101,176,0,128, 
  3,18,38,0,0,0,1,2,21,4,57,0,0,0,255,255,255,255,4,0,0,0,2,0,0,0,64,72,1,128,45,136,1,128,34,16,1,128,95,136,1,128,3,18,22,0,0,0,1,3,17,1,232,49,1,0,1,3,17,1, 
  38,43,1,0,1,21,2,66,0,0,0,255,255,255,255,4,0,0,0,2,0,0,0,4,144,1,128,5,80,1,128,6,16,1,128,3,208,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43, 
  1,0,1,3,17,1,38,43,1,0,1,2,21,4,45,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,34,240,0,128,45,40,65,128,95,40,1,128,3,18,23,0,0,0,1,3,17,1,36,44,1,0,1,21,2,30, 
  0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,3,176,0,128,3,17,1,36,44,1,0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,92,176,0,128,3,17,1,132,50,1,0,1,21,2, 
  42,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,16,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,140,50,1,0,1,24,3,18,21,0,0,0,1,18,21,0,0, 
  0,1,21,4,41,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,34,16,65,128,92,208,0,128,3,17,1,132,50,1,0,1,3,18,22,0,0,0,1,21,2,42,0,0,0,255,255,255,255,2,0,0,0,1,0,0, 
  0,6,208,0,128,5,16,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,140,50,1,0,1,21,4,234,0,0,0,255,255,255,255,19,0,0,0,4,0,0,0,32,24,71,128,64,200,4,128,91, 
  144,196,131,35,216,6,128,92,80,4,128,93,24,196,130,94,224,3,128,95,160,3,128,40,160,6,128,41,104,6,128,42,48,6,128,43,248,197,125,44,192,5,126,45,128,5,126,46,64,5,126,47,8,5,126,109,96,131,128,123, 
  40,3,128,125,240,2,128,3,18,6,0,0,0,1,3,18,8,0,0,0,1,3,17,1,124,52,1,0,1,3,17,1,38,43,1,0,1,3,18,30,0,0,0,1,3,18,20,0,0,0,1,3,17,1,233,42,1,0,1,3, 
  18,19,0,0,0,1,3,17,1,160,43,1,0,1,3,18,29,0,0,0,1,3,17,1,54,46,1,0,1,3,17,1,50,52,1,0,1,3,18,34,0,0,0,1,3,18,27,0,0,0,1,3,18,26,0,0,0,1,3,18, 
  18,0,0,0,1,3,18,17,0,0,0,1,3,17,1,89,42,1,0,1,3,18,50,0,0,0,1,21,2,90,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,4,16,2,128,5,208,1,128,2,144,130,128,3,80,130, 
  128,6,144,1,128,7,80,1,128,3,17,1,36,54,1,0,1,3,17,1,154,43,1,0,1,3,17,1,30,54,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,18,28, 
  0,0,0,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192,127,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3, 
  208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45,48,193,127,97,240,0,128,3,17,1,210,52,1,0,1,3,17,1,38,43, 
  1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0, 
  0,116,240,0,128,45,48,65,128,95,48,1,128,3,17,1,40,53,1,0,1,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43, 
  1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45,48,193,127,99,240,0,128,3,17,1,126,53,1,0,1,3,17,1,38,43,1,0,1,21,2,34,0,0, 
  0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,104,240,0,128,45,48,65, 
  128,95,48,1,128,3,17,1,212,53,1,0,1,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,47,0,0, 
  0,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192,127,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0, 
  128,3,17,1,38,43,1,0,1,1,18,51,0,0,0,1,18,49,0,0,0,1,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,35,176,0,128,3,17,1,115,54,1,0,1,21,2,42,0,0,0,255,255, 
  255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,16,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,2,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192, 
  127,3,17,1,159,42,1,0,1,21,2,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,3,176,0,128,3,17,1,159,42,1,0,1,2,21,4,68,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,64,160, 
  1,128,41,104,193,128,34,48,1,128,95,224,1,128,45,224,1,128,3,18,22,0,0,0,1,3,18,18,0,0,0,1,3,17,1,232,49,1,0,1,3,17,1,38,43,1,0,1,21,2,66,0,0,0,255,255,255,255,4,0, 
  0,0,2,0,0,0,4,144,1,128,5,80,1,128,6,16,1,128,3,208,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,2,21,4,118,0,0, 
  0,255,255,255,255,9,0,0,0,3,0,0,0,94,232,1,128,41,120,3,128,42,64,3,128,43,8,67,129,44,208,2,128,45,152,2,128,46,88,130,126,47,32,2,128,123,176,1,128,3,18,8,0,0,0,1,3,18,30,0, 
  0,0,1,3,18,29,0,0,0,1,3,17,1,220,55,1,0,1,3,18,28,0,0,0,1,3,18,34,0,0,0,1,3,18,27,0,0,0,1,3,18,26,0,0,0,1,3,18,18,0,0,0,1,21,2,42,0,0,0,255, 
  255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,16,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,2,21,4,80,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,40,208,1,128,45,144, 
  1,129,91,8,130,128,35,80,193,127,95,144,1,128,105,64,2,128,3,17,1,64,48,1,0,1,3,17,1,246,47,1,0,1,3,18,37,0,0,0,1,3,18,36,0,0,0,1,3,17,1,170,46,1,0,1,21,2,30,0, 
  0,0,255,255,255,255,1,0,0,0,0,0,0,0,3,176,0,128,3,17,1,246,47,1,0,1,2,21,4,107,0,0,0,255,255,255,255,8,0,0,0,3,0,0,0,94,144,1,128,41,32,3,128,42,232,2,128,43,176,2, 
  128,44,120,2,128,45,64,2,128,46,0,130,126,47,200,1,128,3,18,30,0,0,0,1,3,18,29,0,0,0,1,3,17,1,220,55,1,0,1,3,18,28,0,0,0,1,3,18,34,0,0,0,1,3,18,27,0,0,0,1, 
  3,18,26,0,0,0,1,3,18,18,0,0,0,1,21,2,42,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,16,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,2,21,4,96, 
  0,0,0,255,255,255,255,7,0,0,0,2,0,0,0,46,24,130,129,45,88,2,129,42,200,130,127,43,144,66,128,47,224,1,128,93,168,1,128,94,112,1,128,3,18,30,0,0,0,1,3,18,20,0,0,0,1,3,18,29, 
  0,0,0,1,3,17,1,220,55,1,0,1,3,18,28,0,0,0,1,3,18,27,0,0,0,1,3,18,26,0,0,0,1,21,2,42,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,16,1,128,3, 
  17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,2,21,4,96,0,0,0,255,255,255,255,7,0,0,0,2,0,0,0,46,24,66,129,45,88,66,129,42,200,130,127,43,144,66,128,47,224,1,128,94,168,1,128,125,112, 
  1,128,3,18,6,0,0,0,1,3,18,30,0,0,0,1,3,18,29,0,0,0,1,3,17,1,220,55,1,0,1,3,18,28,0,0,0,1,3,18,27,0,0,0,1,3,18,26,0,0,0,1,21,2,42,0,0,0,255,255, 
  255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,16,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,2,21,4,68,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,40,48,129,128,45,224,1, 
  128,64,104,1,128,95,224,65,128,123,168,1,128,3,18,17,0,0,0,1,3,17,1,114,58,1,0,1,3,18,8,0,0,0,1,3,17,1,38,43,1,0,1,21,2,54,0,0,0,255,255,255,255,3,0,0,0,1,0,0, 
  0,6,240,0,128,3,112,65,128,5,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,38,43,1,0,1,2,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128, 
  45,208,192,127,3,17,1,36,44,1,0,1,21,2,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,3,176,0,128,3,17,1,36,44,1,0,1,2,21,4,137,0,0,0,255,255,255,255,11,0,0,0,3,0,0, 
  0,40,48,66,128,64,240,1,128,91,16,196,129,35,32,195,127,92,224,2,128,45,96,195,128,46,216,3,128,95,96,3,128,109,104,130,128,123,168,2,128,125,160,3,128,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1, 
  3,17,1,139,59,1,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1,3,18,6,0,0,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1, 
  21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1, 
  0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45,48,193,127,97,240,0,128,3,17,1,210,52,1, 
  0,1,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,21,4,150,0,0,0,255,255,255,255,12,0,0,0,3, 
  0,0,0,40,208,130,128,61,192,3,130,64,144,2,128,35,128,67,129,92,64,3,128,45,0,4,127,46,16,2,128,95,0,4,128,91,120,132,128,109,80,130,128,123,8,3,128,125,64,4,128,3,17,1,54,46,1,0,1,3, 
  17,1,124,52,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,19,46,1,0,1,3,17,1,38,43,1,0, 
  1,3,18,6,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0, 
  1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,138,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,40,176,66,128,64,112,2,128, 
  91,24,196,129,35,96,195,127,92,32,3,128,45,160,195,128,46,240,1,128,95,160,3,128,109,48,130,128,123,232,2,128,125,224,3,128,3,17,1,54,46,1,0,1,3,17,1,124,52,1,0,1,3,17,1,160,43,1,0,1, 
  3,18,17,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1,3,18,6,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255, 
  255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1, 
  0,1,3,17,1,32,43,1,0,1,2,21,4,172,0,0,0,255,255,255,255,14,0,0,0,3,0,0,0,40,16,3,130,41,56,4,128,61,112,4,130,35,248,131,129,92,184,3,128,45,176,68,127,46,80,2,128,95,176,4, 
  128,64,208,2,128,91,40,197,128,93,72,67,128,109,144,130,128,123,128,3,128,125,240,4,128,3,17,1,54,46,1,0,1,3,17,1,124,52,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,20,0, 
  0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,18,18,0,0,0,1,3,17,1,19,46,1,0,1,3,17,1,38,43,1,0,1,3,18,6,0,0,0,1,3,18,19,0, 
  0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1, 
  112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,160,0,0,0,255,255,255,255,13,0,0,0,3,0,0,0,40,240,130,128,41,24,4,128,64,176,2,128,35,216,67,129,92,152,3,128, 
  45,80,4,129,46,48,2,128,95,80,4,128,91,200,196,128,93,40,67,128,109,112,130,128,123,96,3,128,125,144,4,128,3,17,1,54,46,1,0,1,3,17,1,124,52,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0, 
  0,0,1,3,18,20,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,18,18,0,0,0,1,3,17,1,38,43,1,0,1,3,18,6,0,0,0,1,3,18,19,0,0, 
  0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112, 
  43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,137,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,40,48,130,128,41,96,3,128,64,240,1,128,35,32,67,129,92,224,2,128,45, 
  152,3,129,46,216,3,128,95,152,3,128,91,16,132,128,109,104,2,128,123,168,2,128,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,17,1,139,59,1,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0, 
  1,3,17,1,89,42,1,0,1,3,18,18,0,0,0,1,3,17,1,38,43,1,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128, 
  5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,150, 
  0,0,0,255,255,255,255,12,0,0,0,3,0,0,0,40,208,2,130,41,192,3,128,61,248,3,130,35,128,131,129,92,64,3,128,45,56,68,127,46,16,2,128,95,56,4,128,64,144,2,128,91,120,132,128,109,80,2,128,123, 
  8,3,128,3,17,1,54,46,1,0,1,3,17,1,124,52,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,18,18, 
  0,0,0,1,3,17,1,19,46,1,0,1,3,17,1,38,43,1,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1, 
  128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,138,0,0,0,255,255,255,255,11,0,0,0, 
  3,0,0,0,40,176,130,128,41,160,3,128,64,112,2,128,35,96,67,129,92,32,3,128,45,216,3,129,46,240,1,128,95,216,3,128,91,24,132,128,109,48,2,128,123,232,2,128,3,17,1,54,46,1,0,1,3,17,1,124, 
  52,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,18,18,0,0,0,1,3,17,1,38,43,1,0,1,3,18,19, 
  0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17, 
  1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,137,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,40,48,66,128,64,240,1,128,91,16,4,130,35,88,195,127,92,24,3, 
  128,45,152,195,128,46,216,3,128,95,152,3,128,93,104,66,128,109,160,2,128,123,224,2,128,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,20,0,0,0,1,3,17,1,139,59,1,0,1,3,18,8,0,0, 
  0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176, 
  1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21, 
  4,150,0,0,0,255,255,255,255,12,0,0,0,3,0,0,0,40,208,130,128,61,248,3,130,64,144,2,128,35,184,67,129,92,120,3,128,45,56,4,127,46,16,2,128,95,56,4,128,91,120,196,128,93,8,67,128,109,80,2, 
  128,123,64,3,128,3,17,1,54,46,1,0,1,3,17,1,124,52,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,20,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17, 
  1,89,42,1,0,1,3,17,1,19,46,1,0,1,3,17,1,38,43,1,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3, 
  240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,138,0,0,0,255,255,255,255,11,0, 
  0,0,3,0,0,0,40,176,66,128,64,112,2,128,91,24,4,130,35,152,195,127,92,88,3,128,45,216,195,128,46,240,1,128,95,216,3,128,93,232,66,128,109,48,2,128,123,32,3,128,3,17,1,54,46,1,0,1,3,17, 
  1,124,52,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,20,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1,3, 
  18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1, 
  3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,57,0,0,0,255,255,255,255,4,0,0,0,2,0,0,0,64,16,1,128,45,136,1,128,123,80,1,128,95,136,193,127,3, 
  17,1,114,58,1,0,1,3,18,8,0,0,0,1,3,17,1,38,43,1,0,1,21,2,54,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,6,240,0,128,3,112,65,128,5,48,1,128,3,17,1,154,43,1,0,1, 
  3,17,1,148,43,1,0,1,3,17,1,38,43,1,0,1,2,21,4,146,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,40,232,3,130,42,176,3,128,34,88,196,127,83,176,2,128,116,240,1,128,45,112,3,128,38, 
  32,196,128,95,112,2,128,64,48,3,128,78,240,66,128,102,48,2,128,3,17,1,126,74,1,0,1,3,17,1,220,72,1,0,1,3,17,1,146,72,1,0,1,3,17,1,70,71,1,0,1,3,17,1,250,69,1,0,1,3, 
  17,1,232,49,1,0,1,3,17,1,38,43,1,0,1,3,18,26,0,0,0,1,3,18,17,0,0,0,1,3,18,52,0,0,0,1,3,18,22,0,0,0,1,21,2,66,0,0,0,255,255,255,255,4,0,0,0,2,0,0, 
  0,4,144,1,128,5,80,1,128,6,16,1,128,3,208,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,2,18,48,0,0,0,21,4,46,0,0, 
  0,255,255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45,48,193,127,111,240,0,128,3,17,1,80,70,1,0,1,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4, 
  208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,110,240,0,128,45,48,65,128,95,48,1,128,3,17,1,166,70,1,0,1,3, 
  17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0, 
  0,1,0,0,0,95,48,129,128,45,48,193,127,101,240,0,128,3,17,1,252,70,1,0,1,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3, 
  17,1,38,43,1,0,1,1,18,55,0,0,0,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192,127,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0, 
  0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45,48,193,127,111,240,0,128,3,17,1, 
  156,71,1,0,1,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255, 
  255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45,48,193,127,109,240,0,128,3,17,1,242,71,1,0,1,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0, 
  128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,95,240,128,128,45,240,192,127,101,48,1,128,3,17,1,38,43,1,0,1,3,17,1, 
  72,72,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,54,0,0,0,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1, 
  0,0,0,95,208,0,128,45,208,192,127,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,53,0,0,0,21, 
  4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192,127,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3, 
  17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45,48,193,127,97,240,0,128,3,17,1,50,73,1,0,1,3,17,1,38,43,1,0,1,21, 
  2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,108,48,1, 
  128,45,240,64,128,95,240,0,128,3,17,1,38,43,1,0,1,3,17,1,136,73,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1, 
  18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,95,240,128,128,45,240,192,127,115,48,1,128,3,17,1,38,43,1,0,1,3,17,1,222,73,1,0,1,21,2,34,0,0,0,255,255,255, 
  255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45,48,193,127,101,240,0, 
  128,3,17,1,52,74,1,0,1,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,57,0,0,0,21,4,34, 
  0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192,127,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1, 
  38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,114,240,0,128,45,48,65,128,95,48,1,128,3,17,1,212,74,1,0,1,3,17,1,38,43,1,0,1,21,2,34, 
  0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48,0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45, 
  48,193,127,117,240,0,128,3,17,1,42,75,1,0,1,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,48, 
  0,0,0,21,4,46,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,95,48,129,128,45,48,193,127,101,240,0,128,3,17,1,128,75,1,0,1,3,17,1,38,43,1,0,1,21,2,34,0,0,0,255,255,255,255,2, 
  0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,18,56,0,0,0,21,4,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,95,208,0,128,45,208,192,127,3,17,1,38,43, 
  1,0,1,21,2,34,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,4,208,0,128,3,208,0,128,3,17,1,38,43,1,0,1,1,21,4,135,0,0,0,255,255,255,255,10,0,0,0,3,0,0,0,64,16,3,128, 
  42,144,3,128,34,0,196,127,83,144,2,128,116,208,1,128,45,80,3,128,38,200,131,128,95,80,2,128,78,208,66,128,102,16,2,128,3,17,1,126,74,1,0,1,3,17,1,220,72,1,0,1,3,17,1,146,72,1,0,1, 
  3,17,1,70,71,1,0,1,3,17,1,250,69,1,0,1,3,17,1,232,49,1,0,1,3,17,1,38,43,1,0,1,3,18,26,0,0,0,1,3,18,52,0,0,0,1,3,18,22,0,0,0,1,21,2,66,0,0,0,255, 
  255,255,255,4,0,0,0,2,0,0,0,4,144,1,128,5,80,1,128,6,16,1,128,3,208,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,2, 
  21,4,168,0,0,0,255,255,255,255,13,0,0,0,3,0,0,0,40,152,4,130,42,96,4,128,34,8,197,127,83,96,3,130,116,160,2,128,45,32,196,129,38,208,196,128,95,32,3,128,64,224,3,128,78,160,67,128,102,224, 
  2,128,123,104,2,128,125,48,2,128,3,18,6,0,0,0,1,3,18,8,0,0,0,1,3,17,1,126,74,1,0,1,3,17,1,220,72,1,0,1,3,17,1,146,72,1,0,1,3,17,1,70,71,1,0,1,3,17,1,250, 
  69,1,0,1,3,17,1,232,49,1,0,1,3,17,1,38,43,1,0,1,3,18,26,0,0,0,1,3,18,17,0,0,0,1,3,18,52,0,0,0,1,3,18,22,0,0,0,1,21,2,66,0,0,0,255,255,255,255,4,0, 
  0,0,2,0,0,0,4,144,1,128,5,80,1,128,6,16,1,128,3,208,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,2,21,4,96,0,0, 
  0,255,255,255,255,7,0,0,0,2,0,0,0,46,24,66,129,45,88,2,128,42,200,130,127,43,144,66,128,47,224,129,128,94,168,1,128,123,112,1,128,3,18,8,0,0,0,1,3,18,30,0,0,0,1,3,18,29,0,0, 
  0,1,3,17,1,220,55,1,0,1,3,18,28,0,0,0,1,3,18,27,0,0,0,1,3,18,26,0,0,0,1,21,2,42,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,16,1,128,3,17,1, 
  154,43,1,0,1,3,17,1,148,43,1,0,1,2,21,4,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,110,176,1,128,105,240,1,128,102,48,130,127,111,112,65,128,115,48,1,128,3,17,1,119,79,1,0,1, 
  3,17,1,58,79,1,0,1,3,17,1,253,78,1,0,1,3,17,1,192,78,1,0,1,3,17,1,131,78,1,0,1,21,2,42,0,0,0,255,255,255,255,2,0,0,0,1,0,0,0,6,208,0,128,5,16,1,128,3,17, 
  1,154,43,1,0,1,3,17,1,148,43,1,0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,108,176,0,128,3,17,1,162,78,1,0,1,2,21,4,29,0,0,0,255,255,255,255,1,0,0,0, 
  0,0,0,0,116,176,0,128,3,18,45,0,0,0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,110,176,0,128,3,17,1,223,78,1,0,1,2,21,4,29,0,0,0,255,255,255,255,1,0,0, 
  0,0,0,0,0,116,176,0,128,3,18,46,0,0,0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,117,176,0,128,3,17,1,28,79,1,0,1,2,21,4,29,0,0,0,255,255,255,255,1,0, 
  0,0,0,0,0,0,109,176,0,128,3,18,43,0,0,0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,98,176,0,128,3,17,1,89,79,1,0,1,2,21,4,29,0,0,0,255,255,255,255,1, 
  0,0,0,0,0,0,0,106,176,0,128,3,18,42,0,0,0,1,2,21,4,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,116,176,0,128,3,17,1,150,79,1,0,1,2,21,4,29,0,0,0,255,255,255,255, 
  1,0,0,0,0,0,0,0,114,176,0,128,3,18,44,0,0,0,1,2,21,4,160,0,0,0,255,255,255,255,13,0,0,0,3,0,0,0,40,176,2,130,41,216,3,128,61,16,4,130,35,152,131,129,92,88,3,128,45,80, 
  68,127,46,48,2,128,95,80,4,128,64,112,2,128,91,200,132,128,93,232,130,128,123,32,3,128,125,144,4,128,3,17,1,54,46,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,20,0,0,0,1, 
  3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,18,18,0,0,0,1,3,17,1,19,46,1,0,1,3,17,1,38,43,1,0,1,3,18,6,0,0,0,1,3,18,19,0,0,0,1, 
  21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1, 
  0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,148,0,0,0,255,255,255,255,12,0,0,0,3,0,0,0,40,144,130,128,41,184,3,128,64,80,2,128,35,120,67,129,92,56,3,128,45,240,3, 
  129,46,16,2,128,95,240,3,128,91,104,132,128,93,200,130,128,123,0,3,128,125,48,4,128,3,17,1,54,46,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,20,0,0,0,1,3,18,8,0,0, 
  0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,18,18,0,0,0,1,3,17,1,38,43,1,0,1,3,18,6,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0, 
  0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1, 
  32,43,1,0,1,2,21,4,126,0,0,0,255,255,255,255,10,0,0,0,3,0,0,0,40,80,66,128,64,16,2,128,91,184,131,129,35,0,195,127,92,192,2,128,45,64,3,129,46,208,1,128,95,64,3,128,123,136,2,128, 
  125,128,3,128,3,17,1,54,46,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1,3,18, 
  6,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17, 
  1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,2,52,0,0,0,255,255,255,255,3,0,0,0,1,0,0,0,4,96,129,128,5,40,1,128,6,240,0, 
  128,3,18,1,0,0,0,1,3,18,2,0,0,0,1,3,17,1,112,43,1,0,1,2,21,4,125,0,0,0,255,255,255,255,10,0,0,0,3,0,0,0,32,8,66,128,40,208,65,128,64,56,3,128,35,184,66,129,92,120, 
  2,128,45,248,2,128,46,120,3,128,95,248,2,128,91,176,67,128,123,64,2,128,3,18,17,0,0,0,1,3,18,50,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3, 
  17,1,38,43,1,0,1,3,17,1,96,83,1,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1,21,2,90,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,4,16,2,128,5,208,1,128,2,144,130,128,3, 
  80,130,128,6,144,1,128,7,80,1,128,3,17,1,36,54,1,0,1,3,17,1,154,43,1,0,1,3,17,1,30,54,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2, 
  18,9,0,0,0,21,4,85,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,59,200,1,129,45,0,2,128,34,112,2,128,43,56,66,127,91,144,65,128,95,80,1,128,3,17,1,36,44,1,0,1,3,18,24,0,0, 
  0,1,3,18,12,0,0,0,1,3,18,25,0,0,0,1,3,18,10,0,0,0,1,3,18,23,0,0,0,1,21,2,30,0,0,0,255,255,255,255,1,0,0,0,0,0,0,0,3,176,0,128,3,17,1,36,44,1,0,1, 
  1,21,4,136,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,32,40,66,128,40,240,65,128,64,88,3,128,35,216,66,129,92,152,2,128,45,24,67,129,46,208,3,128,95,24,3,128,91,8,68,128,123,96,2,128,125, 
  152,3,128,3,18,17,0,0,0,1,3,18,50,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1,3,17,1,96,83,1,0,1,3,18,6,0, 
  0,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1,21,2,90,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,4,16,2,128,5,208,1,128,2,144,130,128,3,80,130,128,6,144,1,128,7,80,1,128,3, 
  17,1,36,54,1,0,1,3,17,1,154,43,1,0,1,3,17,1,30,54,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,137,0,0,0,255,255,255,255,11,0, 
  0,0,3,0,0,0,32,104,66,128,40,48,66,128,64,152,3,128,35,24,67,129,92,216,2,128,45,88,67,129,46,240,1,128,95,88,3,128,91,16,68,128,123,160,2,128,125,216,3,128,3,17,1,54,46,1,0,1,3,18, 
  17,0,0,0,1,3,18,50,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1,3,17,1,96,83,1,0,1,3,18,6,0,0,0,1,3,18, 
  19,0,0,0,1,21,2,90,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,4,16,2,128,5,208,1,128,2,144,130,128,3,80,130,128,6,144,1,128,7,80,1,128,3,17,1,36,54,1,0,1,3,17,1,154,43, 
  1,0,1,3,17,1,30,54,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,136,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,32,40,130,128,41,24, 
  3,128,40,240,129,129,35,216,130,129,92,152,2,128,45,80,3,128,46,208,3,128,95,80,3,128,64,144,3,128,91,8,68,128,123,96,2,128,3,18,17,0,0,0,1,3,18,50,0,0,0,1,3,18,8,0,0,0,1,3, 
  17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,18,18,0,0,0,1,3,17,1,38,43,1,0,1,3,17,1,96,83,1,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1,21,2,90,0,0,0,255,255, 
  255,255,6,0,0,0,2,0,0,0,4,16,2,128,5,208,1,128,2,144,130,128,3,80,130,128,6,144,1,128,7,80,1,128,3,17,1,36,54,1,0,1,3,17,1,154,43,1,0,1,3,17,1,30,54,1,0,1,3,17, 
  1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,137,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,32,104,130,128,41,88,3,128,40,48,130,129,35,24,131,129,92,216,2, 
  128,45,144,3,128,46,240,1,128,95,144,3,128,64,208,3,128,91,16,68,128,123,160,2,128,3,17,1,54,46,1,0,1,3,18,17,0,0,0,1,3,18,50,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1, 
  0,1,3,17,1,89,42,1,0,1,3,18,18,0,0,0,1,3,17,1,38,43,1,0,1,3,17,1,96,83,1,0,1,3,18,19,0,0,0,1,21,2,90,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,4,16, 
  2,128,5,208,1,128,2,144,130,128,3,80,130,128,6,144,1,128,7,80,1,128,3,17,1,36,54,1,0,1,3,17,1,154,43,1,0,1,3,17,1,30,54,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1, 
  0,1,3,17,1,32,43,1,0,1,2,21,4,136,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,32,96,66,128,40,240,65,128,64,144,3,128,35,16,67,129,92,208,2,128,45,80,3,129,46,208,3,128,95,80,3, 
  128,91,8,132,128,93,40,2,128,123,152,2,128,3,18,17,0,0,0,1,3,18,20,0,0,0,1,3,18,50,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1, 
  38,43,1,0,1,3,17,1,96,83,1,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1,21,2,90,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,4,16,2,128,5,208,1,128,2,144,130,128,3,80,130, 
  128,6,144,1,128,7,80,1,128,3,17,1,36,54,1,0,1,3,17,1,154,43,1,0,1,3,17,1,30,54,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4, 
  137,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,32,160,66,128,40,48,66,128,64,208,3,128,35,80,67,129,92,16,3,128,45,144,3,129,46,240,1,128,95,144,3,128,91,16,132,128,93,104,2,128,123,216,2,128, 
  3,17,1,54,46,1,0,1,3,18,17,0,0,0,1,3,18,20,0,0,0,1,3,18,50,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1, 
  3,17,1,96,83,1,0,1,3,18,19,0,0,0,1,21,2,90,0,0,0,255,255,255,255,6,0,0,0,2,0,0,0,4,16,2,128,5,208,1,128,2,144,130,128,3,80,130,128,6,144,1,128,7,80,1,128,3,17,1, 
  36,54,1,0,1,3,17,1,154,43,1,0,1,3,17,1,30,54,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,125,0,0,0,255,255,255,255,10,0,0,0, 
  3,0,0,0,40,16,130,128,41,0,3,128,64,208,1,128,35,192,66,129,92,128,2,128,45,56,3,128,46,120,3,128,95,56,3,128,91,176,67,128,123,72,2,128,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3, 
  18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,18,18,0,0,0,1,3,17,1,38,43,1,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255, 
  255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0, 
  1,3,17,1,32,43,1,0,1,2,21,4,138,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,40,112,2,130,41,96,3,128,61,152,3,128,35,32,131,129,92,224,2,128,45,216,67,127,46,240,1,128,95,216,3,128, 
  64,48,2,128,91,24,68,128,123,168,2,128,3,17,1,54,46,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,18, 
  18,0,0,0,1,3,17,1,19,46,1,0,1,3,17,1,38,43,1,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240, 
  1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,126,0,0,0,255,255,255,255,10,0,0, 
  0,3,0,0,0,40,80,130,128,41,64,3,128,64,16,2,128,35,0,67,129,92,192,2,128,45,120,3,128,46,208,1,128,95,120,3,128,91,184,67,128,123,136,2,128,3,17,1,54,46,1,0,1,3,17,1,160,43,1,0, 
  1,3,18,17,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,18,18,0,0,0,1,3,17,1,38,43,1,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255, 
  255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43, 
  1,0,1,3,17,1,32,43,1,0,1,2,21,4,125,0,0,0,255,255,255,255,10,0,0,0,3,0,0,0,40,16,66,128,64,208,1,128,91,176,195,129,35,248,194,127,92,184,2,128,45,56,195,128,46,120,3,128,95,56, 
  3,128,93,72,2,128,123,128,2,128,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,20,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43, 
  1,0,1,3,18,4,0,0,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43, 
  1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,138,0,0,0,255,255,255,255,11,0,0,0,3,0,0,0,40,112,130,128,61,152, 
  3,130,64,48,2,128,35,88,67,129,92,24,3,128,45,216,3,127,46,240,1,128,95,216,3,128,91,24,132,128,93,168,2,128,123,224,2,128,3,17,1,54,46,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0, 
  1,3,18,20,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,19,46,1,0,1,3,17,1,38,43,1,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0, 
  255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38, 
  43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,126,0,0,0,255,255,255,255,10,0,0,0,3,0,0,0,40,80,66,128,64,16,2,128,91,184,195,129,35,56,195,127,92,248,2,128,45,120,195,128,46,208,1,128,95, 
  120,3,128,93,136,2,128,123,192,2,128,3,17,1,54,46,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,20,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89, 
  42,1,0,1,3,17,1,38,43,1,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1, 
  154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2,21,4,127,0,0,0,255,255,255,255,10,0,0,0,3,0,0,0,40,80,130,128, 
  61,64,3,128,64,16,2,128,35,0,67,129,92,192,2,128,45,128,3,127,46,208,1,128,95,128,3,128,91,192,67,128,123,136,2,128,3,17,1,54,46,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3, 
  18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,19,46,1,0,1,3,17,1,38,43,1,0,1,3,18,19,0,0,0,1,21,2,78,0,0,0,255,255,255,255,5,0,0,0,2, 
  0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0,1,3,17,1,38,43,1,0,1,3,17,1,32,43, 
  1,0,1,2,21,4,115,0,0,0,255,255,255,255,9,0,0,0,3,0,0,0,40,48,66,128,64,240,1,128,91,96,131,129,35,224,194,127,92,160,2,128,45,32,3,128,46,176,1,128,95,32,3,128,123,104,2,128,3,17, 
  1,54,46,1,0,1,3,17,1,160,43,1,0,1,3,18,17,0,0,0,1,3,18,8,0,0,0,1,3,17,1,233,42,1,0,1,3,17,1,89,42,1,0,1,3,17,1,38,43,1,0,1,3,18,19,0,0,0,1,21, 
  2,78,0,0,0,255,255,255,255,5,0,0,0,2,0,0,0,4,176,1,128,5,112,1,128,2,48,130,128,3,240,1,128,6,48,1,128,3,17,1,154,43,1,0,1,3,17,1,148,43,1,0,1,3,17,1,112,43,1,0, 
  1,3,17,1,38,43,1,0,1,3,17,1,32,43,1,0,1,2, 
];

pub mod ast{
  impl AstObject for ASTNode {}
  type ASTSlot = (ASTNode, TokenRange, TokenRange);
  use super::*; 
  type Node = ASTNode;
  
  pub fn default_from<'a> (mut reader: UTF8StringReader)-> Result<Vec<ASTNode>, SherpaParseError> {
    let reduce_functions = ReduceFunctions::<_, u32, true>::new();
    let mut parser = Parser::new(&mut reader, &bytecode);
    parser.init_parser(8);
    let AstSlot (ref_0, __rule_rng__, _) = parser.parse_ast(&reduce_functions.0, &mut None)?;
    let obj_0_0 = ref_0.into_nodes();
    Ok(obj_0_0)
  }
}