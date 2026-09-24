; ModuleID = 'test_project'
source_filename = "test_project"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

define i32 @"test_project::main"() {
main:
}

define i32 @"test_project::helper::helper_function1"(i32 %0) {
main:
}

define i32 @"test_project::helper::helper_function2"(i32 %0) {
main:
}

!llvm.dbg.cu = !{!0}
!llvm.debug.version = !{!2}

!0 = distinct !DICompileUnit(language: DW_LANG_C, file: !1, producer: "Fog (ver.: 0.1.0) with LLVM 23.1.0", isOptimized: false, runtimeVersion: 1, emissionKind: FullDebug, splitDebugInlining: false, debugInfoForProfiling: true)
!1 = !DIFile(filename: "test_project", directory: "<UNUSED>")
!2 = !{i32 1}
