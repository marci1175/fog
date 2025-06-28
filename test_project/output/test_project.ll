; ModuleID = 'main'
source_filename = "main"

@"Krisztian eletkora: %i\0A" = constant [23 x i8] c"Krisztian eletkora: %i\0A"

declare i32 @printf(ptr, i32)

define i32 @main() {
main_fn_entry:
  %a = alloca i32, align 4
  store i32 0, ptr %a, align 4
  %0 = alloca i32, align 4
  store i32 23, ptr %0, align 4
  %1 = alloca i32, align 4
  store i32 2, ptr %1, align 4
  %2 = alloca ptr, align 8
  %str = alloca ptr, align 8
  %3 = alloca i32, align 4
  store i32 2, ptr %3, align 4
  %res = alloca i32, align 4
  %4 = alloca i32, align 4
  br label %loop_body

loop_body:                                        ; preds = %loop_body, %main_fn_entry
  %lhs = load i32, ptr %0, align 4
  %rhs = load i32, ptr %1, align 4
  %int_sub_int = sub i32 %lhs, %rhs
  store i32 %int_sub_int, ptr %a, align 4
  store ptr @"Krisztian eletkora: %i\0A", ptr %2, align 8
  %str1 = load ptr, ptr %2, align 8
  %lhs2 = load i32, ptr %a, align 4
  %rhs3 = load i32, ptr %3, align 4
  %int_mul_int = mul i32 %lhs2, %rhs3
  store i32 %int_mul_int, ptr %res, align 4
  %res4 = load i32, ptr %res, align 4
  %function_call = call i32 @printf(ptr %str1, i32 %res4)
  store i32 %function_call, ptr %4, align 4
  br label %loop_body
}
