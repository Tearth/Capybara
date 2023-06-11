Set-Location ./dist/
Write-Output "Replacing paths..."
Write-Output "------------------"
Get-ChildItem -Path .\ -Filter *.html -File -Name | ForEach-Object {
	(Get-Content $_) -replace """/", """./" -replace "''/", "''./" | Out-File -encoding UTF8 $_
}
Write-Output "Optimizing WASM..."
Write-Output "------------------"
Get-ChildItem -Path .\ -Filter *.wasm -File -Name | ForEach-Object {
	Start-Process -FilePath "wasm-opt" -ArgumentList "-Os -o $_ $_" -Wait -NoNewWindow
}
Write-Output "Making archive..."
Write-Output "-----------------"
Compress-Archive -Path ./* -DestinationPath index.zip -Force
Set-Location ..