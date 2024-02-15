$ZIP = "linux"
$EXEC_TARGET = "template"
$EXEC_ZIP = "template"

$env:RUSTFLAGS = '--remap-path-prefix C:\\Users\\Pawel\\=~'

Write-Output "Building Linux binary..."
Start-Process -FilePath "cross" -ArgumentList "build --release --target=x86_64-unknown-linux-gnu" -Wait -NoNewWindow

Write-Output "Preparing files..."
New-Item -Path "." -Name "target\tmp" -ItemType Directory -Force
Copy-Item -Path ".\data\" -Destination ".\target\tmp\data\" -Recurse
Copy-Item -Path ".\target\x86_64-unknown-linux-gnu\release\$EXEC_TARGET" -Destination ".\target\tmp\$EXEC_ZIP"

Write-Output "Making archive..."
Set-Location .\target\tmp\
Start-Process -FilePath "7z" -ArgumentList "a -tzip $ZIP.zip *" -Wait -NoNewWindow
Set-Location ..\..\
Copy-Item -Path ".\target\tmp\$ZIP.zip" -Destination ".\target\deploy\$ZIP.zip"

Write-Output "Cleaning temporary files..."
Remove-Item -Path ".\target\tmp\" -Recurse

Write-Output "Done"