$ZIP = "web"
$URL = "./"

$env:RUSTFLAGS = '--remap-path-prefix C:\Users\Pawel\=~\'

Write-Output "Building Web binary..."
Start-Process -FilePath "trunk" -ArgumentList "build --release --public-url $URL" -Wait -NoNewWindow
Set-Location .\dist\

Write-Output "Optimizing WASM..."
Get-ChildItem -Path .\ -Filter *.wasm -File -Name | ForEach-Object {
	Start-Process -FilePath "wasm-opt" -ArgumentList "-Os -o $_ $_" -Wait -NoNewWindow
}

Write-Output "Making archive..."
Start-Process -FilePath "7z" -ArgumentList "a -tzip $ZIP.zip *" -Wait -NoNewWindow
Set-Location ..

Write-Output "Copying files..."
New-Item -Path "." -Name "target\deploy" -ItemType Directory -Force
Copy-Item -Path ".\dist\$ZIP.zip" -Destination ".\target\deploy\$ZIP.zip"

Write-Output "Done"