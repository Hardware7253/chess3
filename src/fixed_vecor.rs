// Similiar behaviour to a vector, but the underlying data type is a normal fixed sized array
// This is more friendly for embedded hardware

#[derive(Debug, PartialEq, Clone)]
pub struct FixedVector<T: Copy, const L: usize> {
    pub internal_array: [T; L],
    pub length: usize,
}

impl<T: Copy, const L: usize> FixedVector<T, L> {
    pub fn new(placeholder_value: T) -> Self {
        FixedVector {
            internal_array: [placeholder_value; L],
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn push(&mut self, data: T) {
        self.internal_array[self.length] = data;
        self.length += 1
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length > 0 {
            self.length -= 1;
            return Some(self.internal_array[self.length - 1]);
        }

        None
    }
}