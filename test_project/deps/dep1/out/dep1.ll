; ModuleID = 'dep1'
source_filename = "dep1"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

%Alma = type { i32, i32 }

@Alma = constant [5 x i8] c"Alma\00"
@"%i" = constant [3 x i8] c"%i\00"

declare void @hi_from_cpp()

declare { i32, i32 } @alma_csinalo()

declare void @open_window(ptr)

declare i32 @printf(ptr, ...)

define void @kedvenc() {
main_fn_entry:
  %function_call = call i32 (ptr, ...) @printf(ptr @Alma)
  ret void
}

define i32 @printn(i32 %x) {
main_fn_entry:
  %function_call = call i32 (ptr, ...) @printf(ptr @"%i", i32 %x)
  ret i32 %function_call
}

define void @hi_from_ffi() {
main_fn_entry:
  call void @hi_from_cpp()
  ret void
}

define %Alma @make_alma() {
main_fn_entry:
  %ret_tmp_var = alloca %Alma, align 8
  %function_call = call { i32, i32 } @alma_csinalo()
  store { i32, i32 } %function_call, ptr %ret_tmp_var, align 4
  %ret_tmp_var1 = load %Alma, ptr %ret_tmp_var, align 4
  store %Alma %ret_tmp_var1, ptr %ret_tmp_var, align 4
  %ret_tmp_var2 = load %Alma, ptr %ret_tmp_var, align 4
  ret %Alma %ret_tmp_var2
}

define void @open_win(ptr %win_name) {
main_fn_entry:
  call void @open_window(ptr %win_name)
  ret void
}

!llvm.dbg.cu = !{!0}
!llvm.debug.version = !{!2}

!0 = distinct !DICompileUnit(language: DW_LANG_C, file: !1, producer: "Fog (ver.: 0.1.0) with LLVM 18-1-8", isOptimized: true, runtimeVersion: 1, emissionKind: LineTablesOnly, splitDebugInlining: false)
!1 = !DIFile(filename: "dep1", directory: "C:\\Users\\marci\\Desktop\\fog\\test_project\\deps\\dep1\\deps\\src")
!2 = !{i32 1}
