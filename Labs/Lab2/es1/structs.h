
typedef struct {
  int type;
  float val;
  long timestamp;
} ValueStruct;
typedef struct {
  int type;
  float val[10];
  long timestamp;
} MValueStruct;
typedef struct {
  int type;
  char message[21]; // stringa null terminated lung max 20
} MessageStruct;
typedef struct {
  int type;
  union {
    ValueStruct val;
    MValueStruct mvals;
    MessageStruct messages;
  };
} ExportData;
enum {
    TYPE_VALUE = 1,
    TYPE_MVALUE = 2,
    TYPE_MESSAGE = 3
};