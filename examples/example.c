#include <rust_py.h>
#include <stdio.h>
#include <errno.h>
#include <stdlib.h>
#include <pthread.h>
#include <sys/types.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <string.h>
#include <ctype.h>

# define handle_error(err) ((err).obj==NULL || (err).error!=NULL) ? (printf("%s\n",(err).error),errno =EBADMSG,perror((err).error),exit(1),NULL) : (err).obj ;

// gcc -I../target -L../target/release -Wl,-rpath=../target/release -o example example.c -lrust_py -lpthread
int main() {
    Runtime *runtime = create_runtime();
    Channel *channel = create_client(runtime);

    printf("Value:  %p\n", runtime);
    printf("Value:  %p\n", channel);

    time_t secs = 2;
    time_t startTime = time(NULL);
    uint64_t iter = 0;
    while (true) {
        if (time(NULL) - startTime > secs) {
            startTime = time(NULL);
            printf("Iter in C %lu\n", iter);
            iter += 1;
        }
    }
}

