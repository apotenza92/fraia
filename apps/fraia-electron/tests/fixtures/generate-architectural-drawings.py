from pathlib import Path

from reportlab.lib.pagesizes import A3, landscape
from reportlab.pdfgen import canvas
from pypdf import PdfReader, PdfWriter

OUTPUT = Path(__file__).with_name("architectural-drawings.pdf")
SCANNED_OUTPUT = Path(__file__).with_name("scanned-architectural-drawing.pdf")


def title_block(pdf: canvas.Canvas, title: str, number: str) -> None:
    width, height = landscape(A3)
    pdf.setLineWidth(1)
    pdf.rect(24, 24, width - 48, height - 48)
    pdf.line(24, 72, width - 24, 72)
    pdf.setFont("Helvetica-Bold", 18)
    pdf.drawString(42, 42, title)
    pdf.setFont("Helvetica", 11)
    pdf.drawRightString(width - 42, 42, f"Drawing {number} | Scale 1:100 | Fraia deterministic fixture")


def dimension(pdf: canvas.Canvas, x1: float, y: float, x2: float, label: str) -> None:
    pdf.line(x1, y, x2, y)
    pdf.line(x1, y - 6, x1, y + 6)
    pdf.line(x2, y - 6, x2, y + 6)
    pdf.setFont("Helvetica", 10)
    pdf.drawCentredString((x1 + x2) / 2, y + 8, label)


def plan(pdf: canvas.Canvas) -> None:
    title_block(pdf, "GROUND FLOOR STRUCTURAL PLAN", "A101")
    pdf.setLineWidth(2)
    pdf.rect(150, 180, 760, 470)
    for x in (150, 340, 530, 720, 910):
        pdf.line(x, 180, x, 650)
        pdf.circle(x, 180, 6)
        pdf.circle(x, 650, 6)
    pdf.setFont("Helvetica-Bold", 14)
    pdf.drawString(165, 620, "WORKSHOP STEEL FRAME PLAN")
    pdf.setFont("Helvetica", 11)
    pdf.drawString(165, 595, "Columns on grids 1-5. Main beams span north-south.")
    dimension(pdf, 150, 150, 910, "4 x 5 000 = 20 000 mm")
    pdf.drawString(930, 415, "NORTH")
    pdf.line(960, 380, 960, 470)
    pdf.line(960, 470, 950, 450)
    pdf.line(960, 470, 970, 450)


def elevation(pdf: canvas.Canvas) -> None:
    title_block(pdf, "NORTH FRAME ELEVATION", "A201")
    ground = 180
    for x in (160, 350, 540, 730, 920):
        pdf.line(x, ground, x, 570)
    pdf.line(160, 570, 920, 570)
    for x in (160, 350, 540, 730):
        pdf.line(x, ground, x + 190, 570)
        pdf.line(x + 190, ground, x, 570)
    pdf.setFont("Helvetica-Bold", 14)
    pdf.drawString(175, 610, "BRACED NORTH ELEVATION")
    pdf.setFont("Helvetica", 11)
    pdf.drawString(175, 590, "Indicative bracing bays. Confirm openings before modelling.")
    dimension(pdf, 160, 145, 920, "4 x 5 000 = 20 000 mm")
    dimension(pdf, 120, ground, 120, "")
    pdf.drawString(80, 380, "6 000 mm")


def section(pdf: canvas.Canvas) -> None:
    title_block(pdf, "TYPICAL PORTAL SECTION", "A301")
    pdf.setLineWidth(3)
    pdf.line(220, 180, 220, 500)
    pdf.line(860, 180, 860, 500)
    pdf.line(220, 500, 540, 650)
    pdf.line(540, 650, 860, 500)
    pdf.setLineWidth(1)
    pdf.line(160, 180, 920, 180)
    pdf.setFont("Helvetica-Bold", 14)
    pdf.drawString(235, 690, "TYPICAL TRANSVERSE PORTAL")
    pdf.setFont("Helvetica", 11)
    pdf.drawString(235, 670, "Roof pitch 25 degrees. Eaves and ridge levels require confirmation.")
    dimension(pdf, 220, 145, 860, "12 000 mm")
    pdf.drawString(875, 500, "EAVES 6 000")
    pdf.drawString(550, 650, "RIDGE")


def main() -> None:
    raw_output = OUTPUT.with_suffix(".raw.pdf")
    pdf = canvas.Canvas(str(raw_output), pagesize=landscape(A3), pageCompression=0)
    for draw in (plan, elevation, section):
        draw(pdf)
        pdf.showPage()
    pdf.save()
    reader = PdfReader(str(raw_output))
    if len(reader.pages) != 3:
        raise RuntimeError("fixture must contain exactly three pages")
    writer = PdfWriter()
    writer.append_pages_from_reader(reader)
    third = writer.pages[2]
    third.mediabox.lower_left = (20, 30)
    third.cropbox.lower_left = (40, 50)
    third.cropbox.upper_right = (1160, 810)
    third.rotate(90)
    with OUTPUT.open("wb") as stream:
        writer.write(stream)
    raw_output.unlink()

    scanned_raw = SCANNED_OUTPUT.with_suffix(".raw.pdf")
    scanned = canvas.Canvas(str(scanned_raw), pagesize=landscape(A3), pageCompression=0)
    scanned.drawImage(
        str(OUTPUT.with_name("ocr") / "scanned-plan.png"),
        0,
        0,
        width=landscape(A3)[0],
        height=landscape(A3)[1],
        preserveAspectRatio=False,
        mask=None,
    )
    scanned.showPage()
    scanned.save()
    scanned_reader = PdfReader(str(scanned_raw))
    scanned_writer = PdfWriter()
    scanned_writer.append_pages_from_reader(scanned_reader)
    scanned_writer.pages[0].rotate(90)
    with SCANNED_OUTPUT.open("wb") as stream:
        scanned_writer.write(stream)
    scanned_raw.unlink()


if __name__ == "__main__":
    main()
