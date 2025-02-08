#[derive(Debug)]
pub struct ASM {
    pub section_data: Vec<String>,
    pub section_bss: Vec<String>,
    pub section_text: Vec<String>,
    pub sections_code: Vec<SectionCode>,
}

impl ASM {
    pub fn new() -> Self {
        ASM {
            section_data: Vec::new(),
            section_bss: Vec::new(),
            section_text: Vec::new(),
            sections_code: Vec::new(),
        }
    }

    pub fn join(&self, separator: &str) -> String {
        let mut asm_code = Vec::new();
        asm_code.extend(self.section_data.iter());
        let empty_string = "".to_string();
        asm_code.push(&empty_string);
        asm_code.extend(self.section_bss.iter());
        asm_code.push(&empty_string);
        asm_code.extend(self.section_text.iter());
        asm_code.push(&empty_string);
        for section in &self.sections_code {
            asm_code.push(&section.name);
            asm_code.extend(section.code.iter());
            asm_code.push(&empty_string);
        }
        asm_code
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join(separator)
    }
}

#[derive(Debug)]
pub struct SectionCode {
    pub name: String,
    pub code: Vec<String>,
}

impl SectionCode {
    pub fn new(name: String) -> Self {
        SectionCode {
            name,
            code: Vec::new(),
        }
    }
}
