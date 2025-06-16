; ModuleID = 'main'
source_filename = "main"

declare i32 @srand(i32)

declare i32 @time(i32)

declare i32 @gets()

declare i32 @rand()

declare i32 @printf(ptr, i32)

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
  br label %loop_body
}
