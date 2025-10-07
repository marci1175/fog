; ModuleID = 'main'
source_filename = "main"

%fing = type { i32 }

@"Number %i" = constant [10 x i8] c"Number %i\00"

declare i32 @printf(ptr, ...)

define %fing @finggen() !dbg !2 {
main_fn_entry:
  %ret_tmp_var = alloca %fing, align 8
  %strct_init = alloca %fing, align 8
  %asd = alloca i32, align 4
  store i32 35, ptr %asd, align 4
  %asd1 = load i32, ptr %asd, align 4
  %field_gep = getelementptr inbounds %fing, ptr %strct_init, i32 0, i32 0
  store i32 %asd1, ptr %field_gep, align 4
  %constructed_struct = load %fing, ptr %strct_init, align 4
  store %fing %constructed_struct, ptr %ret_tmp_var, align 4
  %ret_tmp_var2 = load %fing, ptr %ret_tmp_var, align 4
  ret %fing %ret_tmp_var2
}

define i32 @main() !dbg !8 {
main_fn_entry:
  %marci = alloca %fing, align 8
  %function_call = call %fing @finggen()
  store %fing %function_call, ptr %marci, align 4
  %marci1 = load %fing, ptr %marci, align 4
  store %fing %marci1, ptr %marci, align 4
  %input = alloca ptr, align 8
  store ptr @"Number %i", ptr %input, align 8
  %input2 = load ptr, ptr %input, align 8
  %printf_idx_1_arg = alloca i32, align 4
  %deref_strct_val = load i32, ptr %marci, align 4
  store i32 %deref_strct_val, ptr %printf_idx_1_arg, align 4
  %printf_idx_1_arg3 = load i32, ptr %printf_idx_1_arg, align 4
  %function_call4 = call i32 (ptr, ...) @printf(ptr %input2, i32 %printf_idx_1_arg3)
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
!2 = distinct !DISubprogram(name: "finggen", linkageName: "finggen", scope: !1, file: !1, line: 69, type: !3, scopeLine: 69, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!3 = !DISubroutineType(types: !4)
!4 = !{!5}
!5 = !DICompositeType(tag: DW_TAG_structure_type, name: "fing", scope: !1, file: !1, line: 69, size: 4, align: 4, elements: !6, runtimeLang: DW_LANG_C89, identifier: "1")
!6 = !{!7}
!7 = !DIBasicType(name: "I32", size: 4, encoding: DW_ATE_signed)
!8 = distinct !DISubprogram(name: "main", linkageName: "main", scope: !1, file: !1, line: 69, type: !9, scopeLine: 69, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!9 = !DISubroutineType(types: !6)
