
$content = Get-Content "scripts/install.ps1" -Raw
for ($i=0; $i -lt $content.Length; $i++) {
    if ($content[$i] -eq "'") {
        # Find context
        $start = [math]::Max(0, $i - 20)
        $len = [math]::Min(40, $content.Length - $start)
        $context = $content.Substring($start, $len)
        Write-Host "Quote at index $i: $context"
    }
}
