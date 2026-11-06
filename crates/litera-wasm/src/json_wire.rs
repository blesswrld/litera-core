//! Zero-dependency JSON wire serializer (`JsonWireWriter`) for WebAssembly host communication.
//!
//! Avoids pulling `serde` and `serde_json` procedural macros into the WASM binary,
//! providing deterministic RFC 8259 JSON formatting with NaN/Inf safety guards.

pub struct JsonWireWriter {
    buf: String,
    needs_comma: Vec<bool>,
}

impl JsonWireWriter {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: String::with_capacity(cap),
            needs_comma: Vec::with_capacity(8),
        }
    }

    #[inline]
    fn write_sep(&mut self) {
        if let Some(last) = self.needs_comma.last_mut() {
            if *last {
                self.buf.push(',');
            } else {
                *last = true;
            }
        }
    }

    pub fn begin_object(&mut self) {
        self.write_sep();
        self.buf.push('{');
        self.needs_comma.push(false);
    }

    pub fn end_object(&mut self) {
        self.needs_comma.pop();
        self.buf.push('}');
    }

    pub fn begin_array(&mut self, key: &str) {
        self.write_sep();
        self.write_escaped_str(key);
        self.buf.push_str(":[");
        self.needs_comma.push(false);
    }

    pub fn end_array(&mut self) {
        self.needs_comma.pop();
        self.buf.push(']');
    }

    pub fn field_str(&mut self, key: &str, val: &str) {
        self.write_sep();
        self.write_escaped_str(key);
        self.buf.push(':');
        self.write_escaped_str(val);
    }

    pub fn field_u64(&mut self, key: &str, val: u64) {
        self.write_sep();
        self.write_escaped_str(key);
        self.buf.push(':');
        self.buf.push_str(&val.to_string());
    }

    pub fn field_i64(&mut self, key: &str, val: i64) {
        self.write_sep();
        self.write_escaped_str(key);
        self.buf.push(':');
        self.buf.push_str(&val.to_string());
    }

    pub fn field_f32(&mut self, key: &str, val: f32) {
        self.write_sep();
        self.write_escaped_str(key);
        self.buf.push(':');
        let safe = if val.is_finite() { val } else { 0.0 };
        self.buf.push_str(&format!("{safe:.4}"));
    }

    fn write_escaped_str(&mut self, s: &str) {
        self.buf.push('"');
        for ch in s.chars() {
            match ch {
                '"' => self.buf.push_str("\\\""),
                '\\' => self.buf.push_str("\\\\"),
                '\n' => self.buf.push_str("\\n"),
                '\r' => self.buf.push_str("\\r"),
                '\t' => self.buf.push_str("\\t"),
                c if c.is_control() => {
                    self.buf.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => self.buf.push(c),
            }
        }
        self.buf.push('"');
    }

    pub fn into_string(self) -> String {
        self.buf
    }
}
