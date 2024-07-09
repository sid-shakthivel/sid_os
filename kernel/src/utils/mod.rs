pub mod bitwise;
pub mod grub;
pub mod multiboot2;
pub mod ports;
pub mod spinlock;
pub mod string;
pub mod wrapping_zero;

#[macro_export]
macro_rules! either {
    ($test:expr => $true_expr:expr; $false_expr:expr) => {
        if $test {
            $true_expr
        } else {
            $false_expr
        }
    };
}
