#include <stdio.h>
#include <string.h>

int main(int argc, char **argv)
{
    if (argc != 2)
    {
        fprintf(stderr, "the command accepts only one argument\n");
        return 1;
    }

    char buffer[3] = {0};

    // if (strlen(argv[1]) >= 3 && argv[1][0] == 'x')
    //{
    //   printf("copy!\n");
    strcpy(buffer, argv[1]);
    //}

    printf("all good boss!\n");

    return 0;
}