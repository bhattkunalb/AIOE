
$content = Get-Content "scripts/install.ps1" -Raw
$openBraces = [regex]::Matches($content, "{").Count
$closeBraces = [regex]::Matches($content, "}").Count
Write-Host "Open: $openBraces"
Write-Host "Close: $closeBraces"

if ($openBraces -ne $closeBraces) {
    Write-Host "MISMATCH FOUND!"
    # Find which function is not closed
    $lines = Get-Content "scripts/install.ps1"
    $depth = 0
    for ($i=0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        $o = [regex]::Matches($line, "{").Count
        $c = [regex]::Matches($line, "}").Count
        $depth += ($o - $c)
        if ($depth -lt 0) {
            Write-Host "Negative depth at line $($i+1): $line"
            $depth = 0
        }
    }
    Write-Host "Final depth: $depth"
}
