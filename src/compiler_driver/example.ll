; ModuleID = 'example.c'
source_filename = "example.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.MyStruct = type { i32, i64, i8 }

@__const.main.my_init_struct = private unnamed_addr constant %struct.MyStruct { i32 1, i64 2, i8 3 }, align 8
@my_global_struct = dso_local global %struct.MyStruct zeroinitializer, align 8

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca %struct.MyStruct, align 8
  %3 = alloca %struct.MyStruct, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i8, align 1
  store i32 0, i32* %1, align 4
  %7 = getelementptr inbounds %struct.MyStruct, %struct.MyStruct* %2, i32 0, i32 2
  store i8 5, i8* %7, align 8
  %8 = bitcast %struct.MyStruct* %3 to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 8 %8, i8* align 8 bitcast (%struct.MyStruct* @__const.main.my_init_struct to i8*), i64 24, i1 false)
  store i64 7, i64* getelementptr inbounds (%struct.MyStruct, %struct.MyStruct* @my_global_struct, i32 0, i32 1), align 8
  store i32 3, i32* %4, align 4
  store i32 5, i32* %5, align 4
  %9 = load i32, i32* %4, align 4
  %10 = icmp eq i32 %9, 3
  br i1 %10, label %11, label %14

11:                                               ; preds = %0
  %12 = load i32, i32* %5, align 4
  %13 = icmp eq i32 %12, 5
  br label %14

14:                                               ; preds = %11, %0
  %15 = phi i1 [ false, %0 ], [ %13, %11 ]
  %16 = zext i1 %15 to i8
  store i8 %16, i8* %6, align 1
  %17 = load i32, i32* %4, align 4
  %18 = icmp sgt i32 %17, 2
  br i1 %18, label %22, label %19

19:                                               ; preds = %14
  %20 = load i32, i32* %5, align 4
  %21 = icmp sgt i32 %20, 8
  br label %22

22:                                               ; preds = %19, %14
  %23 = phi i1 [ true, %14 ], [ %21, %19 ]
  %24 = zext i1 %23 to i8
  store i8 %24, i8* %6, align 1
  %25 = load i32, i32* %4, align 4
  %26 = icmp slt i32 %25, 4
  br i1 %26, label %27, label %35

27:                                               ; preds = %22
  %28 = load i32, i32* %5, align 4
  %29 = icmp slt i32 %28, 2
  br i1 %29, label %33, label %30

30:                                               ; preds = %27
  %31 = load i32, i32* %5, align 4
  %32 = icmp sgt i32 %31, 3
  br label %33

33:                                               ; preds = %30, %27
  %34 = phi i1 [ true, %27 ], [ %32, %30 ]
  br label %35

35:                                               ; preds = %33, %22
  %36 = phi i1 [ false, %22 ], [ %34, %33 ]
  %37 = zext i1 %36 to i8
  store i8 %37, i8* %6, align 1
  ret i32 0
}

; Function Attrs: argmemonly nofree nounwind willreturn
declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg) #1

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { argmemonly nofree nounwind willreturn }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 7, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 1}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 14.0.0-1ubuntu1.1"}
