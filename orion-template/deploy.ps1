Set-Location ./dist/
Write-Output "Replacing paths..."
Get-ChildItem -Path .\ -Filter *.html -File -Name | ForEach-Object {
	(Get-Content $_) -replace """/", """./" -replace "'/", "'./" | Out-File -encoding ASCII $_
}
Write-Output "Optimizing WASM..."
Get-ChildItem -Path .\ -Filter *.wasm -File -Name | ForEach-Object {
	Start-Process -FilePath "wasm-opt" -ArgumentList "-Os -o $_ $_" -Wait -NoNewWindow
}
Write-Output "Making archive..."
Start-Process -FilePath "7z" -ArgumentList "a -tzip index.zip *" -Wait -NoNewWindow
Write-Output "Done"
Set-Location ..