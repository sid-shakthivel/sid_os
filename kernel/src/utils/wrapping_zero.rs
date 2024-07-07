pub trait WrappingSubZero {
    fn wrapping_sub_zero(self, rhs: Self) -> Self;
}

impl WrappingSubZero for u8 {
    fn wrapping_sub_zero(self, rhs: Self) -> Self {
        if self < rhs {
            0
        } else {
            self.wrapping_sub(rhs)
        }
    }
}

impl WrappingSubZero for u16 {
    fn wrapping_sub_zero(self, rhs: Self) -> Self {
        if self < rhs {
            0
        } else {
            self.wrapping_sub(rhs)
        }
    }
}

impl WrappingSubZero for u32 {
    fn wrapping_sub_zero(self, rhs: Self) -> Self {
        if self < rhs {
            0
        } else {
            self.wrapping_sub(rhs)
        }
    }
}

impl WrappingSubZero for u64 {
    fn wrapping_sub_zero(self, rhs: Self) -> Self {
        if self < rhs {
            0
        } else {
            self.wrapping_sub(rhs)
        }
    }
}

impl WrappingSubZero for usize {
    fn wrapping_sub_zero(self, rhs: Self) -> Self {
        if self < rhs {
            0
        } else {
            self.wrapping_sub(rhs)
        }
    }
}
