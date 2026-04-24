
$content = Get-Content "scripts/install.ps1" -Raw
$doubleQuotes = [regex]::Matches($content, '"').Count
$singleQuotes = [regex]::Matches($content, "'").Count
Write-Host "Double Quotes: $doubleQuotes"
Write-Host "Single Quotes: $singleQuotes"

if ($doubleQuotes % 2 -ne 0) {
    Write-Host "UNBALANCED DOUBLE QUOTES"
}
if ($singleQuotes % 2 -ne 0) {
    Write-Host "UNBALANCED SINGLE QUOTES"
}
