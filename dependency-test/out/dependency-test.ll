; ModuleID = 'main'
source_filename = "main"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

@"Szia!" = constant [6 x i8] c"Szia!\00"

declare i32 @printf(ptr, ...)

define i32 @szia() !dbg !3 {
main_fn_entry:
  %input = alloca ptr, align 8
  store ptr @"Szia!", ptr %input, align 8
  %input1 = load ptr, ptr %input, align 8
  %function_call = call i32 (ptr, ...) @printf(ptr %input1)
  %0 = alloca i32, align 4
  store i32 %function_call, ptr %0, align 4
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var2 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var2
}

!llvm.dbg.cu = !{!0}
!llvm.debug.version = !{!2}

!0 = distinct !DICompileUnit(language: DW_LANG_C, file: !1, producer: "Fog (ver.: 0.1.0) with LLVM 21.1.2", isOptimized: false, runtimeVersion: 1, emissionKind: FullDebug, splitDebugInlining: false, debugInfoForProfiling: true)
!1 = !DIFile(filename: "main", directory: "C:\\Users\\marci\\Desktop\\fog\\dependency-test\\src")
!2 = !{i32 1}
!3 = distinct !DISubprogram(name: "szia", linkageName: "szia", scope: !1, file: !1, line: 69, type: !4, scopeLine: 69, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!4 = !DISubroutineType(types: !5)
!5 = !{!6}
!6 = !DIBasicType(name: "I32", size: 4, encoding: DW_ATE_signed)
