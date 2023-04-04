#!/usr/bin/env fish
clang -O2 main.c -o generate_data
clang -O2 read_out.c -o reader
./generate_data
./reader > out.txt
cargo run --quiet -- --file data.bin > out2.txt
diff out.txt out2.txt
if [ $status. -eq 0 ]
    echo "PASS"
else
    echo "FAIL"
end
rm out.txt
rm out2.txt
rm reader
rm generate_data
rm data.bin