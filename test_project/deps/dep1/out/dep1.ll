; ModuleID = 'dep1'
source_filename = "dep1"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

@Alma = addrspace(8) constant [5 x i8] c"Alma\00"

declare i32 @printf(ptr addrspace(8), ...)

define void @kedvenc() !dbg !3 {
main_fn_entry:
  %input = alloca ptr addrspace(8), align 8
  store ptr addrspace(8) @Alma, ptr %input, align 8
  %input1 = load ptr addrspace(8), ptr %input, align 8
  %function_call = call i32 (ptr addrspace(8), ...) @printf(ptr addrspace(8) %input1)
  %0 = alloca i32, align 4
  store i32 %function_call, ptr %0, align 4
  ret void
}

!llvm.dbg.cu = !{!0}
!llvm.debug.version = !{!2}

!0 = distinct !DICompileUnit(language: DW_LANG_C, file: !1, producer: "Fog (ver.: 0.1.0) with LLVM 18-1-8", isOptimized: false, runtimeVersion: 1, emissionKind: FullDebug, splitDebugInlining: false, debugInfoForProfiling: true)
!1 = !DIFile(filename: "dep1", directory: "C:\\Users\\marci\\Desktop\\fog\\test_project\\deps\\dep1\\deps\\src")
!2 = !{i32 1}
!3 = distinct !DISubprogram(name: "kedvenc", linkageName: "kedvenc", scope: !1, file: !1, line: 69, type: !4, scopeLine: 69, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!4 = !DISubroutineType(types: !5)
!5 = !{null}
