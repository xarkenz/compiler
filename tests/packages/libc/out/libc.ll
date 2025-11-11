source_filename = "\\?\C:\Users\seane\Projects\compiler\tests\packages\libc\main.cupr"

declare i8* @malloc(i64)

declare i8* @calloc(i64, i64)

declare i8* @realloc(i8*, i64)

declare void @free(i8*)

declare i32 @rand()

declare void @srand(i32)

declare i32 @atexit(void()*)

declare void @exit(i32)

declare i32 @isalnum(i32)

declare i32 @isalpha(i32)

declare i32 @islower(i32)

declare i32 @isupper(i32)

declare i32 @isdigit(i32)

declare i32 @isxdigit(i32)

declare i32 @iscntrl(i32)

declare i32 @isgraph(i32)

declare i32 @isspace(i32)

declare i32 @isblank(i32)

declare i32 @isprint(i32)

declare i32 @ispunct(i32)

declare i32 @tolower(i32)

declare i32 @toupper(i32)

%"type.::libc::stdio::CFile" = type opaque

declare %"type.::libc::stdio::CFile"* @fopen(i8*, i8*)

declare i32 @fclose(%"type.::libc::stdio::CFile"*)

declare i32 @feof(%"type.::libc::stdio::CFile"*)

declare i8* @fgets(i8*, i32, %"type.::libc::stdio::CFile"*)

declare i32 @printf(i8*, ...)

declare i32 @puts(i8*)

declare i64 @strlen(i8*)

declare i8* @memcpy(i8*, i8*, i64)

