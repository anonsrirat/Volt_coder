# Generates the Volt app icons (violet rounded square + white bolt).
Add-Type -AssemblyName System.Drawing

$outDir = Join-Path $PSScriptRoot "icons"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

function New-PtF([double]$x, [double]$y) {
    return (New-Object System.Drawing.PointF -ArgumentList ([float]$x), ([float]$y))
}

function New-VoltBitmap([int]$size) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias

    $rect = New-Object System.Drawing.Rectangle(0, 0, $size, $size)
    $radius = [int]($size * 0.22)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = $radius * 2
    $path.AddArc(0, 0, $d, $d, 180, 90)
    $path.AddArc($size - $d, 0, $d, $d, 270, 90)
    $path.AddArc($size - $d, $size - $d, $d, $d, 0, 90)
    $path.AddArc(0, $size - $d, $d, $d, 90, 90)
    $path.CloseFigure()

    $brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        $rect,
        [System.Drawing.Color]::FromArgb(124, 77, 255),
        [System.Drawing.Color]::FromArgb(90, 31, 216),
        45.0)
    $g.FillPath($brush, $path)

    # White bolt polygon, scaled from a 24x24 design grid.
    $s = [double]$size / 24.0
    $pts = New-Object 'System.Drawing.PointF[]' 6
    $pts[0] = New-PtF (13.0 * $s) (2.0 * $s)
    $pts[1] = New-PtF (4.5 * $s) (13.5 * $s)
    $pts[2] = New-PtF (11.0 * $s) (13.5 * $s)
    $pts[3] = New-PtF (10.0 * $s) (22.0 * $s)
    $pts[4] = New-PtF (18.5 * $s) (10.5 * $s)
    $pts[5] = New-PtF (12.0 * $s) (10.5 * $s)

    $white = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
    $g.FillPolygon($white, $pts)

    $g.Dispose()
    return $bmp
}

function Save-Png([int]$size, [string]$name) {
    $bmp = New-VoltBitmap $size
    $path = Join-Path $outDir $name
    $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Host "wrote $path"
}

Save-Png 32  "32x32.png"
Save-Png 128 "128x128.png"
Save-Png 256 "128x128@2x.png"
Save-Png 512 "icon.png"

$big = New-VoltBitmap 256
$hicon = $big.GetHicon()
$icon = [System.Drawing.Icon]::FromHandle($hicon)
$fs = [System.IO.File]::Create((Join-Path $outDir "icon.ico"))
$icon.Save($fs)
$fs.Close()
$big.Dispose()
Write-Host "wrote icon.ico"
Write-Host "done."
