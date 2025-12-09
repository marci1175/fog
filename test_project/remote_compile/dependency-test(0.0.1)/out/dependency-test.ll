; ModuleID = 'main'
source_filename = "main"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

@"Szia!" = constant [6 x i8] c"Szia!\00"

declare i32 @printf(ptr, ...)

define i32 @szia() {
main_fn_entry:
  %function_call = call i32 (ptr, ...) @printf(ptr @"Szia!")
  ret i32 0
}

!llvm.dbg.cu = !{!0}
!llvm.debug.version = !{!2}

!0 = distinct !DICompileUnit(language: DW_LANG_C, file: !1, producer: "Fog (ver.: 0.1.0) with LLVM 21.1.2", isOptimized: true, runtimeVersion: 1, emissionKind: LineTablesOnly, splitDebugInlining: false)
!1 = !DIFile(filename: "main", directory: "\\\\?\\C:\\Users\\marci\\Desktop\\fog\\deps\\dependency-test(0.0.1)\\src")
!2 = !{i32 1}
