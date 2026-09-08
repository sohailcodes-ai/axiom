use crate::hir::{HIRProgram, HIRFunction, HIRStmt, HIRExpr, HIRBinOp, HIRUnaryOp};

pub struct WasmBackend {
    module: WasmModule,
}

struct WasmModule {
    types: Vec<WasmType>,
    functions: Vec<u32>,
    memories: Vec<WasmMemory>,
    globals: Vec<WasmGlobal>,
    data_segments: Vec<WasmDataSegment>,
    code_section: Vec<WasmFunctionBody>,
    export_section: Vec<WasmExport>,
    function_names: Vec<(u32, String)>,
}

#[derive(Debug, Clone)]
enum WasmType {
    Func(Vec<WasmValType>, Vec<WasmValType>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WasmValType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

impl WasmValType {
    fn type_byte(&self) -> u8 {
        match self {
            WasmValType::I32 => 0x7F,
            WasmValType::I64 => 0x7E,
            WasmValType::F32 => 0x7D,
            WasmValType::F64 => 0x7C,
            WasmValType::V128 => 0x7B,
            WasmValType::FuncRef => 0x70,
            WasmValType::ExternRef => 0x6F,
        }
    }
}

#[derive(Debug, Clone)]
struct WasmMemory {
    min_pages: u32,
    max_pages: Option<u32>,
}

#[derive(Debug, Clone)]
struct WasmGlobal {
    val_type: WasmValType,
    mutable: bool,
    init_value: WasmInitExpr,
}

#[derive(Debug, Clone)]
enum WasmInitExpr {
    I32Const(i32),
    I64Const(i64),
}

#[derive(Debug, Clone)]
struct WasmDataSegment {
    offset: WasmInitExpr,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct WasmFunctionBody {
    locals: Vec<(u32, WasmValType)>,
    code: Vec<u8>,
}

#[derive(Debug, Clone)]
struct WasmExport {
    name: String,
    kind: WasmExportKind,
    index: u32,
}

#[derive(Debug, Clone, Copy)]
enum WasmExportKind {
    Func = 0,
    Table = 1,
    Memory = 2,
    Global = 3,
}

impl WasmBackend {
    pub fn new() -> Self {
        WasmBackend {
            module: WasmModule {
                types: Vec::new(),
                functions: Vec::new(),
                memories: Vec::new(),
                globals: Vec::new(),
                data_segments: Vec::new(),
                code_section: Vec::new(),
                export_section: Vec::new(),
                function_names: Vec::new(),
            },
        }
    }

    pub fn compile(&mut self, program: &HIRProgram) -> Result<Vec<u8>, String> {
        self.module.memories.push(WasmMemory {
            min_pages: 1,
            max_pages: Some(256),
        });

        for func in &program.functions {
            self.compile_function(func)?;
        }

        self.module.export_section.push(WasmExport {
            name: "memory".to_string(),
            kind: WasmExportKind::Memory,
            index: 0,
        });

        Ok(self.emit_module())
    }

    fn compile_function(&mut self, func: &HIRFunction) -> Result<(), String> {
        let mut param_types = Vec::new();
        for _ in 0..func.param_count {
            param_types.push(WasmValType::I32);
        }

        let mut result_types = Vec::new();
        if !func.body.is_empty() {
            if let Some(HIRStmt::Return(Some(_))) = func.body.last() {
                result_types.push(WasmValType::I32);
            }
        }

        self.module.types.push(WasmType::Func(param_types, result_types));

        let func_idx = self.module.functions.len() as u32;
        self.module.functions.push(self.module.types.len() as u32 - 1);

        let mut codegen = WasmFunctionCodegen::new(func.locals_count, func.param_count);
        codegen.compile_body(&func.body)?;

        self.module.code_section.push(codegen.into_body());
        self.module.function_names.push((func_idx, func.name.clone()));

        if func.is_main {
            self.module.export_section.push(WasmExport {
                name: func.name.clone(),
                kind: WasmExportKind::Func,
                index: func_idx,
            });
        }

        Ok(())
    }

    fn emit_module(&self) -> Vec<u8> {
        let mut wasm = Vec::new();

        wasm.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]);
        wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);

        self.emit_type_section(&mut wasm);
        self.emit_function_section(&mut wasm);
        self.emit_memory_section(&mut wasm);
        self.emit_export_section(&mut wasm);
        self.emit_code_section(&mut wasm);
        self.emit_name_section(&mut wasm);

        wasm
    }

    fn emit_type_section(&self, wasm: &mut Vec<u8>) {
        if self.module.types.is_empty() {
            return;
        }

        wasm.push(0x01);

        let mut type_data = Vec::new();
        encode_u32(&mut type_data, self.module.types.len() as u32);

        for ty in &self.module.types {
            match ty {
                WasmType::Func(params, results) => {
                    type_data.push(0x60);
                    encode_u32(&mut type_data, params.len() as u32);
                    for p in params {
                        type_data.push(p.type_byte());
                    }
                    encode_u32(&mut type_data, results.len() as u32);
                    for r in results {
                        type_data.push(r.type_byte());
                    }
                }
            }
        }

        encode_u32(wasm, type_data.len() as u32);
        wasm.extend_from_slice(&type_data);
    }

    fn emit_function_section(&self, wasm: &mut Vec<u8>) {
        if self.module.functions.is_empty() {
            return;
        }

        wasm.push(0x03);

        let mut func_data = Vec::new();
        encode_u32(&mut func_data, self.module.functions.len() as u32);
        for &type_idx in &self.module.functions {
            encode_u32(&mut func_data, type_idx);
        }

        encode_u32(wasm, func_data.len() as u32);
        wasm.extend_from_slice(&func_data);
    }

    fn emit_memory_section(&self, wasm: &mut Vec<u8>) {
        if self.module.memories.is_empty() {
            return;
        }

        wasm.push(0x05);

        let mut mem_data = Vec::new();
        encode_u32(&mut mem_data, self.module.memories.len() as u32);
        for mem in &self.module.memories {
            if let Some(max) = mem.max_pages {
                mem_data.push(0x01);
                encode_u32(&mut mem_data, mem.min_pages);
                encode_u32(&mut mem_data, max);
            } else {
                mem_data.push(0x00);
                encode_u32(&mut mem_data, mem.min_pages);
            }
        }

        encode_u32(wasm, mem_data.len() as u32);
        wasm.extend_from_slice(&mem_data);
    }

    fn emit_export_section(&self, wasm: &mut Vec<u8>) {
        if self.module.export_section.is_empty() {
            return;
        }

        wasm.push(0x07);

        let mut export_data = Vec::new();
        encode_u32(&mut export_data, self.module.export_section.len() as u32);
        for exp in &self.module.export_section {
            let name_bytes = exp.name.as_bytes();
            encode_u32(&mut export_data, name_bytes.len() as u32);
            export_data.extend_from_slice(name_bytes);
            export_data.push(exp.kind as u8);
            encode_u32(&mut export_data, exp.index);
        }

        encode_u32(wasm, export_data.len() as u32);
        wasm.extend_from_slice(&export_data);
    }

    fn emit_code_section(&self, wasm: &mut Vec<u8>) {
        if self.module.code_section.is_empty() {
            return;
        }

        wasm.push(0x0A);

        let mut code_data = Vec::new();
        encode_u32(&mut code_data, self.module.code_section.len() as u32);
        for body in &self.module.code_section {
            let mut func_data = Vec::new();

            encode_u32(&mut func_data, body.locals.len() as u32);
            for &(count, val_type) in &body.locals {
                encode_u32(&mut func_data, count);
                func_data.push(val_type.type_byte());
            }

            func_data.extend_from_slice(&body.code);

            encode_u32(&mut code_data, func_data.len() as u32);
            code_data.extend_from_slice(&func_data);
        }

        encode_u32(wasm, code_data.len() as u32);
        wasm.extend_from_slice(&code_data);
    }

    fn emit_name_section(&self, wasm: &mut Vec<u8>) {
        if self.module.function_names.is_empty() {
            return;
        }

        wasm.push(0x00);

        let mut name_data = Vec::new();

        name_data.push(0x01);
        encode_u32(&mut name_data, self.module.function_names.len() as u32);
        for &(idx, ref name) in &self.module.function_names {
            encode_u32(&mut name_data, idx);
            let name_bytes = name.as_bytes();
            encode_u32(&mut name_data, name_bytes.len() as u32);
            name_data.extend_from_slice(name_bytes);
        }

        encode_u32(wasm, name_data.len() as u32);
        wasm.extend_from_slice(&name_data);
    }
}

impl Default for WasmBackend {
    fn default() -> Self {
        Self::new()
    }
}

struct WasmFunctionCodegen {
    code: Vec<u8>,
    locals_count: usize,
    param_count: usize,
    local_types: Vec<WasmValType>,
}

impl WasmFunctionCodegen {
    fn new(locals_count: usize, param_count: usize) -> Self {
        WasmFunctionCodegen {
            code: Vec::new(),
            locals_count,
            param_count,
            local_types: vec![WasmValType::I32; locals_count],
        }
    }

    fn compile_body(&mut self, body: &[HIRStmt]) -> Result<(), String> {
        for stmt in body {
            self.compile_stmt(stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &HIRStmt) -> Result<(), String> {
        match stmt {
            HIRStmt::Let(idx, expr) => {
                self.compile_expr(expr)?;
                self.code.push(0x21);
                encode_u32(&mut self.code, *idx as u32);
            }
            HIRStmt::Assign(idx, expr) => {
                self.compile_expr(expr)?;
                self.code.push(0x22);
                encode_u32(&mut self.code, *idx as u32);
            }
            HIRStmt::Expr(expr) => {
                self.compile_expr(expr)?;
                self.code.push(0x1A);
            }
            HIRStmt::Return(Some(expr)) => {
                self.compile_expr(expr)?;
                self.code.push(0x0F);
            }
            HIRStmt::Return(None) => {
                self.code.push(0x0F);
            }
            HIRStmt::Print(expr) => {
                self.compile_expr(expr)?;
                self.code.push(0x1A);
            }
            HIRStmt::While { condition, body } => {
                let loop_start = self.code.len();
                self.compile_expr(condition)?;
                let br_if_pos = self.code.len();
                self.code.push(0x0D);
                encode_u32(&mut self.code, 0);

                for stmt in body {
                    self.compile_stmt(stmt)?;
                }
                self.code.push(0x0C);
                encode_u32(&mut self.code, 0);

                let loop_end = self.code.len();
                let offset = (loop_end - br_if_pos - 5) as i32;
                self.code[br_if_pos + 1..br_if_pos + 5].copy_from_slice(&offset.to_le_bytes());
            }
            HIRStmt::If { condition, then_body, else_body } => {
                self.compile_expr(condition)?;
                let br_if_pos = self.code.len();
                self.code.push(0x0D);
                encode_u32(&mut self.code, 0);

                for stmt in then_body {
                    self.compile_stmt(stmt)?;
                }

                if let Some(else_body) = else_body {
                    self.code.push(0x05);
                    let else_start = self.code.len();
                    let if_offset = (else_start - br_if_pos - 5) as i32;
                    self.code[br_if_pos + 1..br_if_pos + 5].copy_from_slice(&if_offset.to_le_bytes());

                    for stmt in else_body {
                        self.compile_stmt(stmt)?;
                    }

                    let end_pos = self.code.len();
                    let else_offset = (end_pos - else_start) as i32;
                    let else_br_pos = else_start - 5;
                    self.code[else_br_pos + 1..else_br_pos + 5].copy_from_slice(&(-else_offset).to_le_bytes());
                } else {
                    let end_pos = self.code.len();
                    let offset = (end_pos - br_if_pos - 5) as i32;
                    self.code[br_if_pos + 1..br_if_pos + 5].copy_from_slice(&offset.to_le_bytes());
                }
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &HIRExpr) -> Result<(), String> {
        match expr {
            HIRExpr::Integer(n) => {
                self.code.push(0x41);
                encode_i32(&mut self.code, *n as i32);
            }
            HIRExpr::Float(n) => {
                self.code.push(0x44);
                self.code.extend_from_slice(&n.to_le_bytes());
            }
            HIRExpr::String(_) => {
                self.code.push(0x41);
                encode_u32(&mut self.code, 0);
            }
            HIRExpr::Bool(b) => {
                self.code.push(0x41);
                encode_u32(&mut self.code, if *b { 1 } else { 0 });
            }
            HIRExpr::Unit => {
                self.code.push(0x41);
                encode_u32(&mut self.code, 0);
            }
            HIRExpr::LoadLocal(idx) => {
                self.code.push(0x20);
                encode_u32(&mut self.code, *idx as u32);
            }
            HIRExpr::BinaryOp { op, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                self.compile_binop(op)?;
            }
            HIRExpr::UnaryOp { op, expr } => {
                self.compile_expr(expr)?;
                self.compile_unaryop(op)?;
            }
            HIRExpr::Print(expr) => {
                self.compile_expr(expr)?;
            }
            HIRExpr::Return(expr) => {
                self.compile_expr(expr)?;
            }
            HIRExpr::MakeStruct { fields } => {
                for field in fields {
                    self.compile_expr(field)?;
                }
            }
            HIRExpr::GetField { object, index } => {
                self.compile_expr(object)?;
                self.code.push(0x41);
                encode_u32(&mut self.code, *index as u32);
            }
            HIRExpr::MakeArray { elements } => {
                for elem in elements {
                    self.compile_expr(elem)?;
                }
            }
            HIRExpr::ArrayGet { array, index } => {
                self.compile_expr(array)?;
                self.compile_expr(index)?;
            }
            HIRExpr::Call { func, args } => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.code.push(0x10);
                encode_u32(&mut self.code, 0);
            }
            HIRExpr::CrossDomainCall { func, args, .. } => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.code.push(0x10);
                encode_u32(&mut self.code, 0);
            }
            HIRExpr::StoreLocal(_, stmt) => {
                self.compile_stmt(stmt)?;
            }
            HIRExpr::If { condition, then_body, else_body } => {
                self.compile_expr(condition)?;
                let br_if_pos = self.code.len();
                self.code.push(0x0D);
                encode_u32(&mut self.code, 0);

                for stmt in then_body {
                    self.compile_stmt(stmt)?;
                }

                if let Some(else_body) = else_body {
                    self.code.push(0x05);
                    let else_start = self.code.len();
                    let if_offset = (else_start - br_if_pos - 5) as i32;
                    self.code[br_if_pos + 1..br_if_pos + 5].copy_from_slice(&if_offset.to_le_bytes());

                    for stmt in else_body {
                        self.compile_stmt(stmt)?;
                    }

                    let end_pos = self.code.len();
                    let else_offset = (end_pos - else_start) as i32;
                    let else_br_pos = else_start - 5;
                    self.code[else_br_pos + 1..else_br_pos + 5].copy_from_slice(&(-else_offset).to_le_bytes());
                } else {
                    let end_pos = self.code.len();
                    let offset = (end_pos - br_if_pos - 5) as i32;
                    self.code[br_if_pos + 1..br_if_pos + 5].copy_from_slice(&offset.to_le_bytes());
                }
            }
            HIRExpr::While { condition, body } => {
                let loop_start = self.code.len();
                self.compile_expr(condition)?;
                let br_if_pos = self.code.len();
                self.code.push(0x0D);
                encode_u32(&mut self.code, 0);

                for stmt in body {
                    self.compile_stmt(stmt)?;
                }
                self.code.push(0x0C);
                encode_u32(&mut self.code, 0);

                let loop_end = self.code.len();
                let offset = (loop_end - br_if_pos - 5) as i32;
                self.code[br_if_pos + 1..br_if_pos + 5].copy_from_slice(&offset.to_le_bytes());
            }
        }
        Ok(())
    }

    fn compile_binop(&mut self, op: &HIRBinOp) -> Result<(), String> {
        match op {
            HIRBinOp::Add => { self.code.push(0x6A); Ok(()) }
            HIRBinOp::Sub => { self.code.push(0x6B); Ok(()) }
            HIRBinOp::Mul => { self.code.push(0x6C); Ok(()) }
            HIRBinOp::Div => { self.code.push(0x6D); Ok(()) }
            HIRBinOp::Mod => { self.code.push(0x6F); Ok(()) }
            HIRBinOp::Eq => { self.code.push(0x46); Ok(()) }
            HIRBinOp::Neq => { self.code.push(0x47); Ok(()) }
            HIRBinOp::Lt => { self.code.push(0x48); Ok(()) }
            HIRBinOp::Gt => { self.code.push(0x4A); Ok(()) }
            HIRBinOp::Lte => { self.code.push(0x4C); Ok(()) }
            HIRBinOp::Gte => { self.code.push(0x4E); Ok(()) }
            HIRBinOp::And => { self.code.push(0x71); Ok(()) }
            HIRBinOp::Or => { self.code.push(0x72); Ok(()) }
        }
    }

    fn compile_unaryop(&mut self, op: &HIRUnaryOp) -> Result<(), String> {
        match op {
            HIRUnaryOp::Neg => {
                self.code.push(0x41);
                encode_u32(&mut self.code, 0);
                self.code.push(0x6B);
                Ok(())
            }
            HIRUnaryOp::Not => {
                self.code.push(0x41);
                encode_u32(&mut self.code, 0);
                self.code.push(0x47);
                Ok(())
            }
        }
    }

    fn into_body(self) -> WasmFunctionBody {
        WasmFunctionBody {
            locals: vec![(self.locals_count as u32, WasmValType::I32)],
            code: self.code,
        }
    }
}

fn encode_u32(buf: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn encode_i32(buf: &mut Vec<u8>, value: i32) {
    let mut more = true;
    let mut val = value as i64;
    while more {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if (val == 0 && (byte & 0x40) == 0) || (val == -1 && (byte & 0x40) != 0) {
            more = false;
        } else {
            byte |= 0x80;
        }
        buf.push(byte);
    }
}

pub fn compile_to_wasm(program: &HIRProgram, output_path: &str) -> Result<(), String> {
    let mut backend = WasmBackend::new();
    let wasm_bytes = backend.compile(program)?;

    std::fs::write(output_path, &wasm_bytes)
        .map_err(|e| format!("failed to write WASM file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_u32() {
        let mut buf = Vec::new();
        encode_u32(&mut buf, 0);
        assert_eq!(buf, vec![0x00]);

        buf.clear();
        encode_u32(&mut buf, 127);
        assert_eq!(buf, vec![0x7F]);

        buf.clear();
        encode_u32(&mut buf, 128);
        assert_eq!(buf, vec![0x80, 0x01]);
    }

    #[test]
    fn test_encode_i32() {
        let mut buf = Vec::new();
        encode_i32(&mut buf, 0);
        assert_eq!(buf, vec![0x00]);

        buf.clear();
        encode_i32(&mut buf, -1);
        assert_eq!(buf, vec![0x7F]);

        buf.clear();
        encode_i32(&mut buf, 128);
        assert_eq!(buf, vec![0x80, 0x01]);
    }

    #[test]
    fn test_wasm_backend_creation() {
        let backend = WasmBackend::new();
        assert!(backend.module.types.is_empty());
    }

    #[test]
    fn test_empty_program() {
        let mut backend = WasmBackend::new();
        let program = HIRProgram { functions: vec![] };
        let result = backend.compile(&program);
        assert!(result.is_ok());
        let wasm = result.unwrap();
        assert!(wasm.starts_with(&[0x00, 0x61, 0x73, 0x6D]));
    }
}
