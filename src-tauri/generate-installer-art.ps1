# Generates the NSIS installer artwork (header + sidebar) for Volt.
Add-Type -AssemblyName System.Drawing

$outDir = Join-Path $PSScriptRoot "installer"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

function New-Gradient([int]$w, [int]$h, [int]$hue) {
    $bmp = New-Object System.Drawing.Bitmap($w, $h)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $rect = New-Object System.Drawing.Rectangle(0, 0, $w, $h)
    $c1 = [System.Drawing.Color]::FromArgb(124, 77, 255)
    $c2 = [System.Drawing.Color]::FromArgb(56, 20, 130)
    $brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush($rect, $c1, $c2, 25.0)
    $g.FillRectangle($brush, $rect)
    $g.Dispose()
    return $bmp
}

function New-PtF([double]$x, [double]$y) {
    return (New-Object System.Drawing.PointF -ArgumentList ([float]$x), ([float]$y))
}

function Draw-Bolt($g, [double]$cx, [double]$cy, [double]$s) {
    $pts = New-Object 'System.Drawing.PointF[]' 6
    $pts[0] = New-PtF ($cx + 1.0 * $s) ($cy - 10.0 * $s)
    $pts[1] = New-PtF ($cx - 7.5 * $s) ($cy + 1.5 * $s)
    $pts[2] = New-PtF ($cx - 1.0 * $s) ($cy + 1.5 * $s)
    $pts[3] = New-PtF ($cx - 2.0 * $s) ($cy + 10.0 * $s)
    $pts[4] = New-PtF ($cx + 6.5 * $s) ($cy - 1.5 * $s)
    $pts[5] = New-PtF ($cx) ($cy - 1.5 * $s)
    $white = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
    $g.FillPolygon($white, $pts)
}

# --- Sidebar (164 x 314) ---
$sb = New-Gradient 164 314 262
$g = [System.Drawing.Graphics]::FromImage($sb)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
Draw-Bolt $g 82 90 2.6
$font = New-Object System.Drawing.Font("Segoe UI", 20, [System.Drawing.FontStyle]::Bold)
$small = New-Object System.Drawing.Font("Segoe UI", 8.5, [System.Drawing.FontStyle]::Regular)
$white = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
$soft = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(215, 215, 255))
$sf = New-Object System.Drawing.StringFormat
$sf.Alignment = [System.Drawing.StringAlignment]::Center
$g.DrawString("Volt", $font, $white, (New-Object System.Drawing.RectangleF(0, 130, 164, 40)), $sf)
$g.DrawString("AI Coding Agent", $small, $soft, (New-Object System.Drawing.RectangleF(0, 168, 164, 30)), $sf)
$g.DrawString("v0.2.0", $small, $soft, (New-Object System.Drawing.RectangleF(0, 285, 164, 20)), $sf)
$g.Dispose()
$sb.Save((Join-Path $outDir "sidebar.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$sb.Dispose()
Write-Host "wrote sidebar.bmp"

# --- Header (150 x 57) ---
$hd = New-Gradient 150 57 262
$g = [System.Drawing.Graphics]::FromImage($hd)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
Draw-Bolt $g 26 28 0.95
$font = New-Object System.Drawing.Font("Segoe UI", 12, [System.Drawing.FontStyle]::Bold)
$white = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
$g.DrawString("Volt", $font, $white, 46.0, 16.0)
$g.Dispose()
$hd.Save((Join-Path $outDir "header.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$hd.Dispose()
Write-Host "wrote header.bmp"
Write-Host "done."
