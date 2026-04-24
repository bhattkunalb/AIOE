
$content = Get-Content "scripts/install.ps1" -Raw
$errors = $null
$tokens = $null
$ast = [System.Management.Automation.Language.Parser]::ParseInput($content, [ref]$tokens, [ref]$errors)

if ($errors) {
    foreach ($e in $errors) {
        Write-Host "Line $($e.Extent.StartLineNumber): $($e.Message)"
    }
} else {
    Write-Host "Valid"
}
