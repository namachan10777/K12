#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>

int main() {
	int fd = open("test", O_WRONLY);
	fchmod(fd, 0777);
	close(fd);
}
