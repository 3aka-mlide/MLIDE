use capstone::prelude::*;

pub struct Disassembler {
    cs: Capstone,
}

impl Disassembler {
    pub fn new() -> Self {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(false)
            .build()
            .expect("Failed to initialize Capstone");

        Self { cs }
    }

    pub fn disassemble_bytes(&self, bytes: &[u8], base_address: u64) -> String {
        let mut output = String::new();

        match self.cs.disasm_all(bytes, base_address) {
            Ok(insns) => {
                for i in insns.as_ref() {
                    let line = format!(
                        "0x{:08X}:  {: <8} {}\n",
                        i.address(),
                        i.mnemonic().unwrap_or(""),
                        i.op_str().unwrap_or("")
                    );
                    output.push_str(&line);
                }
            }
            Err(e) => {
                output = format!("; Error disassembling: {}", e);
            }
        }

        if output.is_empty() {
            output = "; No instructions found or invalid executable data.".to_string();
        }

        output
    }
}

impl Default for Disassembler {
    fn default() -> Self {
        Self::new()
    }
}