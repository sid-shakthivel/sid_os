; /src/cpu.asm

global inb_raw
inb_raw:
  mov dx, di ; Address, first parameter
  in al, dx
  ret

global outb_raw
outb_raw:
  mov dx, di ; Address, first parameter
  mov al, sil ; Value, second parameter
  out dx, al
  ret

global outpw_raw
outpw_raw:
  mov dx, di ; Address (16 Bit) 
  mov ax, si ; Value (16 Bit)
  out dx, ax
  ret

global inpw_raw
inpw_raw:
  mov dx, di ; Address (16 Bit) 
  in ax, dx
  ret

; Load IDT
global flush_idt    
flush_idt:
  cli ; Disable interrupts
  extern IDTR
  lidt [IDTR]
  ret

%macro pushaq 0
push rax
push rbx
push rcx
push rdx
push rsi
push rdi
%endmacro

%macro popaq 0
pop rdi
pop rsi
pop rdx
pop rcx
pop rbx
pop rax
%endmacro

%macro handle_no_err_exception 1
global handle_no_err_exception%1
handle_no_err_exception%1:
    xchg bx, bx
    push qword 0 ; Dummy error code
    push qword %1 ; Number
    pushaq ; Push registers
    cld
    mov rax, 0xDEADBEEF
    popaq
    add rsp, 0x10 ; Must remove both 64 bit values pushed onto stack
    iretq ; Exit from interrupt
%endmacro

handle_no_err_exception 0

; global flush_tlb
; flush_tlb:
;   push rax
;   mov rax, cr3
;   mov cr3, rax
;   pop rax
;   ret

; global speedy_write
; speedy_write:
;   xchg bx, bx
;   ret