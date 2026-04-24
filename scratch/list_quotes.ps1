
$lines = Get-Content "scripts/install.ps1"
for ($i=0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i]
    if ($line -match "'") {
        Write-Host "$($i+1): $line"
    }
}
