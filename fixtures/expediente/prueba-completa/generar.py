"""Genera los archivos de la prueba completa del Expediente (todo ficticio). Uso: python generar.py

Toma los textos de la institución de prueba (los 13 de ../ y los de textos/) y los pone en archivos como los que
sube una institución: PDF con texto, escaneados, fotos de celular, capturas, Word y Excel, y algunos con fallas
(contraseña, dañado, tipo que no se usa, en el lugar equivocado, sin texto). Lo que debe pasar con cada uno está en
esperado.json y en docs/16-institucion-de-prueba.md.
"""
import io
import random
import zipfile
from pathlib import Path

from docx import Document
from docx.shared import Inches, Pt
from openpyxl import Workbook
from openpyxl.styles import Font
from PIL import Image, ImageDraw, ImageFilter, ImageFont
from reportlab.lib import colors
from reportlab.lib.pagesizes import letter
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.utils import ImageReader
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.pdfgen import canvas
from reportlab.platypus import PageBreak, Paragraph, SimpleDocTemplate, Spacer, Table, TableStyle

HERE = Path(__file__).parent
OUT = HERE / "archivos"
FONTS = Path("C:/Windows/Fonts")

pdfmetrics.registerFont(TTFont("Times", str(FONTS / "times.ttf")))
pdfmetrics.registerFont(TTFont("Times-Bold", str(FONTS / "timesbd.ttf")))
BODY = ParagraphStyle("body", fontName="Times", fontSize=11.5, leading=15, spaceAfter=7)
HEAD = ParagraphStyle("head", fontName="Times-Bold", fontSize=13, leading=17, spaceAfter=9)


def pages_of(name):
    """The pages of a text with `=== PAGINA n ===`, each a list of paragraphs (the first line of the file is a title)."""
    path = HERE.parent / name if (HERE.parent / name).exists() else HERE / "textos" / name
    raw = path.read_text(encoding="utf-8")
    pages = []
    for chunk in raw.split("=== PAGINA ")[1:]:
        body = chunk.split("\n", 1)[1] if "\n" in chunk else ""
        pages.append([p.strip() for p in body.split("\n") if p.strip()])
    return pages


def is_title(line):
    letters = [c for c in line if c.isalpha()]
    return bool(letters) and all(c.isupper() for c in letters) and len(line) < 70


# ---------------------------------------------------------------- PDF with text


def text_pdf(name, pages, table_pages=()):
    """A PDF with its text, page by page. The pages in `table_pages` draw their «a | b» lines as a table with borders."""
    story = []
    for i, page in enumerate(pages):
        if i:
            story.append(PageBreak())
        rows = []
        for line in page:
            if i in table_pages and "|" in line:
                rows.append([c.strip() for c in line.split("|")])
                continue
            if rows:
                story.append(table(rows))
                rows = []
            story.append(Paragraph(line, HEAD if is_title(line) else BODY))
        if rows:
            story.append(table(rows))
    SimpleDocTemplate(str(OUT / name), pagesize=letter, leftMargin=72, rightMargin=72).build(story)


def table(rows):
    t = Table(rows, colWidths=[330, 110])
    t.setStyle(TableStyle([
        ("FONT", (0, 0), (-1, -1), "Times", 11),
        ("GRID", (0, 0), (-1, -1), 0.6, colors.black),
        ("ALIGN", (1, 0), (1, -1), "RIGHT"),
        ("FONT", (0, -1), (-1, -1), "Times-Bold", 11),
    ]))
    return t


# ---------------------------------------------------------------- what a scanner or a phone leaves

SERIF = ImageFont.truetype(str(FONTS / "times.ttf"), 34)
SERIF_BOLD = ImageFont.truetype(str(FONTS / "timesbd.ttf"), 38)


def wrap(text, font, width):
    words, lines, line = text.split(), [], ""
    for w in words:
        test = f"{line} {w}".strip()
        if font.getlength(test) <= width:
            line = test
        else:
            lines.append(line)
            line = w
    if line:
        lines.append(line)
    return lines


def sheet(page, height=2339):
    """A letter sheet at 200 dpi with the page's text, a little off white, as a scanner leaves it."""
    img = Image.new("L", (1700, height), 246)
    d = ImageDraw.Draw(img)
    y = 170
    for para in page:
        font = SERIF_BOLD if is_title(para) else SERIF
        for line in wrap(para, font, 1380):
            d.text((160, y), line, fill=22, font=font)
            y += 50
        y += 22
    return img


def dirty(img, seed, amount=6000):
    rnd = random.Random(seed)
    px = img.load()
    for _ in range(amount):
        px[rnd.randrange(img.width), rnd.randrange(img.height)] = rnd.randrange(120, 250)
    return img.rotate(0.4, fillcolor=246)


def worn(img):
    """A bad copy: blurred, washed out and dirty."""
    img = img.filter(ImageFilter.GaussianBlur(1.6)).point(lambda v: 125 + v * 125 // 255)
    return dirty(img, 7, 22000)


def scanned_pdf(name, pages, turned=(), bad=False):
    imgs = []
    for i, page in enumerate(pages):
        img = dirty(sheet(page), i)
        if bad:
            img = worn(img)
        if i in turned:
            # put in the scanner on its side
            img = img.rotate(-90, expand=True)
        imgs.append(img)
    imgs[0].save(OUT / name, "PDF", resolution=200.0, save_all=True, append_images=imgs[1:])


def phone_photo(name, page, blur=0.6, angle=1.6, quality=82):
    """A photo of a page taken with a phone: the sheet a little turned on a gray table, soft."""
    img = sheet(page)
    bottom = 170 + sum(len(wrap(p, SERIF, 1380)) * 50 + 22 for p in page) + 120
    img = img.crop((0, 0, 1700, min(bottom, 2339))).point(lambda v: 30 + v * 200 // 255)
    img = img.rotate(angle, expand=True, fillcolor=95).filter(ImageFilter.GaussianBlur(blur))
    table_ = Image.new("L", (img.width + 160, img.height + 160), 95)
    table_.paste(img, (80, 80))
    table_.convert("RGB").save(OUT / name, "JPEG", quality=quality)


def screenshot(name, page):
    """A screenshot of a registry's page at 96 dpi: letters about 7 pixels tall."""
    sans = ImageFont.truetype(str(FONTS / "arial.ttf"), 10)
    bold = ImageFont.truetype(str(FONTS / "arialbd.ttf"), 11)
    lines = [l for p in page for l in wrap(p, sans, 600)]
    shot = Image.new("RGB", (640, 40 + len(lines) * 17 + 20), "white")
    d = ImageDraw.Draw(shot)
    d.rectangle([0, 0, 640, 26], fill=(31, 78, 120))
    d.text((10, 7), "Registro Federal de las OSC · Consulta de la constancia", fill="white", font=bold)
    for i, line in enumerate(lines):
        d.text((14, 38 + i * 17), line, fill=(25, 25, 25), font=sans)
    shot.save(OUT / name, "PNG")


# ---------------------------------------------------------------- Word, Excel and the rest


def word(name, pages, picture_pages=()):
    """A Word file; the pages in `picture_pages` go as a scanned page pasted in."""
    doc = Document()
    doc.styles["Normal"].font.name = "Times New Roman"
    doc.styles["Normal"].font.size = Pt(12)
    for i, page in enumerate(pages):
        if i:
            doc.add_page_break()
        if i in picture_pages:
            buf = io.BytesIO()
            dirty(sheet(page, height=1500), 40 + i).save(buf, "PNG")
            buf.seek(0)
            doc.add_picture(buf, width=Inches(6.3))
            continue
        for para in page:
            p = doc.add_paragraph(para)
            if is_title(para):
                p.runs[0].bold = True
    doc.save(OUT / name)


def excel(name, pages):
    wb = Workbook()
    ws = wb.active
    ws.title = "Presupuesto 2026"
    ws.column_dimensions["A"].width = 52
    ws.column_dimensions["B"].width = 16
    r = 1
    for line in pages[0]:
        cells = [c.strip() for c in line.split("|")]
        ws.cell(r, 1, cells[0]).font = Font(bold=len(cells) == 1 or cells[0].startswith("Total"))
        if len(cells) > 1:
            ws.cell(r, 2, int(cells[1].replace(",", ""))).number_format = "#,##0"
        r += 1
    wb.save(OUT / name)


def organization_chart(name):
    """The chart of the areas of the house, drawn in boxes: no names of people, only areas."""
    sans = ImageFont.truetype(str(FONTS / "arial.ttf"), 30)
    bold = ImageFont.truetype(str(FONTS / "arialbd.ttf"), 40)
    img = Image.new("RGB", (2000, 1250), "white")
    d = ImageDraw.Draw(img)
    d.text((560, 50), "ORGANIGRAMA 2026", fill="black", font=bold)
    d.text((440, 110), "Casa de Reposo Ficticia San Ejemplo, I.A.P.", fill="black", font=sans)

    def box(cx, y, text):
        w = max(sans.getlength(t) for t in text.split("\n")) + 60
        h = 50 + 40 * text.count("\n")
        d.rectangle([cx - w / 2, y, cx + w / 2, y + h], outline="black", width=3)
        for i, t in enumerate(text.split("\n")):
            d.text((cx - sans.getlength(t) / 2, y + 10 + i * 40), t, fill="black", font=sans)
        return y + h

    bottom = box(1000, 200, "Patronato")
    d.line([1000, bottom, 1000, 330], fill="black", width=3)
    bottom = box(1000, 330, "Dirección General")
    d.line([1000, bottom, 1000, 470], fill="black", width=3)
    areas = ["Enfermería", "Cocina y\nnutrición", "Terapia física\ny ocupacional", "Trabajo\nsocial", "Administración", "Mantenimiento\ny limpieza"]
    d.line([175, 470, 175 + 5 * 330, 470], fill="black", width=3)
    for i, area in enumerate(areas):
        cx = 175 + i * 330
        d.line([cx, 470, cx, 520], fill="black", width=3)
        box(cx, 520, area)
    d.text((160, 1120), "Aprobado por el Patronato en su sesión del 12 de diciembre de 2025.", fill="black", font=sans)
    img.save(OUT / name, "PNG")


def mixed_contract(name, pages):
    """Page 1: a printed header and its body scanned (read as a picture, all of it); page 2 with its text."""
    first = pages[0]
    body = Image.new("L", (1500, 1700), 248)
    d = ImageDraw.Draw(body)
    y = 40
    for para in first[1:]:
        for line in wrap(para, SERIF, 1400):
            d.text((50, y), line, fill=25, font=SERIF)
            y += 48
        y += 20
    buf = io.BytesIO()
    dirty(body, 3).save(buf, "PNG")
    buf.seek(0)
    c = canvas.Canvas(str(OUT / name), pagesize=letter)
    c.setFont("Times-Bold", 14)
    c.drawString(72, 740, first[0])
    c.drawImage(ImageReader(buf), 36, 60, width=540, height=640)
    c.showPage()
    c.setFont("Times", 11.5)
    y = 720
    for para in pages[1]:
        for line in wrap(para, ImageFont.truetype(str(FONTS / "times.ttf"), 12), 460):
            c.drawString(72, y, line)
            y -= 15
        y -= 8
    c.showPage()
    c.save()


def with_password(name, pages):
    c = canvas.Canvas(str(OUT / name), pagesize=letter, encrypt="clave-ficticia")
    c.setFont("Times", 11.5)
    y = 720
    for para in pages[0]:
        for line in wrap(para, ImageFont.truetype(str(FONTS / "times.ttf"), 12), 460):
            c.drawString(72, y, line)
            y -= 15
        y -= 8
    c.save()


def damaged(name, source):
    """A PDF cut in half, as when a download fails."""
    raw = (OUT / source).read_bytes()
    (OUT / name).write_bytes(raw[: len(raw) // 2])


def open_document(name):
    """A file of a kind the Expediente does not take (OpenDocument)."""
    with zipfile.ZipFile(OUT / name, "w") as z:
        z.writestr("mimetype", "application/vnd.oasis.opendocument.text")
        z.writestr("content.xml", "<office:document-content/>")


def no_text_photo(name):
    """A photo so blurred that no letter can be read."""
    page = pages_of("licencia-uso-suelo.txt")[0]
    img = sheet(page).crop((0, 0, 1700, 1100)).filter(ImageFilter.GaussianBlur(9))
    img.rotate(4, expand=True, fillcolor=95).convert("RGB").save(OUT / name, "JPEG", quality=60)


def main():
    OUT.mkdir(exist_ok=True)
    text_pdf("01-acta-constitutiva.pdf", pages_of("acta-constitutiva.txt"))
    scanned_pdf("02-reformas-estatutos-2019-escaneo.pdf", pages_of("reformas-estatutos-2019.txt"))
    scanned_pdf("03-poder-notarial-hoja-girada.pdf", pages_of("poder-notarial.txt"), turned=(1,))
    text_pdf("04-constancia-situacion-fiscal.pdf", pages_of("constancia-situacion-fiscal.txt"))
    phone_photo("05-opinion-cumplimiento-foto.jpg", pages_of("opinion-cumplimiento.txt")[0])
    scanned_pdf("06-oficio-donataria-escaneo.pdf", pages_of("oficio-donataria.txt"))
    screenshot("07-cluni-captura.png", pages_of("cluni.txt")[0])
    scanned_pdf("08-registro-junta-destenido.pdf", pages_of("registro-junta.txt"), bad=True)
    word("09-aviso-funcionamiento.docx", pages_of("aviso-funcionamiento.txt"))
    with_password("10-proteccion-civil-con-contrasena.pdf", pages_of("programa-proteccion-civil.txt"))
    text_pdf("10b-proteccion-civil.pdf", pages_of("programa-proteccion-civil.txt"))
    word("11-aviso-privacidad.docx", pages_of("aviso-privacidad.txt"))
    word("12-reglamento-interno-con-hoja-escaneada.docx", pages_of("reglamento-interno.txt"), picture_pages=(2,))
    organization_chart("13-organigrama.png")
    text_pdf("14-estados-financieros-2025.pdf", pages_of("estados-financieros-2025.txt"), table_pages=(0,))
    damaged("14b-estados-financieros-2025-danado.pdf", "14-estados-financieros-2025.pdf")
    text_pdf("15-informe-anual-2025.pdf", pages_of("informe-anual-2025.txt"))
    open_document("15b-informe-anual-2025.odt")
    excel("16-presupuesto-junta-2026.xlsx", pages_of("presupuesto-junta-2026.txt"))
    text_pdf("16b-presupuesto-junta-2026.pdf", pages_of("presupuesto-junta-2026.txt"), table_pages=(0,))
    phone_photo("17-licencia-uso-suelo-foto.jpg", pages_of("licencia-uso-suelo.txt")[0], angle=-1.2)
    mixed_contract("18-contrato-comodato-pagina-mixta.pdf", pages_of("contrato-comodato.txt"))
    # the constancia of the SAT under a name that says acta: put on «Acta constitutiva», it fills nothing
    text_pdf("19-acta-constitutiva-version-final.pdf", pages_of("constancia-situacion-fiscal.txt"))
    no_text_photo("20-foto-sin-texto.jpg")
    print("listo:", len(list(OUT.iterdir())), "archivos en", OUT)


if __name__ == "__main__":
    main()
