; ModuleID = 'main'
source_filename = "main"

@"Int: %i\0A" = constant [8 x i8] c"Int: %i\0A"

declare i32 @printf(ptr, i32)

define i32 @main() {
main_fn_entry:
  %int_sub_int = sub i32 23, 2
  %int_add_int = add i32 %int_sub_int, 1
  %int_mul_int = mul i32 %int_add_int, 3
  %int_add_int9 = add i32 %int_mul_int, 2
  %int_sub_int13 = sub i32 %int_add_int9, 435
  %int_add_int17 = add i32 %int_sub_int13, 5353
  %int_mul_int21 = mul i32 %int_add_int17, 2
  %int_mul_int25 = mul i32 %int_mul_int21, 4
  %int_div_int = sdiv i32 %int_mul_int25, 9
  br label %loop_body

loop_body:                                        ; preds = %loop_body, %main_fn_entry
  %function_call = call i32 @printf(ptr @"Int: %i\0A", i32 %int_div_int)
  br label %loop_body
}
