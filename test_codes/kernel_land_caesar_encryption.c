#include <sys/syscall.h>
#include <stdio.h>
#include <unistd.h>

int main() {
	char src[] = "The quick brown fox jumps over the lazy dog!";
	syscall(439, 13, src, sizeof(src));
	printf("%s\n", src);
}
