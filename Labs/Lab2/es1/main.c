#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include "structs.h"

void generate_export_data(ExportData *data) {
    int type = rand() % 3 + 1;
    data->type = type;
    switch (type) {
        case TYPE_VALUE:
            data->val.type = TYPE_VALUE;
            data->val.val = (float)rand() / (float)RAND_MAX;
            data->val.timestamp = rand();
            break;
        case TYPE_MVALUE:
            data->mvals.type = TYPE_MVALUE;
            for (int i = 0; i < 10; i++) {
                data->mvals.val[i] = (float)rand() / (float)RAND_MAX;
            }
            data->mvals.timestamp = rand();
            break;
        case TYPE_MESSAGE:
            data->messages.type = TYPE_MESSAGE;
            for (int i = 0; i < 20; i++) {
                data->messages.message[i] = 'a' + rand() % 26;
            }
            data->messages.message[20] = '\0';
            break;
    }
}

void export(ExportData *data, int n, FILE* pf) {
    fwrite(data, sizeof(ExportData), n, pf);
}
int main() {
    srand(time(NULL));
    ExportData data[100];
    for (int i = 0; i < 100; i++) {
        generate_export_data(&data[i]);
    }
    FILE* pf = fopen("data.bin", "wb");
    export(data, 100, pf);
    fclose(pf);
    return 0;
}