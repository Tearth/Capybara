$ZIP = "windows"
$EXEC_TARGET = "template"
$EXEC_ZIP = "template"

$env:RUSTFLAGS = '--remap-path-prefix C:\\Users\\Pawel\\=~'

Write-Output "Building Windows binary..."
Start-Process -FilePath "cargo" -ArgumentList "build --release --target=x86_64-pc-windows-msvc" -Wait -NoNewWindow

Write-Output "Preparing files..."
New-Item -Path "." -Name "target\tmp" -ItemType Directory -Force
Copy-Item -Path ".\data\" -Destination ".\target\tmp\data\" -Recurse
Copy-Item -Path ".\target\x86_64-pc-windows-msvc\release\$EXEC_TARGET.exe" -Destination ".\target\tmp\$EXEC_ZIP.exe"

Write-Output "Making archive..."
Set-Location .\target\tmp\
Start-Process -FilePath "7z" -ArgumentList "a -tzip $ZIP.zip *" -Wait -NoNewWindow
Set-Location ..\..\
Copy-Item -Path ".\target\tmp\$ZIP.zip" -Destination ".\target\deploy\$ZIP.zip"

Write-Output "Cleaning temporary files..."
Remove-Item -Path ".\target\tmp\" -Recurse

Write-Output "Done"