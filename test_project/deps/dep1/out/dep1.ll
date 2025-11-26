; ModuleID = 'dep1'
source_filename = "dep1"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

%Alma = type { i32, i32 }

@Alma = constant [5 x i8] c"Alma\00"
@"%i" = constant [3 x i8] c"%i\00"

declare i32 @printf(ptr, ...)

declare void @open_window(ptr)

declare { i32, i32 } @alma_csinalo()

declare void @hi_from_cpp()

define void @kedvenc() !dbg !3 {
main_fn_entry:
  %input = alloca ptr, align 8
  store ptr @Alma, ptr %input, align 8
  %input1 = load ptr, ptr %input, align 8
  %function_call = call i32 (ptr, ...) @printf(ptr %input1)
  %0 = alloca i32, align 4
  store i32 %function_call, ptr %0, align 4
  ret void
}

define i32 @printn(i32 %x) !dbg !6 {
main_fn_entry:
  %x1 = alloca i32, align 4
  store i32 %x, ptr %x1, align 4
  %ret_tmp_var = alloca i32, align 4
  %input = alloca ptr, align 8
  store ptr @"%i", ptr %input, align 8
  %input2 = load ptr, ptr %input, align 8
  %printf_idx_1_arg = alloca i32, align 4
  %var_deref = load i32, ptr %x1, align 4
  store i32 %var_deref, ptr %printf_idx_1_arg, align 4
  %printf_idx_1_arg3 = load i32, ptr %printf_idx_1_arg, align 4
  %function_call = call i32 (ptr, ...) @printf(ptr %input2, i32 %printf_idx_1_arg3)
  store i32 %function_call, ptr %ret_tmp_var, align 4
  %ret_tmp_var4 = load i32, ptr %ret_tmp_var, align 4
  store i32 %ret_tmp_var4, ptr %ret_tmp_var, align 4
  %ret_tmp_var5 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var5
}

define void @hi_from_ffi() !dbg !10 {
main_fn_entry:
  call void @hi_from_cpp()
  ret void
}

define %Alma @make_alma() !dbg !11 {
main_fn_entry:
  %ret_tmp_var = alloca %Alma, align 8
  %function_call = call { i32, i32 } @alma_csinalo()
  store { i32, i32 } %function_call, ptr %ret_tmp_var, align 4
  %ret_tmp_var1 = load %Alma, ptr %ret_tmp_var, align 4
  store %Alma %ret_tmp_var1, ptr %ret_tmp_var, align 4
  %ret_tmp_var2 = load %Alma, ptr %ret_tmp_var, align 4
  ret %Alma %ret_tmp_var2
}

define void @open_win(ptr %win_name) !dbg !15 {
main_fn_entry:
  %win_name1 = alloca ptr, align 8
  store ptr %win_name, ptr %win_name1, align 8
  %win_name2 = alloca ptr, align 8
  %var_deref = load ptr, ptr %win_name1, align 8
  store ptr %var_deref, ptr %win_name2, align 8
  %win_name3 = load ptr, ptr %win_name2, align 8
  call void @open_window(ptr %win_name3)
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
!6 = distinct !DISubprogram(name: "printn", linkageName: "printn", scope: !1, file: !1, line: 69, type: !7, scopeLine: 69, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!7 = !DISubroutineType(types: !8)
!8 = !{!9, !9}
!9 = !DIBasicType(name: "I32", size: 4, encoding: DW_ATE_signed)
!10 = distinct !DISubprogram(name: "hi_from_ffi", linkageName: "hi_from_ffi", scope: !1, file: !1, line: 69, type: !4, scopeLine: 69, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!11 = distinct !DISubprogram(name: "make_alma", linkageName: "make_alma", scope: !1, file: !1, line: 69, type: !12, scopeLine: 69, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!12 = !DISubroutineType(types: !13)
!13 = !{!14}
!14 = !DICompositeType(tag: DW_TAG_structure_type, name: "Alma", scope: !1, file: !1, line: 69, size: 8, align: 4, elements: !8, runtimeLang: DW_LANG_C89, identifier: "1")
!15 = distinct !DISubprogram(name: "open_win", linkageName: "open_win", scope: !1, file: !1, line: 69, type: !16, scopeLine: 69, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!16 = !DISubroutineType(types: !17)
!17 = !{null, !18}
!18 = !DIBasicType(name: "String", size: 24, encoding: DW_ATE_ASCII)
