; ModuleID = 'main'
source_filename = "main"

@szamok.1 = constant [7 x i8] c"szamok\00"
@"Number %s" = constant [10 x i8] c"Number %s\00"

declare i32 @printf(ptr, ...)

define ptr @szamok() !dbg !2 {
main_fn_entry:
  %ret_tmp_var = alloca ptr, align 8
  store ptr @szamok.1, ptr %ret_tmp_var, align 8
  %ret_tmp_var1 = load ptr, ptr %ret_tmp_var, align 8
  ret ptr %ret_tmp_var1
}

define i32 @main() !dbg !6 {
main_fn_entry:
  %szamok = alloca i32, align 4
  store i32 2, ptr %szamok, align 4
  %input = alloca ptr, align 8
  store ptr @"Number %s", ptr %input, align 8
  %input1 = load ptr, ptr %input, align 8
  %printf_idx_1_arg = alloca ptr, align 8
  %function_call = call ptr @szamok()
  store ptr %function_call, ptr %printf_idx_1_arg, align 8
  %printf_idx_1_arg2 = load ptr, ptr %printf_idx_1_arg, align 8
  store ptr %printf_idx_1_arg2, ptr %printf_idx_1_arg, align 8
  %printf_idx_1_arg3 = load ptr, ptr %printf_idx_1_arg, align 8
  %function_call4 = call i32 (ptr, ...) @printf(ptr %input1, ptr %printf_idx_1_arg3)
  %0 = alloca i32, align 4
  store i32 %function_call4, ptr %0, align 4
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var5 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var5
}

!llvm.dbg.cu = !{!0}

!0 = distinct !DICompileUnit(language: DW_LANG_C, file: !1, producer: "Fog (ver.: 0.1.0) with LLVM 18-1-8", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, debugInfoForProfiling: true)
!1 = !DIFile(filename: "main", directory: "src/")
!2 = distinct !DISubprogram(name: "szamok", linkageName: "szamok", scope: !1, file: !1, line: 69, type: !3, scopeLine: 69, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!3 = !DISubroutineType(types: !4)
!4 = !{!5}
!5 = !DIBasicType(name: "String", size: 24, encoding: DW_ATE_ASCII)
!6 = distinct !DISubprogram(name: "main", linkageName: "main", scope: !1, file: !1, line: 69, type: !7, scopeLine: 69, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!7 = !DISubroutineType(types: !8)
!8 = !{!9}
!9 = !DIBasicType(name: "I32", size: 4, encoding: DW_ATE_signed)
