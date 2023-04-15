cl /O2 /nologo main.c | Out-Null
cl /O2 /nologo read_out.c | Out-Null
./main.exe
./read_out.exe > out.txt
cargo run --quiet -- --file data.bin > out2.txt
$ret = Compare-Object -ReferenceObject (Get-Content -Path out.txt) -DifferenceObject (Get-Content -Path out2.txt)
if ($null -eq $ret) {
    Write-Host "PASS"
} else {
    Write-Host "FAIL"
}
rm out.txt
rm out2.txt
rm main.obj
rm read_out.obj
rm read_out.exe
rm main.exe
rm data.bin