use std::iter;

pub struct Rewinding {
    pub save_states: [Option<Vec<u8>>; 10],
    pub before_last_ss_load: Option<Vec<u8>>,
    pub rewind_buffer: RWBuffer,
    pub rewinding: bool,
}

impl Rewinding {
    pub fn set_rw_buf_size(&mut self, secs: usize) {
        self.rewind_buffer = RWBuffer::new(secs);
    }
}

impl Default for Rewinding {
    fn default() -> Self {
        Self {
            save_states: [None, None, None, None, None, None, None, None, None, None],
            before_last_ss_load: None,
            rewind_buffer: RWBuffer::new(60 * 10),
            rewinding: false,
        }
    }
}

pub struct RWBuffer {
    vec: Vec<Vec<u8>>,
    cur_idx: usize,
    stop_idx: usize,
}

impl RWBuffer {
    pub fn pop(&mut self) -> Option<&[u8]> {
        if self.cur_idx == self.stop_idx {
            return None;
        }
        if self.cur_idx == 0 {
            self.cur_idx = self.vec.len() - 1;
        } else {
            self.cur_idx -= 1;
        };
        Some(&self.vec[self.cur_idx])
    }

    pub fn push(&mut self, val: Vec<u8>) {
        if self.cur_idx == self.vec.len() - 1 {
            self.cur_idx = 0;
        } else {
            self.cur_idx += 1;
        }
        self.stop_idx = self.cur_idx + 1;
        self.vec[self.cur_idx] = val;
    }

    fn new(secs: usize) -> Self {
        Self {
            vec: iter::repeat(vec![]).take(60 * secs).collect(),
            cur_idx: 0,
            stop_idx: 0,
        }
    }
}
