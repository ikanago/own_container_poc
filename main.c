#define _GNU_SOURCE
#include <errno.h>
#include <sched.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

typedef struct {
  int argc;
  char **argv;
} child_config_t;

const size_t STACK_SIZE = 1024 * 1024;

int child(void *arg) {
  child_config_t *config = arg;
  if (execve(config->argv[0], config->argv, NULL)) {
    fprintf(stderr, "execve failed %m\n");
    return -1;
  }
  return 0;
}

void run_container(child_config_t *config) {
  char *stack = malloc(STACK_SIZE);
  if (!stack) {
    fprintf(stderr, "malloc failed, out of memory\n");
    return;
  }

  int flags = 0;
  int child_pid = 0;
  if ((child_pid = clone(child, stack + STACK_SIZE, flags | SIGCHLD, config))) {
    fprintf(stderr, "clone failed: %s\n", strerror(errno));
  }

  if (waitpid(child_pid, NULL, 0)) {
    fprintf(stderr, "waitpid failed: %s\n", strerror(errno));
  }
}

int main(int argc, char **argv) {
  child_config_t config = {.argc = argc - 1, .argv = &argv[1]};
  run_container(&config);
}
