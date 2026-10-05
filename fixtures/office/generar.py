"""Genera los archivos de prueba de Office (datos ficticios). Uso: python generar.py"""
import io
from openpyxl import Workbook
from openpyxl.drawing.image import Image as XlImage
from openpyxl.styles import Alignment, Border, Font, PatternFill, Side
from openpyxl.worksheet.datavalidation import DataValidation
from PIL import Image, ImageDraw
from docx import Document
from docx.shared import Pt, RGBColor, Inches
from docx.enum.text import WD_ALIGN_PARAGRAPH


def logo_png(color, size=(120, 60)):
    img = Image.new("RGB", size, color)
    ImageDraw.Draw(img).rectangle([8, 8, size[0] - 8, size[1] - 8], outline="white", width=3)
    buf = io.BytesIO()
    img.save(buf, "PNG")
    buf.seek(0)
    return buf


def excel():
    wb = Workbook()
    ws = wb.active
    ws.title = "Datos generales"
    thin = Side(style="thin", color="444444")
    box = Border(left=thin, right=thin, top=thin, bottom=thin)
    answer = PatternFill("solid", fgColor="FFF2CC")
    head = PatternFill("solid", fgColor="1F4E78")

    ws.add_image(XlImage(logo_png("#1F4E78")), "A1")
    ws.merge_cells("C1:F2")
    ws["C1"] = "CUESTIONARIO DE SOLICITUD (FICTICIO)"
    ws["C1"].font = Font(bold=True, size=16, color="FFFFFF")
    ws["C1"].fill = head
    ws["C1"].alignment = Alignment(horizontal="center", vertical="center")
    ws.row_dimensions[1].height = 30
    ws.row_dimensions[2].height = 30
    ws.column_dimensions["A"].width = 38
    ws.column_dimensions["B"].width = 26
    ws.column_dimensions["C"].width = 18

    rows = [
        (5, "Nombre de la institución"),
        (6, "Número de beneficiarios directos"),
        (7, "Fecha de inicio del proyecto"),
        (8, "Tipo de institución"),
        (9, "Monto solicitado (MXN)"),
        (10, "Monto aportado por la institución (MXN)"),
    ]
    for r, q in rows:
        ws.cell(r, 1, q).font = Font(bold=True)
        ws.cell(r, 1).border = box
        ws.cell(r, 2).fill = answer
        ws.cell(r, 2).border = box
    ws.merge_cells("B5:D5")  # respuesta en celda combinada
    ws["B9"].number_format = '"$"#,##0.00'
    ws["B10"].number_format = '"$"#,##0.00'
    ws["B7"].number_format = "dd/mm/yyyy"

    ws["A11"] = "Total del proyecto (MXN)"
    ws["A11"].font = Font(bold=True)
    ws["B11"] = "=B9+B10"
    ws["B11"].number_format = '"$"#,##0.00'
    ws["A12"] = "Porcentaje de aportación propia"
    ws["B12"] = "=IF(B11=0,0,B10/B11)"
    ws["B12"].number_format = "0.0%"

    ws["A14"] = "Describa el impacto esperado"
    ws["A14"].font = Font(bold=True)
    ws.merge_cells("A15:F18")
    ws["A15"].fill = answer
    ws["A15"].alignment = Alignment(wrap_text=True, vertical="top")

    dv = DataValidation(type="list", formula1='"Asilo,Casa hogar,Otra"', allow_blank=False)
    ws.add_data_validation(dv)
    dv.add("B8")

    ws2 = wb.create_sheet("Presupuesto")
    ws2.add_image(XlImage(logo_png("#C55A11", (80, 40))), "F1")
    ws2.append(["Concepto", "Cantidad", "Precio unitario", "Importe"])
    for c in ws2[1]:
        c.font = Font(bold=True, color="FFFFFF")
        c.fill = head
    for i in range(2, 8):
        ws2.cell(i, 1).fill = answer
        ws2.cell(i, 2).fill = answer
        ws2.cell(i, 3).fill = answer
        ws2.cell(i, 4, f"=B{i}*C{i}")
    ws2["A8"] = "TOTAL"
    ws2["D8"] = "=SUM(D2:D7)"
    ws2["A8"].font = Font(bold=True)

    ws3 = wb.create_sheet("Catálogos")
    ws3["A1"] = "uso interno"
    ws3.sheet_state = "hidden"
    wb.save("cuestionario-prueba.xlsx")


def word():
    d = Document()
    sec = d.sections[0]
    sec.header.paragraphs[0].text = "Convocatoria ficticia · Encabezado"
    sec.header.paragraphs[0].runs[0].font.bold = True
    sec.header.paragraphs[0].add_run().add_picture(logo_png("#1F4E78", (120, 40)), width=Inches(1.2))
    sec.footer.paragraphs[0].text = "Pie de página · {{institucion_nombre}}"
    sec.footer.paragraphs[0].alignment = WD_ALIGN_PARAGRAPH.CENTER

    st = d.styles["Normal"]
    st.font.name = "Calibri"
    st.font.size = Pt(11)
    d.add_heading("Proyecto: {{titulo_proyecto}}", level=1)

    d.add_heading("1. Justificación", level=2)
    d.add_paragraph("{{section:justificacion}}")

    d.add_heading("2. Población beneficiada", level=2)
    p = d.add_paragraph()
    r = p.add_run("Atendemos a ")
    r2 = p.add_run("{{beneficiarios")  # marcador partido en dos runs (caso difícil)
    r2.bold = True
    r2.font.color.rgb = RGBColor(0xC0, 0x00, 0x00)
    p.add_run("_total}}").bold = True
    p.add_run(" personas.")

    d.add_heading("3. Objetivo", level=2)
    d.add_paragraph("{{section:objetivo}}", style="List Bullet")

    d.add_heading("4. Presupuesto", level=2)
    d.add_paragraph("{{table:presupuesto}}")
    t = d.add_table(rows=2, cols=4)
    t.style = "Light Grid Accent 1"
    for i, h in enumerate(["Concepto", "Cantidad", "Precio", "Importe"]):
        t.cell(0, i).text = h
    t.cell(1, 0).text = "(ejemplo existente)"

    d.add_heading("5. Responsable", level=2)
    d.add_paragraph("Elaboró: {{institucion_nombre}}")
    d.core_properties.author = "Autor de prueba (debe limpiarse)"
    d.save("plantilla-prueba.docx")


def pdf():
    from reportlab.lib import colors
    from reportlab.lib.pagesizes import letter
    from reportlab.lib.styles import getSampleStyleSheet
    from reportlab.platypus import Paragraph, SimpleDocTemplate, Spacer, Table, TableStyle, PageBreak

    st = getSampleStyleSheet()
    doc = SimpleDocTemplate("convocatoria-con-tablas.pdf", pagesize=letter)
    t = Table(
        [["Concepto", "Monto máximo (MXN)", "Plazo (meses)"],
         ["Obra y equipamiento", "$300,000", "12"],
         ["Capacitación", "$50,000", "6"]]
    )
    t.setStyle(TableStyle([("GRID", (0, 0), (-1, -1), 0.5, colors.black),
                           ("BACKGROUND", (0, 0), (-1, 0), colors.lightgrey)]))
    doc.build([
        Paragraph("Convocatoria ficticia 2026", st["Title"]),
        Paragraph("Requisito REQ-01: el monto solicitado no puede exceder los límites de la tabla.", st["Normal"]),
        Spacer(1, 12), t, PageBreak(),
        Paragraph("Documentos requeridos: acta constitutiva, comprobante de domicilio, cotizaciones.", st["Normal"]),
    ])
    # "scanned" PDF: pages are only images, no text layer
    img = Image.new("RGB", (850, 1100), "white")
    d = ImageDraw.Draw(img)
    d.text((60, 80), "Convocatoria escaneada ficticia", fill="black")
    d.rectangle([60, 140, 700, 400], outline="black", width=2)
    img.save("convocatoria-escaneada.pdf", "PDF", resolution=100.0)


excel()
word()
pdf()
print("ok")
