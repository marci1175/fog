; ModuleID = 'main'
source_filename = "main"

@"asd %f" = constant [6 x i8] c"asd %f"

declare void @printf(ptr, ...)

define i32 @main() {
main_fn_entry:
  call void (ptr, ...) @printf(ptr @"asd %f", double 2.340000e+01)
  ret i32 0
}
