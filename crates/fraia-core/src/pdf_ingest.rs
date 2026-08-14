use crate::{
    DrawingViewRole, SourceDerivative, SourceDerivativeKind, SourceDerivativeRequest, SourceId,
    SourceLibraryError, SourceLibraryPolicy, SourceMediaType, inspect_source,
    read_source_derivative, read_source_original, source_derivatives, store_source_derivative,
};
use lopdf::{Document, Object, ObjectId, content::Content};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, Instant};

pub const PDF_INDEX_SCHEMA_VERSION: &str = "fraia.pdf-index.v1";
pub const PDF_PARSER_ID: &str = "lopdf";
pub const PDF_PARSER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfDocumentIndex {
    pub schema_version: String,
    pub source_id: SourceId,
    pub source_sha256: String,
    pub parser: String,
    pub parser_version: String,
    pub page_count: u32,
    pub encrypted: bool,
    pub pages: Vec<PdfPageIndex>,
    #[serde(default)]
    pub drawing_register: Vec<PdfDrawingRegisterEntry>,
    #[serde(default)]
    pub callouts: Vec<PdfViewCallout>,
    pub extraction_method: PdfExtractionMethod,
    #[serde(default)]
    pub diagnostics: Vec<PdfDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageIndex {
    pub page_number: u32,
    pub media_box: PdfBox,
    pub crop_box: PdfBox,
    pub rotation_degrees: i16,
    pub user_unit: f64,
    pub coordinate_space: String,
    pub width_points: f64,
    pub height_points: f64,
    pub native_text: String,
    #[serde(default)]
    pub text_runs: Vec<PdfTextRun>,
    #[serde(default)]
    pub title_block: PdfTitleBlock,
    pub native_text_characters: usize,
    pub vector_path_operations: usize,
    pub embedded_image_count: usize,
    pub classification: PdfPageClassification,
    pub extraction_method: PdfExtractionMethod,
    /// Maps unrotated PDF user-space coordinates to the displayed page.
    pub source_to_display_transform: [f64; 6],
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfTitleBlock {
    pub sheet_number: Option<PdfTextField>,
    pub sheet_title: Option<PdfTextField>,
    pub discipline: Option<PdfTextField>,
    pub revision: Option<PdfTextField>,
    pub scale: Option<PdfTextField>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfTextField {
    pub value: String,
    pub page_number: u32,
    pub source_box: PdfBox,
    pub extraction_method: PdfExtractionMethod,
    pub parser: String,
    pub parser_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfDrawingRegisterEntry {
    pub sheet_number: PdfTextField,
    pub sheet_title: Option<PdfTextField>,
    pub register_page_number: u32,
    pub matched_page_number: Option<u32>,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfViewCallout {
    pub callout_id: String,
    pub source_page_number: u32,
    pub source_box: PdfBox,
    pub view_kind: DrawingViewRole,
    pub view_label: String,
    pub target_sheet_number: Option<String>,
    pub target_page_number: Option<u32>,
    pub confidence: f64,
    pub materially_conflicted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfTextRun {
    pub text: String,
    pub source_box: PdfBox,
    pub font_size: f64,
    pub extraction_method: PdfExtractionMethod,
    pub parser: String,
    pub parser_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfViewRoleEvidence {
    pub text: String,
    pub page_number: u32,
    pub source_box: PdfBox,
    pub evidence_kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfViewRoleSuggestion {
    pub inference_id: String,
    pub role: DrawingViewRole,
    pub confidence: f64,
    pub evidence: Vec<PdfViewRoleEvidence>,
    pub extraction_method: PdfExtractionMethod,
    pub parser: String,
    pub parser_version: String,
    pub materially_conflicted: bool,
    pub requires_question: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfViewRoleInference {
    pub source_id: SourceId,
    pub source_sha256: String,
    pub page_number: u32,
    pub crop: PdfBox,
    pub suggestions: Vec<PdfViewRoleSuggestion>,
    pub diagnostics: Vec<PdfDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfPageClassification {
    VectorText,
    Scanned,
    Mixed,
    VectorOnly,
    Blank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfExtractionMethod {
    NativePdfObjects,
    OcrUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfDiagnostic {
    pub code: PdfDiagnosticCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfDiagnosticCode {
    Encrypted,
    Corrupt,
    Oversized,
    PageLimit,
    DecompressionLimit,
    Timeout,
    Cancelled,
    RendererUnavailable,
    OcrUnavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfIndexPolicy {
    pub max_pdf_bytes: u64,
    pub max_pages: usize,
    pub max_decompressed_bytes_per_page: usize,
    pub max_native_text_characters_per_page: usize,
    pub max_operations_per_page: usize,
    pub max_millis: u64,
}

impl Default for PdfIndexPolicy {
    fn default() -> Self {
        Self {
            max_pdf_bytes: 256 * 1024 * 1024,
            max_pages: 10_000,
            max_decompressed_bytes_per_page: 64 * 1024 * 1024,
            max_native_text_characters_per_page: 2_000_000,
            max_operations_per_page: 5_000_000,
            max_millis: 120_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfIndexCheckpoint {
    Parsed,
    PageIndexed(u32),
    Complete,
}

#[derive(Debug)]
pub struct PdfIngestionError {
    pub diagnostic: PdfDiagnostic,
}
impl std::fmt::Display for PdfIngestionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.diagnostic.message)
    }
}
impl std::error::Error for PdfIngestionError {}

pub fn index_pdf(
    project_dir: &Path,
    source_id: &SourceId,
) -> Result<PdfDocumentIndex, PdfIngestionError> {
    index_pdf_with_policy_and_hook(project_dir, source_id, &PdfIndexPolicy::default(), |_| {
        Ok(())
    })
}

pub fn index_and_store_pdf(
    project_dir: &Path,
    source_id: &SourceId,
) -> Result<(PdfDocumentIndex, SourceDerivative, bool), PdfIngestionError> {
    for derivative in source_derivatives(project_dir, source_id).map_err(source_error)? {
        if derivative.kind == SourceDerivativeKind::VectorIndex
            && derivative.parser == PDF_PARSER_ID
            && derivative.parser_version == "0.44.0"
            && derivative.media_type == "application/vnd.fraia.pdf-index+json"
        {
            let (_, bytes) =
                read_source_derivative(project_dir, &derivative.id).map_err(source_error)?;
            let index: PdfDocumentIndex = serde_json::from_slice(&bytes).map_err(|error| {
                diagnostic(
                    PdfDiagnosticCode::Corrupt,
                    format!("stored PDF index is invalid: {error}"),
                )
            })?;
            if index.source_id == *source_id && index.source_sha256 == derivative.source_sha256 {
                return Ok((index, derivative, true));
            }
        }
    }
    let index = index_pdf(project_dir, source_id)?;
    let payload = serde_json::to_vec(&index).map_err(|error| {
        diagnostic(
            PdfDiagnosticCode::Corrupt,
            format!("serialize PDF index: {error}"),
        )
    })?;
    let derivative = store_source_derivative(
        project_dir,
        SourceDerivativeRequest {
            source_id: source_id.clone(),
            kind: SourceDerivativeKind::VectorIndex,
            payload,
            media_type: "application/vnd.fraia.pdf-index+json".into(),
            parser: PDF_PARSER_ID.into(),
            parser_version: "0.44.0".into(),
            units: None,
            coordinate_system: None,
            warnings: Vec::new(),
        },
        &SourceLibraryPolicy::default(),
    )
    .map_err(source_error)?;
    Ok((index, derivative, false))
}

pub fn index_pdf_with_policy_and_hook<F>(
    project_dir: &Path,
    source_id: &SourceId,
    policy: &PdfIndexPolicy,
    mut hook: F,
) -> Result<PdfDocumentIndex, PdfIngestionError>
where
    F: FnMut(PdfIndexCheckpoint) -> Result<(), PdfIngestionError>,
{
    let started = Instant::now();
    let record = inspect_source(project_dir, source_id).map_err(source_error)?;
    if record.detected_media_type != SourceMediaType::Pdf {
        return Err(diagnostic(
            PdfDiagnosticCode::Corrupt,
            "the selected source is not a PDF",
        ));
    }
    if record.byte_size > policy.max_pdf_bytes {
        return Err(diagnostic(
            PdfDiagnosticCode::Oversized,
            format!(
                "PDF size {} exceeds limit {}",
                record.byte_size, policy.max_pdf_bytes
            ),
        ));
    }
    let bytes = read_source_original(project_dir, source_id).map_err(source_error)?;
    let document = Document::load_mem(&bytes).map_err(|error| {
        diagnostic(
            PdfDiagnosticCode::Corrupt,
            format!("PDF parser rejected the document: {error}"),
        )
    })?;
    if document.is_encrypted() {
        return Err(diagnostic(
            PdfDiagnosticCode::Encrypted,
            "encrypted PDFs require an explicit password workflow and were not indexed",
        ));
    }
    hook(PdfIndexCheckpoint::Parsed)?;
    enforce_time(started, policy)?;
    let pages = document.get_pages();
    if pages.len() > policy.max_pages {
        return Err(diagnostic(
            PdfDiagnosticCode::PageLimit,
            format!(
                "PDF page count {} exceeds limit {}",
                pages.len(),
                policy.max_pages
            ),
        ));
    }
    let mut indexed_pages = Vec::with_capacity(pages.len());
    for (page_number, page_id) in pages {
        enforce_time(started, policy)?;
        let page = index_page(&document, page_number, page_id, policy)?;
        indexed_pages.push(page);
        hook(PdfIndexCheckpoint::PageIndexed(page_number))?;
    }
    hook(PdfIndexCheckpoint::Complete)?;
    let (drawing_register, callouts) = derive_package_text_evidence(&indexed_pages);
    Ok(PdfDocumentIndex {
        schema_version: PDF_INDEX_SCHEMA_VERSION.into(), source_id: source_id.clone(), source_sha256: record.sha256,
        parser: PDF_PARSER_ID.into(), parser_version: format!("lopdf-0.44.0/fraia-{PDF_PARSER_VERSION}"), page_count: indexed_pages.len() as u32,
        encrypted: false, pages: indexed_pages, drawing_register, callouts, extraction_method: PdfExtractionMethod::NativePdfObjects,
        diagnostics: vec![PdfDiagnostic { code: PdfDiagnosticCode::OcrUnavailable, message: "OCR is not integrated; scanned-page text remains unavailable and is never treated as authoritative.".into() }],
    })
}

pub fn pdf_renderer_unavailable_diagnostic() -> PdfDiagnostic {
    PdfDiagnostic { code: PdfDiagnosticCode::RendererUnavailable, message: "No reviewed packaged PDF renderer is configured. Development Poppler is opt-in and is not a release runtime.".into() }
}

fn index_page(
    document: &Document,
    page_number: u32,
    page_id: ObjectId,
    policy: &PdfIndexPolicy,
) -> Result<PdfPageIndex, PdfIngestionError> {
    let media_box = inherited_box(document, page_id, b"MediaBox").ok_or_else(|| {
        diagnostic(
            PdfDiagnosticCode::Corrupt,
            format!("PDF page {page_number} has no valid MediaBox"),
        )
    })?;
    let crop_box = inherited_box(document, page_id, b"CropBox").unwrap_or(media_box.clone());
    let rotation = inherited_integer(document, page_id, b"Rotate")
        .unwrap_or(0)
        .rem_euclid(360) as i16;
    if !matches!(rotation, 0 | 90 | 180 | 270) {
        return Err(diagnostic(
            PdfDiagnosticCode::Corrupt,
            format!("PDF page {page_number} has unsupported rotation {rotation}"),
        ));
    }
    let user_unit = inherited_number(document, page_id, b"UserUnit").unwrap_or(1.0);
    if !user_unit.is_finite() || user_unit <= 0.0 {
        return Err(diagnostic(
            PdfDiagnosticCode::Corrupt,
            format!("PDF page {page_number} has invalid UserUnit"),
        ));
    }
    let content_bytes = document.get_page_content_with_limit(page_id, policy.max_decompressed_bytes_per_page)
        .map_err(|error| diagnostic(PdfDiagnosticCode::DecompressionLimit, format!("PDF page {page_number} content exceeded the decompression limit or was invalid: {error}")))?;
    let content = Content::decode(&content_bytes).map_err(|error| {
        diagnostic(
            PdfDiagnosticCode::Corrupt,
            format!("PDF page {page_number} has invalid content operations: {error}"),
        )
    })?;
    if content.operations.len() > policy.max_operations_per_page {
        return Err(diagnostic(
            PdfDiagnosticCode::Oversized,
            format!(
                "PDF page {page_number} operation count exceeds limit {}",
                policy.max_operations_per_page
            ),
        ));
    }
    let vector_path_operations = content
        .operations
        .iter()
        .filter(|operation| {
            matches!(
                operation.operator.as_str(),
                "m" | "l"
                    | "c"
                    | "v"
                    | "y"
                    | "h"
                    | "re"
                    | "S"
                    | "s"
                    | "f"
                    | "F"
                    | "f*"
                    | "B"
                    | "B*"
                    | "b"
                    | "b*"
                    | "n"
            )
        })
        .count();
    let embedded_image_count = content
        .operations
        .iter()
        .filter(|operation| operation.operator == "Do")
        .count();
    let text_runs = extract_spatial_text_runs(&content, policy)?;
    let mut native_text = document
        .extract_text_with_limit(&[page_number], policy.max_decompressed_bytes_per_page)
        .unwrap_or_default();
    if native_text.chars().count() > policy.max_native_text_characters_per_page {
        native_text = native_text
            .chars()
            .take(policy.max_native_text_characters_per_page)
            .collect();
    }
    let native_text_characters = native_text
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    let classification = match (
        native_text_characters > 0,
        vector_path_operations > 0,
        embedded_image_count > 0,
    ) {
        (true, _, true) | (true, true, _) => PdfPageClassification::Mixed,
        (true, false, false) => PdfPageClassification::VectorText,
        (false, _, true) => PdfPageClassification::Scanned,
        (false, true, false) => PdfPageClassification::VectorOnly,
        (false, false, false) => PdfPageClassification::Blank,
    };
    let (width, height) = (crop_box.x1 - crop_box.x0, crop_box.y1 - crop_box.y0);
    let (width_points, height_points) = if matches!(rotation, 90 | 270) {
        (height, width)
    } else {
        (width, height)
    };
    let title_block = infer_title_block(page_number, &text_runs, &crop_box);
    Ok(PdfPageIndex {
        page_number,
        media_box,
        crop_box: crop_box.clone(),
        rotation_degrees: rotation,
        user_unit,
        coordinate_space: "pdf_user_space_points".into(),
        width_points,
        height_points,
        native_text,
        text_runs,
        title_block,
        native_text_characters,
        vector_path_operations,
        embedded_image_count,
        classification,
        extraction_method: if native_text_characters == 0 {
            PdfExtractionMethod::OcrUnavailable
        } else {
            PdfExtractionMethod::NativePdfObjects
        },
        source_to_display_transform: display_transform(&crop_box, rotation),
    })
}

pub fn infer_pdf_view_role(
    index: &PdfDocumentIndex,
    page_number: u32,
    crop: PdfBox,
    margin_points: f64,
) -> Result<PdfViewRoleInference, PdfIngestionError> {
    let page = index
        .pages
        .iter()
        .find(|page| page.page_number == page_number)
        .ok_or_else(|| {
            diagnostic(
                PdfDiagnosticCode::Corrupt,
                "PDF inference page was not indexed",
            )
        })?;
    if !valid_box(&crop) || !margin_points.is_finite() || margin_points < 0.0 {
        return Err(diagnostic(
            PdfDiagnosticCode::Corrupt,
            "PDF inference crop or margin is invalid",
        ));
    }
    if page.text_runs.is_empty() {
        return Ok(PdfViewRoleInference { source_id:index.source_id.clone(),source_sha256:index.source_sha256.clone(),page_number,crop,suggestions:Vec::new(),diagnostics:vec![PdfDiagnostic{code:PdfDiagnosticCode::OcrUnavailable,message:"No spatial native text is available. OCR remains unavailable; Fraia did not fabricate a view role.".into()}] });
    }
    let surrounding = PdfBox {
        x0: crop.x0 - margin_points,
        y0: crop.y0 - margin_points,
        x1: crop.x1 + margin_points,
        y1: crop.y1 + margin_points,
    };
    let mut scores = std::collections::BTreeMap::<
        String,
        (DrawingViewRole, f64, Vec<PdfViewRoleEvidence>),
    >::new();
    for run in &page.text_runs {
        if !intersects(&run.source_box, &surrounding) {
            continue;
        }
        let inside = intersects(&run.source_box, &crop);
        let upper = run.text.to_ascii_uppercase();
        let candidates = [
            (
                DrawingViewRole::Plan,
                "plan",
                ["PLAN", "FLOOR PLAN", "ROOF PLAN"].as_slice(),
            ),
            (
                DrawingViewRole::Elevation,
                "elevation",
                ["ELEVATION", "ELEV."].as_slice(),
            ),
            (
                DrawingViewRole::Section,
                "section",
                ["SECTION", "SEC."].as_slice(),
            ),
            (
                DrawingViewRole::Detail,
                "detail",
                ["DETAIL", "DET."].as_slice(),
            ),
            (
                DrawingViewRole::Schedule,
                "schedule",
                ["SCHEDULE"].as_slice(),
            ),
        ];
        for (role, key, terms) in candidates {
            if terms.iter().any(|term| upper.contains(term)) {
                let entry = scores.entry(key.into()).or_insert((role, 0.0, Vec::new()));
                entry.1 += if inside { 0.75 } else { 0.35 };
                entry.2.push(PdfViewRoleEvidence {
                    text: run.text.clone(),
                    page_number,
                    source_box: run.source_box.clone(),
                    evidence_kind: if inside {
                        "crop_text".into()
                    } else {
                        "surrounding_text".into()
                    },
                });
            }
        }
    }
    let mut suggestions = scores
        .into_values()
        .map(|(role, score, evidence)| PdfViewRoleSuggestion {
            inference_id: format!(
                "pdf-view:{}:{}:{:?}",
                index.source_sha256, page_number, role
            )
            .to_ascii_lowercase(),
            role,
            confidence: score.min(0.99),
            evidence,
            extraction_method: PdfExtractionMethod::NativePdfObjects,
            parser: index.parser.clone(),
            parser_version: index.parser_version.clone(),
            materially_conflicted: false,
            requires_question: false,
        })
        .collect::<Vec<_>>();
    suggestions.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then_with(|| format!("{:?}", a.role).cmp(&format!("{:?}", b.role)))
    });
    if suggestions.len() > 1 && (suggestions[0].confidence - suggestions[1].confidence).abs() < 0.2
    {
        for suggestion in &mut suggestions {
            suggestion.materially_conflicted = true;
            suggestion.requires_question = true;
        }
    }
    if let Some(first) = suggestions.first_mut() {
        if first.confidence < 0.7 {
            first.requires_question = true;
        }
    }
    Ok(PdfViewRoleInference {
        source_id: index.source_id.clone(),
        source_sha256: index.source_sha256.clone(),
        page_number,
        crop,
        suggestions,
        diagnostics: Vec::new(),
    })
}

fn extract_spatial_text_runs(
    content: &Content,
    policy: &PdfIndexPolicy,
) -> Result<Vec<PdfTextRun>, PdfIngestionError> {
    let mut runs = Vec::new();
    let mut text_matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut current_transform = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut transform_stack = Vec::new();
    let mut font_size = 12.0;
    for operation in &content.operations {
        match operation.operator.as_str() {
            "q" => transform_stack.push(current_transform),
            "Q" => {
                current_transform = transform_stack
                    .pop()
                    .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
            }
            "cm" => {
                if let Some(matrix) = operation_matrix(&operation.operands) {
                    current_transform = multiply_affine(current_transform, matrix);
                }
            }
            "Tf" => {
                if let Some(value) = operation.operands.get(1).and_then(number) {
                    if value.is_finite() && value > 0.0 {
                        font_size = value;
                    }
                }
            }
            "Tm" => {
                if let Some(matrix) = operation_matrix(&operation.operands) {
                    text_matrix = matrix;
                }
            }
            "Td" | "TD" => {
                if operation.operands.len() >= 2 {
                    text_matrix[4] += number(&operation.operands[0]).unwrap_or(0.0);
                    text_matrix[5] += number(&operation.operands[1]).unwrap_or(0.0);
                }
            }
            "Tj" | "'" | "\"" => {
                if let Some(text) = operation.operands.last().and_then(pdf_string) {
                    push_text_run(
                        &mut runs,
                        text,
                        multiply_affine(current_transform, text_matrix),
                        font_size,
                        policy,
                    )?;
                }
            }
            "TJ" => {
                if let Some(Object::Array(items)) = operation.operands.first() {
                    let text = items.iter().filter_map(pdf_string).collect::<String>();
                    push_text_run(
                        &mut runs,
                        text,
                        multiply_affine(current_transform, text_matrix),
                        font_size,
                        policy,
                    )?;
                }
            }
            _ => {}
        }
    }
    Ok(runs)
}
fn push_text_run(
    runs: &mut Vec<PdfTextRun>,
    text: String,
    matrix: [f64; 6],
    font_size: f64,
    policy: &PdfIndexPolicy,
) -> Result<(), PdfIngestionError> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Ok(());
    }
    if runs.len() >= policy.max_operations_per_page {
        return Err(diagnostic(
            PdfDiagnosticCode::Oversized,
            "PDF spatial text run count exceeds operation limit",
        ));
    }
    let width = (text.chars().count() as f64 * font_size * 0.55).max(font_size * 0.25);
    let corners = [
        [0.0, 0.0],
        [width, 0.0],
        [0.0, font_size],
        [width, font_size],
    ]
    .map(|point| transform_point(matrix, point));
    let x0 = corners
        .iter()
        .map(|point| point[0])
        .fold(f64::INFINITY, f64::min);
    let x1 = corners
        .iter()
        .map(|point| point[0])
        .fold(f64::NEG_INFINITY, f64::max);
    let y0 = corners
        .iter()
        .map(|point| point[1])
        .fold(f64::INFINITY, f64::min);
    let y1 = corners
        .iter()
        .map(|point| point[1])
        .fold(f64::NEG_INFINITY, f64::max);
    runs.push(PdfTextRun {
        text,
        source_box: PdfBox { x0, y0, x1, y1 },
        font_size,
        extraction_method: PdfExtractionMethod::NativePdfObjects,
        parser: PDF_PARSER_ID.into(),
        parser_version: format!("lopdf-0.44.0/fraia-{PDF_PARSER_VERSION}"),
    });
    Ok(())
}
fn operation_matrix(values: &[Object]) -> Option<[f64; 6]> {
    if values.len() < 6 {
        return None;
    }
    Some([
        number(&values[0])?,
        number(&values[1])?,
        number(&values[2])?,
        number(&values[3])?,
        number(&values[4])?,
        number(&values[5])?,
    ])
}
fn multiply_affine(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}
fn transform_point(m: [f64; 6], p: [f64; 2]) -> [f64; 2] {
    [
        m[0] * p[0] + m[2] * p[1] + m[4],
        m[1] * p[0] + m[3] * p[1] + m[5],
    ]
}
fn pdf_string(object: &Object) -> Option<String> {
    match object {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).into_owned()),
        _ => None,
    }
}
fn valid_box(b: &PdfBox) -> bool {
    [b.x0, b.y0, b.x1, b.y1].into_iter().all(f64::is_finite) && b.x1 > b.x0 && b.y1 > b.y0
}
fn intersects(a: &PdfBox, b: &PdfBox) -> bool {
    a.x1 >= b.x0 && a.x0 <= b.x1 && a.y1 >= b.y0 && a.y0 <= b.y1
}

fn text_field(page: u32, run: &PdfTextRun, value: String) -> PdfTextField {
    PdfTextField {
        value,
        page_number: page,
        source_box: run.source_box.clone(),
        extraction_method: run.extraction_method,
        parser: run.parser.clone(),
        parser_version: run.parser_version.clone(),
    }
}
fn infer_title_block(page: u32, runs: &[PdfTextRun], crop: &PdfBox) -> PdfTitleBlock {
    let mut result = PdfTitleBlock::default();
    let title_zone_x = crop.x0 + (crop.x1 - crop.x0) * 0.55;
    let title_zone_y = crop.y0 + (crop.y1 - crop.y0) * 0.35;
    for run in runs
        .iter()
        .filter(|run| run.source_box.x0 >= title_zone_x && run.source_box.y0 <= title_zone_y)
    {
        let upper = run.text.to_ascii_uppercase();
        if result.scale.is_none() && (upper.contains("SCALE") || looks_like_scale(&upper)) {
            result.scale = Some(text_field(page, run, run.text.clone()));
        } else if result.revision.is_none()
            && (upper.starts_with("REV") || upper.starts_with("REVISION"))
        {
            result.revision = Some(text_field(page, run, run.text.clone()));
        } else if result.discipline.is_none()
            && ["STRUCTURAL", "ARCHITECTURAL", "CIVIL", "MECHANICAL"]
                .iter()
                .any(|term| upper.contains(term))
        {
            result.discipline = Some(text_field(page, run, run.text.clone()));
        } else if result.sheet_number.is_none() && looks_like_sheet_number(&upper) {
            result.sheet_number = Some(text_field(page, run, run.text.clone()));
        } else if result.sheet_title.is_none() && run.text.len() >= 4 {
            result.sheet_title = Some(text_field(page, run, run.text.clone()));
        }
    }
    result
}
fn derive_package_text_evidence(
    pages: &[PdfPageIndex],
) -> (Vec<PdfDrawingRegisterEntry>, Vec<PdfViewCallout>) {
    let sheet_pages = pages
        .iter()
        .filter_map(|page| {
            page.title_block
                .sheet_number
                .as_ref()
                .map(|field| (field.value.to_ascii_uppercase(), page.page_number))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut register = Vec::new();
    let mut callouts = Vec::new();
    for page in pages {
        let register_page = page.text_runs.iter().any(|run| {
            let u = run.text.to_ascii_uppercase();
            u.contains("DRAWING REGISTER")
                || u.contains("DRAWING LIST")
                || u.contains("SHEET INDEX")
        });
        for (position, run) in page.text_runs.iter().enumerate() {
            let upper = run.text.to_ascii_uppercase();
            if register_page && looks_like_sheet_number(&upper) {
                let title = page
                    .text_runs
                    .get(position + 1)
                    .filter(|next| next.source_box.y0 - run.source_box.y0 < 20.0)
                    .map(|next| text_field(page.page_number, next, next.text.clone()));
                register.push(PdfDrawingRegisterEntry {
                    sheet_number: text_field(page.page_number, run, upper.clone()),
                    sheet_title: title,
                    register_page_number: page.page_number,
                    matched_page_number: sheet_pages.get(&upper).copied(),
                    confidence: if sheet_pages.contains_key(&upper) {
                        0.95
                    } else {
                        0.75
                    },
                });
            }
            if let Some((role, label, target)) = parse_callout(&upper) {
                let target_page_number = target
                    .as_ref()
                    .and_then(|sheet| sheet_pages.get(sheet).copied());
                callouts.push(PdfViewCallout {
                    callout_id: format!("pdf-callout:{}:{}", page.page_number, position),
                    source_page_number: page.page_number,
                    source_box: run.source_box.clone(),
                    view_kind: role,
                    view_label: label,
                    target_sheet_number: target,
                    target_page_number,
                    confidence: if target_page_number.is_some() {
                        0.95
                    } else {
                        0.8
                    },
                    materially_conflicted: false,
                });
            }
        }
    }
    (register, callouts)
}
fn looks_like_scale(v: &str) -> bool {
    v.split_whitespace().any(|token| {
        let parts = token.split(':').collect::<Vec<_>>();
        parts.len() == 2 && parts.iter().all(|part| part.trim().parse::<u32>().is_ok())
    })
}
fn looks_like_sheet_number(v: &str) -> bool {
    let v = v.trim();
    v.len() >= 2
        && v.len() <= 16
        && v.chars().any(|c| c.is_ascii_alphabetic())
        && v.chars().any(|c| c.is_ascii_digit())
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
}
fn parse_callout(v: &str) -> Option<(DrawingViewRole, String, Option<String>)> {
    let words = v
        .split(|c: char| c.is_whitespace() || c == '/')
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let role = if words.contains(&"SECTION") {
        DrawingViewRole::Section
    } else if words.contains(&"DETAIL") {
        DrawingViewRole::Detail
    } else {
        return None;
    };
    let tokens = words;
    let label = tokens
        .iter()
        .skip_while(|token| {
            !token.contains(if role == DrawingViewRole::Section {
                "SECTION"
            } else {
                "DETAIL"
            })
        })
        .nth(1)
        .copied()
        .unwrap_or("UNLABELLED")
        .to_string();
    let target = tokens
        .iter()
        .rev()
        .find(|token| looks_like_sheet_number(token))
        .map(|v| v.to_string());
    Some((role, label, target))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfBox {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

fn inherited_box(document: &Document, mut id: ObjectId, key: &[u8]) -> Option<PdfBox> {
    loop {
        let dictionary = document.get_dictionary(id).ok()?;
        if let Ok(value) = dictionary.get(key) {
            let value = resolve(document, value);
            let array = value.as_array().ok()?;
            if array.len() == 4 {
                let values = array.iter().map(number).collect::<Option<Vec<_>>>()?;
                if values.iter().all(|value| value.is_finite())
                    && values[2] > values[0]
                    && values[3] > values[1]
                {
                    return Some(PdfBox {
                        x0: values[0],
                        y0: values[1],
                        x1: values[2],
                        y1: values[3],
                    });
                }
            }
            return None;
        }
        id = dictionary.get(b"Parent").ok()?.as_reference().ok()?;
    }
}
fn inherited_integer(document: &Document, mut id: ObjectId, key: &[u8]) -> Option<i64> {
    loop {
        let dictionary = document.get_dictionary(id).ok()?;
        if let Ok(value) = dictionary.get(key) {
            return resolve(document, value).as_i64().ok();
        }
        id = dictionary.get(b"Parent").ok()?.as_reference().ok()?;
    }
}
fn inherited_number(document: &Document, mut id: ObjectId, key: &[u8]) -> Option<f64> {
    loop {
        let dictionary = document.get_dictionary(id).ok()?;
        if let Ok(value) = dictionary.get(key) {
            return number(resolve(document, value));
        }
        id = dictionary.get(b"Parent").ok()?.as_reference().ok()?;
    }
}
fn resolve<'a>(document: &'a Document, object: &'a Object) -> &'a Object {
    object
        .as_reference()
        .ok()
        .and_then(|id| document.get_object(id).ok())
        .unwrap_or(object)
}
fn number(object: &Object) -> Option<f64> {
    match object {
        Object::Integer(value) => Some(*value as f64),
        Object::Real(value) => Some(*value as f64),
        _ => None,
    }
}
fn display_transform(crop: &PdfBox, rotation: i16) -> [f64; 6] {
    match rotation {
        0 => [1.0, 0.0, 0.0, 1.0, -crop.x0, -crop.y0],
        90 => [0.0, 1.0, -1.0, 0.0, crop.y1, -crop.x0],
        180 => [-1.0, 0.0, 0.0, -1.0, crop.x1, crop.y1],
        270 => [0.0, -1.0, 1.0, 0.0, -crop.y0, crop.x1],
        _ => unreachable!(),
    }
}
fn enforce_time(started: Instant, policy: &PdfIndexPolicy) -> Result<(), PdfIngestionError> {
    if started.elapsed() > Duration::from_millis(policy.max_millis) {
        Err(diagnostic(
            PdfDiagnosticCode::Timeout,
            "PDF indexing exceeded the configured time limit",
        ))
    } else {
        Ok(())
    }
}
fn source_error(error: SourceLibraryError) -> PdfIngestionError {
    diagnostic(PdfDiagnosticCode::Corrupt, error.to_string())
}
fn diagnostic(code: PdfDiagnosticCode, message: impl Into<String>) -> PdfIngestionError {
    PdfIngestionError {
        diagnostic: PdfDiagnostic {
            code,
            message: message.into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceImportRequest, create_named_project_package, import_source};
    use lopdf::{Dictionary, Stream, dictionary};
    use std::fs;

    struct Fixture {
        root: std::path::PathBuf,
        project: std::path::PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "fraia-pdf-index-{}-{}",
                std::process::id(),
                std::thread::current().name().unwrap_or("test")
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir(&root).expect("create fixture root");
            let project = root.join("project");
            create_named_project_package(&project, "PDF fixture").expect("create project");
            Self { root, project }
        }
        fn import(&self, name: &str, bytes: &[u8]) -> SourceId {
            let path = self.root.join(name);
            fs::write(&path, bytes).expect("write PDF fixture");
            import_source(
                &self.project,
                SourceImportRequest {
                    selected_path: path,
                    display_alias: None,
                    expected_media_type: Some(SourceMediaType::Pdf),
                },
            )
            .expect("import PDF")
            .record
            .id
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn multipage_pdf() -> Vec<u8> {
        let mut document = Document::with_version("1.7");
        let pages_id = document.new_object_id();
        let font_id = document.add_object(
            dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica" },
        );
        let image_id = document.add_object(Stream::new(dictionary! { "Type" => "XObject", "Subtype" => "Image", "Width" => 2, "Height" => 2, "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8 }, vec![0, 64, 128, 255]));
        let definitions = vec![
            (
                vec![
                    lopdf::content::Operation::new("m", vec![20.into(), 30.into()]),
                    lopdf::content::Operation::new("l", vec![300.into(), 30.into()]),
                    lopdf::content::Operation::new("S", vec![]),
                    lopdf::content::Operation::new("BT", vec![]),
                    lopdf::content::Operation::new(
                        "Tf",
                        vec![Object::Name(b"F1".to_vec()), 12.into()],
                    ),
                    lopdf::content::Operation::new(
                        "Tm",
                        vec![
                            1.into(),
                            0.into(),
                            0.into(),
                            1.into(),
                            50.into(),
                            300.into(),
                        ],
                    ),
                    lopdf::content::Operation::new(
                        "Tj",
                        vec![Object::string_literal("PLAN LEVEL 1")],
                    ),
                    lopdf::content::Operation::new("ET", vec![]),
                ],
                0,
            ),
            (
                vec![lopdf::content::Operation::new(
                    "Do",
                    vec![Object::Name(b"Im1".to_vec())],
                )],
                0,
            ),
            (
                vec![
                    lopdf::content::Operation::new("BT", vec![]),
                    lopdf::content::Operation::new(
                        "Tf",
                        vec![Object::Name(b"F1".to_vec()), 12.into()],
                    ),
                    lopdf::content::Operation::new(
                        "Tm",
                        vec![
                            1.into(),
                            0.into(),
                            0.into(),
                            1.into(),
                            60.into(),
                            250.into(),
                        ],
                    ),
                    lopdf::content::Operation::new(
                        "Tj",
                        vec![Object::string_literal("ROTATED ELEVATION")],
                    ),
                    lopdf::content::Operation::new("ET", vec![]),
                    lopdf::content::Operation::new("Do", vec![Object::Name(b"Im1".to_vec())]),
                ],
                90,
            ),
        ];
        let mut page_ids = Vec::new();
        for (operations, rotation) in definitions {
            let content_id = document.add_object(Stream::new(
                Dictionary::new(),
                Content { operations }.encode().expect("encode content"),
            ));
            let page_id = document.add_object(dictionary! {
                "Type" => "Page", "Parent" => pages_id,
                "MediaBox" => vec![10.into(), 20.into(), 610.into(), 420.into()],
                "CropBox" => vec![30.into(), 40.into(), 590.into(), 400.into()],
                "Rotate" => rotation, "UserUnit" => 2,
                "Resources" => dictionary! { "Font" => dictionary! { "F1" => font_id }, "XObject" => dictionary! { "Im1" => image_id } },
                "Contents" => content_id,
            });
            page_ids.push(page_id);
        }
        document.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => page_ids.iter().copied().map(Object::Reference).collect::<Vec<_>>(), "Count" => page_ids.len() as i64 }));
        let catalog_id =
            document.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        document.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        document
            .save_to(&mut bytes)
            .expect("save deterministic PDF");
        bytes
    }

    #[test]
    fn indexes_vector_scanned_mixed_rotated_pages_and_exact_source_coordinates() {
        let fixture = Fixture::new();
        let source_id = fixture.import("drawings.pdf", &multipage_pdf());
        let index = index_pdf(&fixture.project, &source_id).expect("index PDF");
        assert_eq!(index.page_count, 3);
        assert_eq!(index.pages[0].classification, PdfPageClassification::Mixed);
        assert!(index.pages[0].native_text.contains("PLAN LEVEL 1"));
        assert_eq!(index.pages[0].text_runs[0].text, "PLAN LEVEL 1");
        assert_eq!(index.pages[0].text_runs[0].source_box.x0, 50.0);
        assert_eq!(
            index.pages[1].classification,
            PdfPageClassification::Scanned
        );
        assert_eq!(
            index.pages[1].extraction_method,
            PdfExtractionMethod::OcrUnavailable
        );
        assert_eq!(index.pages[2].classification, PdfPageClassification::Mixed);
        assert_eq!(index.pages[2].rotation_degrees, 90);
        assert_eq!(index.pages[2].user_unit, 2.0);
        assert_eq!(index.pages[2].coordinate_space, "pdf_user_space_points");
        assert_eq!(
            index.pages[2].crop_box,
            PdfBox {
                x0: 30.0,
                y0: 40.0,
                x1: 590.0,
                y1: 400.0
            }
        );
        assert_eq!(index.pages[2].width_points, 360.0);
        assert_eq!(index.pages[2].height_points, 560.0);
        assert_eq!(
            index.pages[2].source_to_display_transform,
            [0.0, 1.0, -1.0, 0.0, 400.0, -30.0]
        );
    }

    #[test]
    fn indexes_product_scanned_pdf_fixture_without_fabricating_native_text() {
        let fixture = Fixture::new();
        let bytes = include_bytes!(
            "../../../apps/fraia-electron/tests/fixtures/scanned-architectural-drawing.pdf"
        );
        let source_id = fixture.import("scanned-architectural-drawing.pdf", bytes);
        let index = index_pdf(&fixture.project, &source_id).expect("index scanned PDF");
        assert_eq!(index.page_count, 1);
        assert_eq!(
            index.pages[0].classification,
            PdfPageClassification::Scanned
        );
        assert_eq!(index.pages[0].native_text_characters, 0);
        assert!(index.pages[0].native_text.is_empty());
        assert!(index.pages[0].text_runs.is_empty());
        assert_eq!(index.pages[0].embedded_image_count, 1);
        assert_eq!(index.pages[0].rotation_degrees, 90);
        assert_eq!(index.pages[0].width_points, 800.0);
        assert_eq!(index.pages[0].height_points, 1200.0);
        assert_eq!(
            index.pages[0].extraction_method,
            PdfExtractionMethod::OcrUnavailable
        );
    }

    #[test]
    fn spatial_text_ranks_crop_role_and_conflicts_without_fabricating_scanned_text() {
        let fixture = Fixture::new();
        let source_id = fixture.import("drawings.pdf", &multipage_pdf());
        let mut index = index_pdf(&fixture.project, &source_id).expect("index PDF");
        let plan = infer_pdf_view_role(
            &index,
            1,
            PdfBox {
                x0: 40.0,
                y0: 280.0,
                x1: 180.0,
                y1: 330.0,
            },
            20.0,
        )
        .expect("infer plan");
        assert_eq!(plan.suggestions[0].role, DrawingViewRole::Plan);
        assert!(plan.suggestions[0].confidence >= 0.7);
        assert!(!plan.suggestions[0].requires_question);
        assert_eq!(plan.suggestions[0].evidence[0].source_box.x0, 50.0);

        index.pages[0].text_runs.push(PdfTextRun {
            text: "SECTION A-A".into(),
            source_box: PdfBox {
                x0: 55.0,
                y0: 290.0,
                x1: 145.0,
                y1: 302.0,
            },
            font_size: 12.0,
            extraction_method: PdfExtractionMethod::NativePdfObjects,
            parser: PDF_PARSER_ID.into(),
            parser_version: PDF_PARSER_VERSION.into(),
        });
        let conflict = infer_pdf_view_role(&index, 1, plan.crop.clone(), 20.0).unwrap();
        assert!(
            conflict
                .suggestions
                .iter()
                .all(|suggestion| suggestion.materially_conflicted)
        );
        assert!(
            conflict
                .suggestions
                .iter()
                .all(|suggestion| suggestion.requires_question)
        );

        let scanned =
            infer_pdf_view_role(&index, 2, index.pages[1].crop_box.clone(), 20.0).unwrap();
        assert!(scanned.suggestions.is_empty());
        assert_eq!(
            scanned.diagnostics[0].code,
            PdfDiagnosticCode::OcrUnavailable
        );
    }

    #[test]
    fn title_block_register_and_cross_sheet_callouts_keep_exact_boxes() {
        let run = |text: &str, x: f64, y: f64| PdfTextRun {
            text: text.into(),
            source_box: PdfBox {
                x0: x,
                y0: y,
                x1: x + 80.0,
                y1: y + 12.0,
            },
            font_size: 12.0,
            extraction_method: PdfExtractionMethod::NativePdfObjects,
            parser: PDF_PARSER_ID.into(),
            parser_version: PDF_PARSER_VERSION.into(),
        };
        let crop = PdfBox {
            x0: 10.0,
            y0: 20.0,
            x1: 610.0,
            y1: 420.0,
        };
        let title = infer_title_block(
            2,
            &[
                run("S201", 500.0, 40.0),
                run("STRUCTURAL SECTIONS", 500.0, 65.0),
                run("SCALE 1:100", 500.0, 90.0),
            ],
            &crop,
        );
        assert_eq!(title.sheet_number.as_ref().unwrap().value, "S201");
        assert_eq!(title.sheet_number.as_ref().unwrap().source_box.x0, 500.0);
        assert_eq!(title.scale.as_ref().unwrap().value, "SCALE 1:100");
        let page = |number, runs: Vec<PdfTextRun>, title_block: PdfTitleBlock| PdfPageIndex {
            page_number: number,
            media_box: crop.clone(),
            crop_box: crop.clone(),
            rotation_degrees: 0,
            user_unit: 1.0,
            coordinate_space: "pdf_user_space_points".into(),
            width_points: 600.0,
            height_points: 400.0,
            native_text: String::new(),
            text_runs: runs,
            title_block,
            native_text_characters: 1,
            vector_path_operations: 0,
            embedded_image_count: 0,
            classification: PdfPageClassification::VectorText,
            extraction_method: PdfExtractionMethod::NativePdfObjects,
            source_to_display_transform: [1.0, 0.0, 0.0, 1.0, -10.0, -20.0],
        };
        let register_page = page(
            1,
            vec![
                run("DRAWING REGISTER", 40.0, 380.0),
                run("S201", 40.0, 350.0),
                run("STRUCTURAL SECTIONS", 130.0, 350.0),
            ],
            PdfTitleBlock::default(),
        );
        let sheet_page = page(2, vec![run("SECTION A-A / S201", 100.0, 200.0)], title);
        let (register, callouts) = derive_package_text_evidence(&[register_page, sheet_page]);
        assert_eq!(register[0].matched_page_number, Some(2));
        assert_eq!(register[0].sheet_number.source_box.x0, 40.0);
        assert_eq!(callouts[0].target_page_number, Some(2));
        assert_eq!(callouts[0].view_kind, DrawingViewRole::Section);
        assert_eq!(callouts[0].source_box.x0, 100.0);
    }

    #[test]
    fn spatial_text_composes_rotated_text_matrix_and_non_zero_graphics_origin() {
        let content = Content {
            operations: vec![
                lopdf::content::Operation::new("q", vec![]),
                lopdf::content::Operation::new(
                    "cm",
                    vec![
                        1.into(),
                        0.into(),
                        0.into(),
                        1.into(),
                        100.into(),
                        200.into(),
                    ],
                ),
                lopdf::content::Operation::new("BT", vec![]),
                lopdf::content::Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 10.into()]),
                lopdf::content::Operation::new(
                    "Tm",
                    vec![
                        0.into(),
                        1.into(),
                        (-1).into(),
                        0.into(),
                        20.into(),
                        30.into(),
                    ],
                ),
                lopdf::content::Operation::new("Tj", vec![Object::string_literal("SECTION A-A")]),
                lopdf::content::Operation::new("ET", vec![]),
                lopdf::content::Operation::new("Q", vec![]),
            ],
        };
        let runs = extract_spatial_text_runs(&content, &PdfIndexPolicy::default()).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].source_box.x1, 120.0);
        assert_eq!(runs[0].source_box.y0, 230.0);
        assert!(runs[0].source_box.y1 > runs[0].source_box.y0);
    }

    #[test]
    fn persisted_index_resumes_and_retains_source_parser_and_page_identity() {
        let fixture = Fixture::new();
        let source_id = fixture.import("drawings.pdf", &multipage_pdf());
        let (first, derivative, resumed) =
            index_and_store_pdf(&fixture.project, &source_id).expect("index and store");
        assert!(!resumed);
        let (second, second_derivative, resumed) =
            index_and_store_pdf(&fixture.project, &source_id).expect("resume stored index");
        assert!(resumed);
        assert_eq!(first, second);
        assert_eq!(derivative, second_derivative);
        assert_eq!(first.source_sha256, derivative.source_sha256);
        assert_eq!(derivative.parser, PDF_PARSER_ID);
        assert_eq!(derivative.parser_version, "0.44.0");
    }

    #[test]
    fn fails_closed_for_corrupt_oversized_page_limited_and_cancelled_inputs() {
        let fixture = Fixture::new();
        let corrupt = fixture.import("corrupt.pdf", b"%PDF-1.7\nnot a document\n%%EOF\n");
        assert_eq!(
            index_pdf(&fixture.project, &corrupt)
                .expect_err("reject corrupt")
                .diagnostic
                .code,
            PdfDiagnosticCode::Corrupt
        );
        let source_id = fixture.import("drawings.pdf", &multipage_pdf());
        let oversized = PdfIndexPolicy {
            max_pdf_bytes: 4,
            ..PdfIndexPolicy::default()
        };
        assert_eq!(
            index_pdf_with_policy_and_hook(&fixture.project, &source_id, &oversized, |_| Ok(()))
                .expect_err("reject oversize")
                .diagnostic
                .code,
            PdfDiagnosticCode::Oversized
        );
        let limited = PdfIndexPolicy {
            max_pages: 2,
            ..PdfIndexPolicy::default()
        };
        assert_eq!(
            index_pdf_with_policy_and_hook(&fixture.project, &source_id, &limited, |_| Ok(()))
                .expect_err("reject page count")
                .diagnostic
                .code,
            PdfDiagnosticCode::PageLimit
        );
        let cancelled = index_pdf_with_policy_and_hook(
            &fixture.project,
            &source_id,
            &PdfIndexPolicy::default(),
            |checkpoint| {
                if checkpoint == PdfIndexCheckpoint::PageIndexed(1) {
                    Err(diagnostic(
                        PdfDiagnosticCode::Cancelled,
                        "cancelled by caller",
                    ))
                } else {
                    Ok(())
                }
            },
        )
        .expect_err("cancel indexing");
        assert_eq!(cancelled.diagnostic.code, PdfDiagnosticCode::Cancelled);
        assert!(
            source_derivatives(&fixture.project, &source_id)
                .expect("no partial derivatives")
                .is_empty()
        );
    }
}
