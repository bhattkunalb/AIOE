
$content = Get-Content "scripts/install.ps1" -Raw
for ($i=0; $i -lt $content.Length; $i++) {
    if ($content[$i] -eq "'") {
        $context = $content.Substring([math]::Max(0, $i - 20), [math]::Min(40, $content.Length - [math]::Max(0, $i - 20)))
        Write-Host "Quote at index $i --- $context"
    }
}
