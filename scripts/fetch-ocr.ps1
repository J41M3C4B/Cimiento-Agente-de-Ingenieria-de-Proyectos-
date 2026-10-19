# Fetches what reads scanned pages and photos (OCR, ADR-035) into src-tauri/resources/ocr, where the installer
# takes it from. Run it once after cloning (and again if this file changes):
#   powershell -ExecutionPolicy Bypass -File scripts/fetch-ocr.ps1
# Every download is checked against its SHA-256; nothing is installed on the computer (7-Zip is only unpacked to
# a temporary folder to open the installer of Tesseract).
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $root "src-tauri\resources\ocr"
$work = Join-Path ([System.IO.Path]::GetTempPath()) ("cimiento-ocr-" + [guid]::NewGuid())

$downloads = @{
    tesseract = @{ url = "https://github.com/tesseract-ocr/tesseract/releases/download/5.5.3/tesseract-ocr-w64-setup-5.5.3.20260724.exe"; sha = "bee9e3434bd94fd65387d9be28cd467a41f61b1275383b55b0f59a1331270ae4" }
    # the precise model for Spanish (tessdata_best): fewer mistakes in names, amounts and folios
    spanish   = @{ url = "https://raw.githubusercontent.com/tesseract-ocr/tessdata_best/main/spa.traineddata"; sha = "e2c1ffdad8b30f26c45d4017a9183d3a7f9aa69e59918be4f88b126fac99ab2c" }
    # English goes first in the reading: the Spanish model has no «@», and an email it misreads escapes the scanner
    english   = @{ url = "https://raw.githubusercontent.com/tesseract-ocr/tessdata_best/main/eng.traineddata"; sha = "8280aed0782fe27257a68ea10fe7ef324ca0f8d85bd2fd145d1c2b560bcb66ba" }
    # which way a page is turned (ADR-035): a sheet scanned sideways is put upright before it is read again
    osd       = @{ url = "https://raw.githubusercontent.com/tesseract-ocr/tessdata_best/main/osd.traineddata"; sha = "9cf5d576fcc47564f11265841e5ca839001e7e6f38ff7f7aacf46d15a96b00ff" }
    # the version pdfium-render reads with (feature pdfium_latest = 7881)
    pdfium    = @{ url = "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7881/pdfium-win-x64.tgz"; sha = "73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08" }
    sevenzip  = @{ url = "https://github.com/ip7z/7zip/releases/download/26.04/7z2604-x64.msi"; sha = "0b01334a418654293513449f61d6bfc99e5196ca371e3c3dc961eda57cd535c6" }
}

# What tesseract.exe needs from its installer (the rest are training tools)
$libraries = @(
    "libarchive-13.dll", "libb2-1.dll", "libbrotlicommon.dll", "libbrotlidec.dll", "libbz2-1.dll", "libcurl-4.dll",
    "libdeflate.dll", "libexpat-1.dll", "libgcc_s_seh-1.dll", "libgif-7.dll", "libiconv-2.dll", "libidn2-0.dll",
    "libintl-8.dll", "libjbig-0.dll", "libjpeg-8.dll", "libleptonica-6.dll", "libLerc.dll", "liblz4.dll",
    "liblzma-5.dll", "libopenjp2-7.dll", "libpng16-16.dll", "libpsl-5.dll", "libsharpyuv-0.dll", "libssh2-1.dll",
    "libstdc++-6.dll", "libtesseract-5.dll", "libtiff-6.dll", "libunistring-5.dll", "libwebp-7.dll",
    "libwebpmux-3.dll", "libwinpthread-1.dll", "libzstd.dll", "zlib1.dll"
)

function Get-Checked($name) {
    $d = $downloads[$name]
    $file = Join-Path $work ("download-" + $name)
    Write-Host "Descargando $name..."
    Invoke-WebRequest -Uri $d.url -OutFile $file
    $sha = (Get-FileHash -Algorithm SHA256 $file).Hash.ToLower()
    if ($sha -ne $d.sha) { throw "$name no coincide con su huella ($sha)" }
    return $file
}

New-Item -ItemType Directory -Force $work | Out-Null
try {
    $setup = Get-Checked "tesseract"
    $spanish = Get-Checked "spanish"
    $english = Get-Checked "english"
    $osd = Get-Checked "osd"
    $pdfium = Get-Checked "pdfium"
    $msi = Get-Checked "sevenzip"

    $seven = Join-Path $work "7z"
    $p = Start-Process msiexec.exe -ArgumentList "/a `"$msi`" /qn TARGETDIR=`"$seven`"" -Wait -PassThru
    if ($p.ExitCode -ne 0) { throw "no se pudo abrir 7-Zip ($($p.ExitCode))" }
    $sevenExe = (Get-ChildItem -Recurse $seven -Filter 7z.exe | Select-Object -First 1).FullName
    $tess = Join-Path $work "tesseract"
    & $sevenExe x -y "-o$tess" $setup | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "no se pudo abrir el instalador de Tesseract" }
    $pdf = Join-Path $work "pdfium"
    New-Item -ItemType Directory -Force $pdf | Out-Null
    # the tar of Windows: the one of Git, when it comes first in the PATH, takes «C:» for another computer
    & (Join-Path $env:SystemRoot "System32/tar.exe") -xzf $pdfium -C $pdf
    if ($LASTEXITCODE -ne 0) { throw "no se pudo abrir Pdfium" }

    if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
    $bin = Join-Path $dest "tesseract"
    $data = Join-Path $dest "tessdata"
    $licenses = Join-Path $dest "licenses"
    New-Item -ItemType Directory -Force $bin, (Join-Path $data "configs"), (Join-Path $dest "pdfium"), $licenses | Out-Null
    Copy-Item (Join-Path $tess "tesseract.exe") $bin
    foreach ($l in $libraries) { Copy-Item (Join-Path $tess $l) $bin }
    Copy-Item (Join-Path $tess "tessdata\configs\tsv") (Join-Path $data "configs")
    Copy-Item $spanish (Join-Path $data "spa.traineddata")
    Copy-Item $english (Join-Path $data "eng.traineddata")
    Copy-Item $osd (Join-Path $data "osd.traineddata")
    Copy-Item (Join-Path $pdf "bin\pdfium.dll") (Join-Path $dest "pdfium")
    Copy-Item (Join-Path $tess "doc\LICENSE") (Join-Path $licenses "tesseract.txt")
    Copy-Item (Join-Path $pdf "LICENSE") (Join-Path $licenses "pdfium-binaries.txt")
    if (Test-Path (Join-Path $pdf "licenses")) { Copy-Item -Recurse (Join-Path $pdf "licenses") (Join-Path $licenses "pdfium") }
    Write-Host "Listo: $dest"
}
finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
