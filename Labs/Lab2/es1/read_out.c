#include <stdio.h>
#include <stdlib.h>
#include "structs.h"

int main() {
    FILE* pf = fopen("data.bin", "rb");
    ExportData data[100];
    fread(data, sizeof(ExportData), 100, pf);
    for (int i = 0; i < 100; i++) {
        switch (data[i].type) {
            case TYPE_VALUE:
                printf("Value: %f, timestamp: %ld", data[i].val.val, data[i].val.timestamp);
                break;
            case TYPE_MVALUE:
                printf("MValue: ");
                for (int j = 0; j < 10; j++) {
                    printf("%f ", data[i].mvals.val[j]);
                }
                printf(", timestamp: %ld", data[i].mvals.timestamp);
                break;
            case TYPE_MESSAGE:
                printf("Message: %s", data[i].messages.message);
                break;
        }
        printf("\n");
    }
    fclose(pf);
}