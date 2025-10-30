import i32 as ProcessID;

// stdio.h
foreign function printf(format: *[u8], ..) -> i32;
// unistd.h
foreign function getpid() -> ProcessID;
foreign function getppid() -> ProcessID;
foreign function fork() -> ProcessID;
// sys/wait.h
foreign function wait(status: *i32) -> ProcessID;

foreign function main() -> i32 {
    // Obtain the PID of the parent process (which is currently the only process)
    let parent_pid = getpid();

    printf("1. I am the parent with PID %d\n", parent_pid);

    // Fork this process (and obtain the PID of the child process in parent process)
    let child_pid = fork();

    if (child_pid == 0) {
        // This is the child process, so obtain the PID of this process
        let child_pid = getpid();

        printf("2. I am the child with PID %d\n", child_pid);

        printf("3. My parent has PID %d\n", parent_pid);
    }
    else {
        // This is the parent process, so wait until the child process finishes
        wait(null);

        printf("4. My child has PID %d\n", child_pid);
    }

    0
}
