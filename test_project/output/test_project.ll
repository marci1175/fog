; ModuleID = 'main'
source_filename = "main"

declare i32 @rand()

declare i32 @gets()

declare i32 @srand(i32)

declare i32 @printf(ptr, i32)

declare i32 @time(i32)

define i32 @main() {
main_fn_entry:
  %seed = alloca i32, align 4
  %since = alloca i32, align 4
  store i32 0, ptr %since, align 4
  %since1 = load i32, ptr %since, align 4
  %function_call = call i32 @time(i32 %since1)
  store i32 %function_call, ptr %seed, align 4
  %seed2 = load i32, ptr %seed, align 4
  store i32 %seed2, ptr %seed, align 4
  %seed3 = load i32, ptr %seed, align 4
  %function_call4 = call i32 @srand(i32 %seed3)
  %0 = alloca i32, align 4
  store i32 %function_call4, ptr %0, align 4
  br label %loop_body

loop_body:                                        ; preds = %loop_body, %main_fn_entry
  %str = alloca ptr, align 8
  %string_buffer = alloca [19 x i8], align 1
  store [19 x i8] c"Random number: %i\0A\00", ptr %string_buffer, align 1
  store ptr %string_buffer, ptr %str, align 8
  %str5 = load ptr, ptr %str, align 8
  %inp = alloca i32, align 4
  %function_call6 = call i32 @rand()
  store i32 %function_call6, ptr %inp, align 4
  %inp7 = load i32, ptr %inp, align 4
  store i32 %inp7, ptr %inp, align 4
  %inp8 = load i32, ptr %inp, align 4
  %function_call9 = call i32 @printf(ptr %str5, i32 %inp8)
  %1 = alloca i32, align 4
  store i32 %function_call9, ptr %1, align 4
  br label %loop_body
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var10 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var10
}
