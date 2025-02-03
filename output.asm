section .data
print_format_str: db "%s\n", 0
print_format_int: db "%d\n", 0
print_format_float: db "%f\n", 0
_var_num1: dq 1
_var_num2: dq 1
_var_result: dq 0

section .text
extern printf
extern exit
global _start
_start:
add:
push rbp
mov rbp, rsp
mov rax, [rel _var_a]
push rax
mov rax, [rel _var_b]
pop rbx
add rax, rbx
jmp _exit
mov rsp, rbp
pop rbp
ret
mov rax, [rel _var_result]
lea rdi, [rel print_format_int]
mov rsi, rax
xor rax, rax
call printf
mov rdi, 0
call exit