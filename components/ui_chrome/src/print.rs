//! Print support for the browser UI
//!
//! Provides print preview, print settings, print job tracking, and system print dialog integration.
//! Supports various paper sizes, orientations, and print-to-PDF functionality.
//!
//! # Example
//!
//! ```rust,ignore
//! use ui_chrome::print::{PrintManager, PrintSettings, PaperSize, Orientation};
//!
//! // Create print manager
//! let mut print_manager = PrintManager::new();
//!
//! // Configure print settings
//! let settings = PrintSettings::default()
//!     .with_paper_size(PaperSize::A4)
//!     .with_orientation(Orientation::Portrait)
//!     .with_scale(100);
//!
//! // Show print preview
//! print_manager.show_preview(&settings);
//! ```

use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Standard paper sizes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum PaperSize {
    /// A4 paper (210mm x 297mm)
    #[default]
    A4,
    /// US Letter (8.5" x 11")
    Letter,
    /// US Legal (8.5" x 14")
    Legal,
    /// A3 paper (297mm x 420mm)
    A3,
    /// A5 paper (148mm x 210mm)
    A5,
    /// Tabloid (11" x 17")
    Tabloid,
    /// Custom size with width and height in millimeters
    Custom { width_mm: f32, height_mm: f32 },
}

impl PaperSize {
    /// Get the paper dimensions in millimeters (width, height)
    pub fn dimensions_mm(&self) -> (f32, f32) {
        match self {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::Letter => (215.9, 279.4), // 8.5" x 11"
            PaperSize::Legal => (215.9, 355.6),  // 8.5" x 14"
            PaperSize::A3 => (297.0, 420.0),
            PaperSize::A5 => (148.0, 210.0),
            PaperSize::Tabloid => (279.4, 431.8), // 11" x 17"
            PaperSize::Custom { width_mm, height_mm } => (*width_mm, *height_mm),
        }
    }

    /// Get the paper dimensions in points (1 point = 1/72 inch)
    pub fn dimensions_points(&self) -> (f32, f32) {
        let (w_mm, h_mm) = self.dimensions_mm();
        // Convert mm to points: mm * 72 / 25.4
        let mm_to_pt = 72.0 / 25.4;
        (w_mm * mm_to_pt, h_mm * mm_to_pt)
    }

    /// Get the display name for this paper size
    pub fn display_name(&self) -> &'static str {
        match self {
            PaperSize::A4 => "A4",
            PaperSize::Letter => "Letter",
            PaperSize::Legal => "Legal",
            PaperSize::A3 => "A3",
            PaperSize::A5 => "A5",
            PaperSize::Tabloid => "Tabloid",
            PaperSize::Custom { .. } => "Custom",
        }
    }

    /// Create a custom paper size from millimeters
    pub fn custom(width_mm: f32, height_mm: f32) -> Self {
        PaperSize::Custom { width_mm, height_mm }
    }

    /// Create a custom paper size from inches
    pub fn custom_inches(width_in: f32, height_in: f32) -> Self {
        PaperSize::Custom {
            width_mm: width_in * 25.4,
            height_mm: height_in * 25.4,
        }
    }
}

/// Page orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Orientation {
    /// Portrait orientation (taller than wide)
    #[default]
    Portrait,
    /// Landscape orientation (wider than tall)
    Landscape,
}

impl Orientation {
    /// Apply orientation to dimensions
    pub fn apply(&self, width: f32, height: f32) -> (f32, f32) {
        match self {
            Orientation::Portrait => (width.min(height), width.max(height)),
            Orientation::Landscape => (width.max(height), width.min(height)),
        }
    }

    /// Get the display name for this orientation
    pub fn display_name(&self) -> &'static str {
        match self {
            Orientation::Portrait => "Portrait",
            Orientation::Landscape => "Landscape",
        }
    }
}

/// Print quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PrintQuality {
    /// Draft quality - fastest, lowest quality
    Draft,
    /// Normal quality - balanced
    #[default]
    Normal,
    /// High quality - slower, better output
    High,
    /// Best quality - slowest, highest quality
    Best,
}

impl PrintQuality {
    /// Get the DPI (dots per inch) for this quality level
    pub fn dpi(&self) -> u32 {
        match self {
            PrintQuality::Draft => 150,
            PrintQuality::Normal => 300,
            PrintQuality::High => 600,
            PrintQuality::Best => 1200,
        }
    }

    /// Get the display name for this quality level
    pub fn display_name(&self) -> &'static str {
        match self {
            PrintQuality::Draft => "Draft (150 DPI)",
            PrintQuality::Normal => "Normal (300 DPI)",
            PrintQuality::High => "High (600 DPI)",
            PrintQuality::Best => "Best (1200 DPI)",
        }
    }
}

/// Print margins in millimeters
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PrintMargins {
    /// Top margin in millimeters
    pub top: f32,
    /// Bottom margin in millimeters
    pub bottom: f32,
    /// Left margin in millimeters
    pub left: f32,
    /// Right margin in millimeters
    pub right: f32,
}

impl Default for PrintMargins {
    fn default() -> Self {
        // Standard margins: 25.4mm (1 inch) on all sides
        Self {
            top: 25.4,
            bottom: 25.4,
            left: 25.4,
            right: 25.4,
        }
    }
}

impl PrintMargins {
    /// Create margins with equal values on all sides
    pub fn all(margin: f32) -> Self {
        Self {
            top: margin,
            bottom: margin,
            left: margin,
            right: margin,
        }
    }

    /// Create margins with no space
    pub fn none() -> Self {
        Self::all(0.0)
    }

    /// Create narrow margins (12.7mm / 0.5 inch)
    pub fn narrow() -> Self {
        Self::all(12.7)
    }

    /// Create wide margins (50.8mm / 2 inches)
    pub fn wide() -> Self {
        Self::all(50.8)
    }

    /// Create custom margins
    pub fn custom(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    /// Convert margins to points
    pub fn to_points(&self) -> (f32, f32, f32, f32) {
        let mm_to_pt = 72.0 / 25.4;
        (
            self.top * mm_to_pt,
            self.bottom * mm_to_pt,
            self.left * mm_to_pt,
            self.right * mm_to_pt,
        )
    }

    /// Calculate the printable area dimensions in mm given paper size
    pub fn printable_area(&self, paper_width: f32, paper_height: f32) -> (f32, f32) {
        let width = paper_width - self.left - self.right;
        let height = paper_height - self.top - self.bottom;
        (width.max(0.0), height.max(0.0))
    }
}

/// Page range selection for printing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PageRange {
    /// Print all pages
    All,
    /// Print current page only
    Current,
    /// Print specific page range (1-indexed, inclusive)
    Range { start: u32, end: u32 },
    /// Print specific pages (1-indexed)
    Pages(Vec<u32>),
}

impl Default for PageRange {
    fn default() -> Self {
        PageRange::All
    }
}

impl PageRange {
    /// Create a range from start to end (1-indexed, inclusive)
    pub fn range(start: u32, end: u32) -> Self {
        PageRange::Range {
            start: start.max(1),
            end: end.max(start.max(1)),
        }
    }

    /// Create a selection of specific pages (1-indexed)
    pub fn pages(pages: Vec<u32>) -> Self {
        let mut pages: Vec<u32> = pages.into_iter().filter(|&p| p > 0).collect();
        pages.sort_unstable();
        pages.dedup();
        PageRange::Pages(pages)
    }

    /// Parse a page range string like "1-5" or "1,3,5-7"
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();
        if input.is_empty() || input.eq_ignore_ascii_case("all") {
            return Ok(PageRange::All);
        }

        let mut pages = Vec::new();
        for part in input.split(',') {
            let part = part.trim();
            if part.contains('-') {
                let mut parts = part.splitn(2, '-');
                let start: u32 = parts
                    .next()
                    .unwrap_or("1")
                    .trim()
                    .parse()
                    .map_err(|_| format!("Invalid page number in '{}'", part))?;
                let end: u32 = parts
                    .next()
                    .unwrap_or("1")
                    .trim()
                    .parse()
                    .map_err(|_| format!("Invalid page number in '{}'", part))?;
                for p in start..=end {
                    pages.push(p);
                }
            } else {
                let page: u32 = part
                    .parse()
                    .map_err(|_| format!("Invalid page number: '{}'", part))?;
                pages.push(page);
            }
        }

        Ok(PageRange::pages(pages))
    }

    /// Check if a page number (1-indexed) is included in this range
    pub fn includes(&self, page: u32, total_pages: u32) -> bool {
        match self {
            PageRange::All => page >= 1 && page <= total_pages,
            PageRange::Current => page == 1, // Current page is always page 1 in preview context
            PageRange::Range { start, end } => page >= *start && page <= *end,
            PageRange::Pages(pages) => pages.contains(&page),
        }
    }

    /// Get the number of pages to print given total page count
    pub fn page_count(&self, total_pages: u32) -> u32 {
        match self {
            PageRange::All => total_pages,
            PageRange::Current => 1,
            PageRange::Range { start, end } => {
                let effective_end = (*end).min(total_pages);
                let effective_start = (*start).min(effective_end);
                effective_end - effective_start + 1
            }
            PageRange::Pages(pages) => pages.iter().filter(|&&p| p <= total_pages).count() as u32,
        }
    }
}

/// Print output destination
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrintDestination {
    /// Print to default system printer
    DefaultPrinter,
    /// Print to a specific printer by name
    Printer(String),
    /// Save as PDF to specified path
    Pdf(PathBuf),
}

impl Default for PrintDestination {
    fn default() -> Self {
        PrintDestination::DefaultPrinter
    }
}

/// Complete print settings configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrintSettings {
    /// Paper size
    pub paper_size: PaperSize,
    /// Page orientation
    pub orientation: Orientation,
    /// Page margins
    pub margins: PrintMargins,
    /// Print quality
    pub quality: PrintQuality,
    /// Scale percentage (1-200)
    pub scale: u8,
    /// Number of copies
    pub copies: u32,
    /// Collate copies when printing multiple
    pub collate: bool,
    /// Print in color or grayscale
    pub color: bool,
    /// Print double-sided (duplex)
    pub duplex: bool,
    /// Page range to print
    pub page_range: PageRange,
    /// Print destination
    pub destination: PrintDestination,
    /// Print headers and footers
    pub headers_footers: bool,
    /// Print background graphics
    pub background_graphics: bool,
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            paper_size: PaperSize::default(),
            orientation: Orientation::default(),
            margins: PrintMargins::default(),
            quality: PrintQuality::default(),
            scale: 100,
            copies: 1,
            collate: true,
            color: true,
            duplex: false,
            page_range: PageRange::default(),
            destination: PrintDestination::default(),
            headers_footers: true,
            background_graphics: true,
        }
    }
}

impl PrintSettings {
    /// Create new print settings with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set paper size
    pub fn with_paper_size(mut self, paper_size: PaperSize) -> Self {
        self.paper_size = paper_size;
        self
    }

    /// Set orientation
    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set margins
    pub fn with_margins(mut self, margins: PrintMargins) -> Self {
        self.margins = margins;
        self
    }

    /// Set quality
    pub fn with_quality(mut self, quality: PrintQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Set scale percentage (clamped to 1-200)
    pub fn with_scale(mut self, scale: u8) -> Self {
        self.scale = scale.clamp(1, 200);
        self
    }

    /// Set number of copies
    pub fn with_copies(mut self, copies: u32) -> Self {
        self.copies = copies.max(1);
        self
    }

    /// Set page range
    pub fn with_page_range(mut self, page_range: PageRange) -> Self {
        self.page_range = page_range;
        self
    }

    /// Set destination
    pub fn with_destination(mut self, destination: PrintDestination) -> Self {
        self.destination = destination;
        self
    }

    /// Get effective page dimensions in mm after applying orientation
    pub fn page_dimensions_mm(&self) -> (f32, f32) {
        let (w, h) = self.paper_size.dimensions_mm();
        self.orientation.apply(w, h)
    }

    /// Get effective page dimensions in points after applying orientation
    pub fn page_dimensions_points(&self) -> (f32, f32) {
        let (w, h) = self.paper_size.dimensions_points();
        self.orientation.apply(w, h)
    }

    /// Get printable area in mm (page size minus margins)
    pub fn printable_area_mm(&self) -> (f32, f32) {
        let (w, h) = self.page_dimensions_mm();
        self.margins.printable_area(w, h)
    }
}

/// Print job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrintJobStatus {
    /// Job is queued, waiting to print
    Queued,
    /// Job is currently printing
    Printing,
    /// Job is paused
    Paused,
    /// Job completed successfully
    Completed,
    /// Job was cancelled
    Cancelled,
    /// Job failed with error
    Failed,
}

impl PrintJobStatus {
    /// Check if the job is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            PrintJobStatus::Completed | PrintJobStatus::Cancelled | PrintJobStatus::Failed
        )
    }

    /// Check if the job is currently active
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            PrintJobStatus::Queued | PrintJobStatus::Printing | PrintJobStatus::Paused
        )
    }

    /// Get display name for this status
    pub fn display_name(&self) -> &'static str {
        match self {
            PrintJobStatus::Queued => "Queued",
            PrintJobStatus::Printing => "Printing",
            PrintJobStatus::Paused => "Paused",
            PrintJobStatus::Completed => "Completed",
            PrintJobStatus::Cancelled => "Cancelled",
            PrintJobStatus::Failed => "Failed",
        }
    }
}

/// Unique identifier for a print job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrintJobId(u64);

impl PrintJobId {
    /// Create a new unique print job ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for PrintJobId {
    fn default() -> Self {
        Self::new()
    }
}

/// A print job with tracking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    /// Unique identifier for this job
    pub id: PrintJobId,
    /// Document title/name
    pub title: String,
    /// Current status
    pub status: PrintJobStatus,
    /// Total number of pages
    pub total_pages: u32,
    /// Number of pages printed
    pub printed_pages: u32,
    /// Print settings used
    pub settings: PrintSettings,
    /// Error message if status is Failed
    pub error_message: Option<String>,
    /// Timestamp when job was created (Unix timestamp)
    pub created_at: u64,
    /// Timestamp when job started printing
    pub started_at: Option<u64>,
    /// Timestamp when job completed/failed
    pub completed_at: Option<u64>,
}

impl PrintJob {
    /// Create a new print job
    pub fn new(title: String, total_pages: u32, settings: PrintSettings) -> Self {
        Self {
            id: PrintJobId::new(),
            title,
            status: PrintJobStatus::Queued,
            total_pages,
            printed_pages: 0,
            settings,
            error_message: None,
            created_at: current_timestamp(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Get progress as a percentage (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.total_pages == 0 {
            return 0.0;
        }
        self.printed_pages as f32 / self.total_pages as f32
    }

    /// Get progress as a percentage (0 to 100)
    pub fn progress_percent(&self) -> u8 {
        (self.progress() * 100.0) as u8
    }

    /// Start the print job
    pub fn start(&mut self) {
        if self.status == PrintJobStatus::Queued {
            self.status = PrintJobStatus::Printing;
            self.started_at = Some(current_timestamp());
        }
    }

    /// Pause the print job
    pub fn pause(&mut self) {
        if self.status == PrintJobStatus::Printing {
            self.status = PrintJobStatus::Paused;
        }
    }

    /// Resume a paused print job
    pub fn resume(&mut self) {
        if self.status == PrintJobStatus::Paused {
            self.status = PrintJobStatus::Printing;
        }
    }

    /// Cancel the print job
    pub fn cancel(&mut self) {
        if self.status.is_active() {
            self.status = PrintJobStatus::Cancelled;
            self.completed_at = Some(current_timestamp());
        }
    }

    /// Mark a page as printed
    pub fn page_printed(&mut self) {
        if self.status == PrintJobStatus::Printing && self.printed_pages < self.total_pages {
            self.printed_pages += 1;
            if self.printed_pages >= self.total_pages {
                self.status = PrintJobStatus::Completed;
                self.completed_at = Some(current_timestamp());
            }
        }
    }

    /// Mark the job as failed
    pub fn fail(&mut self, error: String) {
        self.status = PrintJobStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(current_timestamp());
    }

    /// Complete the job successfully
    pub fn complete(&mut self) {
        self.status = PrintJobStatus::Completed;
        self.printed_pages = self.total_pages;
        self.completed_at = Some(current_timestamp());
    }
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Print preview for displaying page layout
#[derive(Debug)]
pub struct PrintPreview {
    /// Current settings for preview
    settings: PrintSettings,
    /// Total pages in document
    total_pages: u32,
    /// Currently displayed page (1-indexed)
    current_page: u32,
    /// Zoom level (1.0 = 100%)
    zoom: f32,
    /// Whether to show margin guides
    show_margins: bool,
    /// Whether preview is visible
    visible: bool,
}

impl Default for PrintPreview {
    fn default() -> Self {
        Self::new()
    }
}

impl PrintPreview {
    /// Create a new print preview
    pub fn new() -> Self {
        Self {
            settings: PrintSettings::default(),
            total_pages: 1,
            current_page: 1,
            zoom: 1.0,
            show_margins: true,
            visible: false,
        }
    }

    /// Create a preview with specific settings
    pub fn with_settings(settings: PrintSettings) -> Self {
        Self {
            settings,
            total_pages: 1,
            current_page: 1,
            zoom: 1.0,
            show_margins: true,
            visible: false,
        }
    }

    /// Get the current settings
    pub fn settings(&self) -> &PrintSettings {
        &self.settings
    }

    /// Get mutable reference to settings
    pub fn settings_mut(&mut self) -> &mut PrintSettings {
        &mut self.settings
    }

    /// Set the print settings
    pub fn set_settings(&mut self, settings: PrintSettings) {
        self.settings = settings;
    }

    /// Set total number of pages
    pub fn set_total_pages(&mut self, pages: u32) {
        self.total_pages = pages.max(1);
        if self.current_page > self.total_pages {
            self.current_page = self.total_pages;
        }
    }

    /// Get total pages
    pub fn total_pages(&self) -> u32 {
        self.total_pages
    }

    /// Get current page (1-indexed)
    pub fn current_page(&self) -> u32 {
        self.current_page
    }

    /// Set current page (1-indexed)
    pub fn set_current_page(&mut self, page: u32) {
        self.current_page = page.clamp(1, self.total_pages);
    }

    /// Go to next page
    pub fn next_page(&mut self) {
        if self.current_page < self.total_pages {
            self.current_page += 1;
        }
    }

    /// Go to previous page
    pub fn prev_page(&mut self) {
        if self.current_page > 1 {
            self.current_page -= 1;
        }
    }

    /// Get zoom level
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Set zoom level (clamped to 0.25 to 4.0)
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.25, 4.0);
    }

    /// Zoom in by 25%
    pub fn zoom_in(&mut self) {
        self.set_zoom(self.zoom + 0.25);
    }

    /// Zoom out by 25%
    pub fn zoom_out(&mut self) {
        self.set_zoom(self.zoom - 0.25);
    }

    /// Reset zoom to 100%
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
    }

    /// Toggle margin guides visibility
    pub fn toggle_margins(&mut self) {
        self.show_margins = !self.show_margins;
    }

    /// Check if showing margin guides
    pub fn showing_margins(&self) -> bool {
        self.show_margins
    }

    /// Show the preview
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the preview
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if preview is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Render the print preview panel in egui
    pub fn render(&mut self, ctx: &egui::Context) -> PrintPreviewResponse {
        let mut response = PrintPreviewResponse::default();

        if !self.visible {
            return response;
        }

        egui::Window::new("Print Preview")
            .default_size([800.0, 600.0])
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Top toolbar
                ui.horizontal(|ui| {
                    // Page navigation
                    if ui.button("<<").clicked() && self.current_page > 1 {
                        self.current_page = 1;
                    }
                    if ui.button("<").clicked() {
                        self.prev_page();
                    }
                    ui.label(format!(
                        "Page {} of {}",
                        self.current_page, self.total_pages
                    ));
                    if ui.button(">").clicked() {
                        self.next_page();
                    }
                    if ui.button(">>").clicked() && self.current_page < self.total_pages {
                        self.current_page = self.total_pages;
                    }

                    ui.separator();

                    // Zoom controls
                    if ui.button("-").clicked() {
                        self.zoom_out();
                    }
                    ui.label(format!("{}%", (self.zoom * 100.0) as u32));
                    if ui.button("+").clicked() {
                        self.zoom_in();
                    }
                    if ui.button("Reset").clicked() {
                        self.reset_zoom();
                    }

                    ui.separator();

                    // Margin toggle
                    ui.checkbox(&mut self.show_margins, "Show Margins");
                });

                ui.separator();

                // Settings panel on the left
                ui.horizontal(|ui| {
                    // Settings sidebar
                    ui.vertical(|ui| {
                        ui.set_width(200.0);
                        ui.heading("Settings");

                        // Paper size selector
                        ui.label("Paper Size:");
                        egui::ComboBox::from_id_salt("paper_size")
                            .selected_text(self.settings.paper_size.display_name())
                            .show_ui(ui, |ui| {
                                for size in [
                                    PaperSize::A4,
                                    PaperSize::Letter,
                                    PaperSize::Legal,
                                    PaperSize::A3,
                                    PaperSize::A5,
                                    PaperSize::Tabloid,
                                ] {
                                    if ui
                                        .selectable_label(
                                            self.settings.paper_size == size,
                                            size.display_name(),
                                        )
                                        .clicked()
                                    {
                                        self.settings.paper_size = size;
                                    }
                                }
                            });

                        // Orientation selector
                        ui.label("Orientation:");
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    self.settings.orientation == Orientation::Portrait,
                                    "Portrait",
                                )
                                .clicked()
                            {
                                self.settings.orientation = Orientation::Portrait;
                            }
                            if ui
                                .selectable_label(
                                    self.settings.orientation == Orientation::Landscape,
                                    "Landscape",
                                )
                                .clicked()
                            {
                                self.settings.orientation = Orientation::Landscape;
                            }
                        });

                        // Scale slider
                        ui.label(format!("Scale: {}%", self.settings.scale));
                        let mut scale_f32 = self.settings.scale as f32;
                        if ui
                            .add(egui::Slider::new(&mut scale_f32, 25.0..=200.0))
                            .changed()
                        {
                            self.settings.scale = scale_f32 as u8;
                        }

                        // Copies
                        ui.label("Copies:");
                        let mut copies = self.settings.copies;
                        if ui
                            .add(egui::DragValue::new(&mut copies).range(1..=999))
                            .changed()
                        {
                            self.settings.copies = copies;
                        }

                        ui.separator();

                        // Options
                        ui.checkbox(&mut self.settings.color, "Color");
                        ui.checkbox(&mut self.settings.duplex, "Double-sided");
                        ui.checkbox(&mut self.settings.headers_footers, "Headers/Footers");
                        ui.checkbox(&mut self.settings.background_graphics, "Background Graphics");

                        ui.separator();

                        // Action buttons
                        if ui.button("Print").clicked() {
                            response.print_requested = true;
                        }
                        if ui.button("Save as PDF").clicked() {
                            response.save_pdf_requested = true;
                        }
                        if ui.button("Cancel").clicked() {
                            response.cancelled = true;
                            self.visible = false;
                        }
                    });

                    ui.separator();

                    // Page preview area
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            self.render_page_preview(ui);
                        });
                });
            });

        response
    }

    /// Render the page preview area
    fn render_page_preview(&self, ui: &mut egui::Ui) {
        // Get page dimensions in a reasonable display size
        let (page_w, page_h) = self.settings.page_dimensions_mm();

        // Scale to fit in preview area (base size ~500 pixels for largest dimension)
        let base_scale = 500.0 / page_w.max(page_h);
        let display_scale = base_scale * self.zoom;

        let display_w = page_w * display_scale;
        let display_h = page_h * display_scale;

        // Allocate space for the page
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(display_w, display_h), egui::Sense::hover());

        let painter = ui.painter();

        // Draw page shadow
        let shadow_offset = Vec2::new(3.0, 3.0);
        painter.rect_filled(
            Rect::from_min_size(rect.min + shadow_offset, rect.size()),
            0.0,
            Color32::from_gray(180),
        );

        // Draw page background (white paper)
        painter.rect_filled(rect, 0.0, Color32::WHITE);

        // Draw page border
        painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(100)));

        // Draw margin guides if enabled
        if self.show_margins {
            let margins = &self.settings.margins;
            let margin_scale = display_scale;

            let margin_rect = Rect::from_min_max(
                Pos2::new(
                    rect.min.x + margins.left * margin_scale,
                    rect.min.y + margins.top * margin_scale,
                ),
                Pos2::new(
                    rect.max.x - margins.right * margin_scale,
                    rect.max.y - margins.bottom * margin_scale,
                ),
            );

            painter.rect_stroke(
                margin_rect,
                0.0,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 120, 212, 100)),
            );
        }

        // Draw content placeholder
        let content_rect = if self.show_margins {
            let margins = &self.settings.margins;
            let margin_scale = display_scale;
            Rect::from_min_max(
                Pos2::new(
                    rect.min.x + margins.left * margin_scale + 10.0,
                    rect.min.y + margins.top * margin_scale + 10.0,
                ),
                Pos2::new(
                    rect.max.x - margins.right * margin_scale - 10.0,
                    rect.min.y + margins.top * margin_scale + 40.0,
                ),
            )
        } else {
            Rect::from_min_size(rect.min + Vec2::new(20.0, 20.0), Vec2::new(100.0, 20.0))
        };

        // Draw some placeholder content lines
        let line_height = 12.0 * self.zoom;
        let mut y = content_rect.min.y;
        let content_width = content_rect.width();

        for i in 0..((content_rect.height() / (line_height * 1.5)) as usize).min(20) {
            let line_width = if i == 0 {
                content_width * 0.6
            } else {
                content_width * (0.7 + (i % 3) as f32 * 0.1)
            };

            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(content_rect.min.x, y),
                    Vec2::new(line_width, line_height * 0.6),
                ),
                2.0,
                Color32::from_gray(200),
            );

            y += line_height * 1.5;
            if y > rect.max.y - 20.0 {
                break;
            }
        }

        // Draw page number at bottom
        let page_num_text = format!("Page {}", self.current_page);
        painter.text(
            Pos2::new(rect.center().x, rect.max.y - 15.0),
            egui::Align2::CENTER_CENTER,
            page_num_text,
            egui::FontId::default(),
            Color32::GRAY,
        );
    }
}

/// Response from print preview interaction
#[derive(Debug, Default)]
pub struct PrintPreviewResponse {
    /// User requested to print
    pub print_requested: bool,
    /// User requested to save as PDF
    pub save_pdf_requested: bool,
    /// User cancelled the preview
    pub cancelled: bool,
}

/// Print manager for handling print jobs and system dialog integration
#[derive(Debug)]
pub struct PrintManager {
    /// Active print jobs
    jobs: Vec<PrintJob>,
    /// Print preview state
    preview: PrintPreview,
    /// Default print settings
    default_settings: PrintSettings,
}

impl Default for PrintManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PrintManager {
    /// Create a new print manager
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            preview: PrintPreview::new(),
            default_settings: PrintSettings::default(),
        }
    }

    /// Get the print preview
    pub fn preview(&self) -> &PrintPreview {
        &self.preview
    }

    /// Get mutable reference to print preview
    pub fn preview_mut(&mut self) -> &mut PrintPreview {
        &mut self.preview
    }

    /// Show print preview with current settings
    pub fn show_preview(&mut self) {
        self.preview.set_settings(self.default_settings.clone());
        self.preview.show();
    }

    /// Show print preview with specific settings
    pub fn show_preview_with_settings(&mut self, settings: PrintSettings) {
        self.preview.set_settings(settings);
        self.preview.show();
    }

    /// Hide print preview
    pub fn hide_preview(&mut self) {
        self.preview.hide();
    }

    /// Get default settings
    pub fn default_settings(&self) -> &PrintSettings {
        &self.default_settings
    }

    /// Set default settings
    pub fn set_default_settings(&mut self, settings: PrintSettings) {
        self.default_settings = settings;
    }

    /// Create a new print job
    pub fn create_job(&mut self, title: String, total_pages: u32) -> PrintJobId {
        let settings = self.preview.settings().clone();
        let job = PrintJob::new(title, total_pages, settings);
        let id = job.id;
        self.jobs.push(job);
        id
    }

    /// Get a print job by ID
    pub fn get_job(&self, id: PrintJobId) -> Option<&PrintJob> {
        self.jobs.iter().find(|j| j.id == id)
    }

    /// Get mutable reference to a print job by ID
    pub fn get_job_mut(&mut self, id: PrintJobId) -> Option<&mut PrintJob> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }

    /// Get all print jobs
    pub fn jobs(&self) -> &[PrintJob] {
        &self.jobs
    }

    /// Get active (non-terminal) print jobs
    pub fn active_jobs(&self) -> Vec<&PrintJob> {
        self.jobs.iter().filter(|j| j.status.is_active()).collect()
    }

    /// Cancel a print job
    pub fn cancel_job(&mut self, id: PrintJobId) -> bool {
        if let Some(job) = self.get_job_mut(id) {
            job.cancel();
            true
        } else {
            false
        }
    }

    /// Remove completed/cancelled/failed jobs
    pub fn clear_completed_jobs(&mut self) {
        self.jobs.retain(|j| j.status.is_active());
    }

    /// Show system print dialog (platform stub)
    ///
    /// Returns Ok(true) if user confirms, Ok(false) if cancelled, Err on failure
    pub fn show_system_dialog(&self) -> Result<bool, PrintError> {
        show_system_print_dialog(&self.default_settings)
    }

    /// Print to PDF
    pub fn print_to_pdf(&mut self, path: PathBuf, title: String, pages: u32) -> Result<PrintJobId, PrintError> {
        let mut settings = self.preview.settings().clone();
        settings.destination = PrintDestination::Pdf(path);

        let mut job = PrintJob::new(title, pages, settings);
        let id = job.id;

        // Simulate PDF generation (in real implementation, this would render to PDF)
        job.start();
        job.complete();

        self.jobs.push(job);
        Ok(id)
    }

    /// Render the print manager UI (preview and job list)
    pub fn render(&mut self, ctx: &egui::Context) -> PrintManagerResponse {
        let preview_response = self.preview.render(ctx);

        PrintManagerResponse {
            preview_response,
            jobs_updated: false,
        }
    }
}

/// Response from print manager UI
#[derive(Debug, Default)]
pub struct PrintManagerResponse {
    /// Response from print preview
    pub preview_response: PrintPreviewResponse,
    /// Whether jobs list was updated
    pub jobs_updated: bool,
}

/// Errors that can occur during printing
#[derive(Debug, Clone, thiserror::Error)]
pub enum PrintError {
    /// No printer available
    #[error("No printer available")]
    NoPrinter,
    /// Printer not found
    #[error("Printer not found: {0}")]
    PrinterNotFound(String),
    /// Print job failed
    #[error("Print job failed: {0}")]
    JobFailed(String),
    /// PDF generation failed
    #[error("PDF generation failed: {0}")]
    PdfFailed(String),
    /// Invalid settings
    #[error("Invalid print settings: {0}")]
    InvalidSettings(String),
    /// Platform not supported
    #[error("Printing not supported on this platform")]
    UnsupportedPlatform,
    /// User cancelled
    #[error("Print cancelled by user")]
    Cancelled,
}

// Platform-specific print dialog stubs

/// Show the system print dialog (platform-specific)
///
/// Returns Ok(true) if user confirms, Ok(false) if cancelled
#[allow(unused_variables)]
pub fn show_system_print_dialog(settings: &PrintSettings) -> Result<bool, PrintError> {
    #[cfg(target_os = "linux")]
    {
        show_print_dialog_linux(settings)
    }

    #[cfg(target_os = "macos")]
    {
        show_print_dialog_macos(settings)
    }

    #[cfg(target_os = "windows")]
    {
        show_print_dialog_windows(settings)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(PrintError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "linux")]
fn show_print_dialog_linux(_settings: &PrintSettings) -> Result<bool, PrintError> {
    // In a real implementation, this would use GTK print dialog or CUPS
    // For now, return a stub response
    // Could use: gtk::PrintOperation or direct CUPS API
    Ok(true)
}

#[cfg(target_os = "macos")]
fn show_print_dialog_macos(_settings: &PrintSettings) -> Result<bool, PrintError> {
    // In a real implementation, this would use NSPrintOperation
    // For now, return a stub response
    Ok(true)
}

#[cfg(target_os = "windows")]
fn show_print_dialog_windows(_settings: &PrintSettings) -> Result<bool, PrintError> {
    // In a real implementation, this would use Windows Print API
    // For now, return a stub response
    Ok(true)
}

/// Get list of available printers (platform-specific)
pub fn get_available_printers() -> Result<Vec<String>, PrintError> {
    #[cfg(target_os = "linux")]
    {
        get_printers_linux()
    }

    #[cfg(target_os = "macos")]
    {
        get_printers_macos()
    }

    #[cfg(target_os = "windows")]
    {
        get_printers_windows()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(PrintError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "linux")]
fn get_printers_linux() -> Result<Vec<String>, PrintError> {
    // In a real implementation, query CUPS for printer list
    // lpstat -p or cups API
    Ok(vec!["Default Printer".to_string()])
}

#[cfg(target_os = "macos")]
fn get_printers_macos() -> Result<Vec<String>, PrintError> {
    // In a real implementation, use PMServerCreatePrinterList
    Ok(vec!["Default Printer".to_string()])
}

#[cfg(target_os = "windows")]
fn get_printers_windows() -> Result<Vec<String>, PrintError> {
    // In a real implementation, use EnumPrinters API
    Ok(vec!["Default Printer".to_string()])
}

/// Get the default printer name (platform-specific)
pub fn get_default_printer() -> Result<Option<String>, PrintError> {
    #[cfg(target_os = "linux")]
    {
        // Query lpstat -d
        Ok(Some("Default Printer".to_string()))
    }

    #[cfg(target_os = "macos")]
    {
        // Query PMCreateDefaultPrinter
        Ok(Some("Default Printer".to_string()))
    }

    #[cfg(target_os = "windows")]
    {
        // Query GetDefaultPrinter
        Ok(Some("Default Printer".to_string()))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(PrintError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PaperSize tests
    #[test]
    fn test_paper_size_a4_dimensions() {
        let size = PaperSize::A4;
        let (w, h) = size.dimensions_mm();
        assert!((w - 210.0).abs() < 0.1);
        assert!((h - 297.0).abs() < 0.1);
    }

    #[test]
    fn test_paper_size_letter_dimensions() {
        let size = PaperSize::Letter;
        let (w, h) = size.dimensions_mm();
        assert!((w - 215.9).abs() < 0.1);
        assert!((h - 279.4).abs() < 0.1);
    }

    #[test]
    fn test_paper_size_custom() {
        let size = PaperSize::custom(100.0, 200.0);
        let (w, h) = size.dimensions_mm();
        assert!((w - 100.0).abs() < 0.1);
        assert!((h - 200.0).abs() < 0.1);
    }

    #[test]
    fn test_paper_size_custom_inches() {
        let size = PaperSize::custom_inches(8.5, 11.0);
        let (w, h) = size.dimensions_mm();
        assert!((w - 215.9).abs() < 0.1);
        assert!((h - 279.4).abs() < 0.1);
    }

    #[test]
    fn test_paper_size_to_points() {
        let size = PaperSize::A4;
        let (w_pt, h_pt) = size.dimensions_points();
        // A4 is ~595 x 842 points
        assert!((w_pt - 595.3).abs() < 1.0);
        assert!((h_pt - 841.9).abs() < 1.0);
    }

    #[test]
    fn test_paper_size_display_name() {
        assert_eq!(PaperSize::A4.display_name(), "A4");
        assert_eq!(PaperSize::Letter.display_name(), "Letter");
        assert_eq!(PaperSize::Custom { width_mm: 0.0, height_mm: 0.0 }.display_name(), "Custom");
    }

    // Orientation tests
    #[test]
    fn test_orientation_portrait() {
        let (w, h) = Orientation::Portrait.apply(210.0, 297.0);
        assert!(w < h);
    }

    #[test]
    fn test_orientation_landscape() {
        let (w, h) = Orientation::Landscape.apply(210.0, 297.0);
        assert!(w > h);
    }

    #[test]
    fn test_orientation_display_name() {
        assert_eq!(Orientation::Portrait.display_name(), "Portrait");
        assert_eq!(Orientation::Landscape.display_name(), "Landscape");
    }

    // PrintQuality tests
    #[test]
    fn test_print_quality_dpi() {
        assert_eq!(PrintQuality::Draft.dpi(), 150);
        assert_eq!(PrintQuality::Normal.dpi(), 300);
        assert_eq!(PrintQuality::High.dpi(), 600);
        assert_eq!(PrintQuality::Best.dpi(), 1200);
    }

    // PrintMargins tests
    #[test]
    fn test_margins_default() {
        let margins = PrintMargins::default();
        assert!((margins.top - 25.4).abs() < 0.1);
        assert!((margins.bottom - 25.4).abs() < 0.1);
        assert!((margins.left - 25.4).abs() < 0.1);
        assert!((margins.right - 25.4).abs() < 0.1);
    }

    #[test]
    fn test_margins_all() {
        let margins = PrintMargins::all(10.0);
        assert!((margins.top - 10.0).abs() < 0.1);
        assert!((margins.bottom - 10.0).abs() < 0.1);
        assert!((margins.left - 10.0).abs() < 0.1);
        assert!((margins.right - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_margins_none() {
        let margins = PrintMargins::none();
        assert!(margins.top.abs() < 0.1);
    }

    #[test]
    fn test_margins_printable_area() {
        let margins = PrintMargins::all(10.0);
        let (w, h) = margins.printable_area(100.0, 100.0);
        assert!((w - 80.0).abs() < 0.1);
        assert!((h - 80.0).abs() < 0.1);
    }

    // PageRange tests
    #[test]
    fn test_page_range_all() {
        let range = PageRange::All;
        assert!(range.includes(1, 10));
        assert!(range.includes(5, 10));
        assert!(range.includes(10, 10));
        assert!(!range.includes(11, 10));
        assert_eq!(range.page_count(10), 10);
    }

    #[test]
    fn test_page_range_specific() {
        let range = PageRange::range(2, 5);
        assert!(!range.includes(1, 10));
        assert!(range.includes(2, 10));
        assert!(range.includes(5, 10));
        assert!(!range.includes(6, 10));
        assert_eq!(range.page_count(10), 4);
    }

    #[test]
    fn test_page_range_pages() {
        let range = PageRange::pages(vec![1, 3, 5, 7]);
        assert!(range.includes(1, 10));
        assert!(!range.includes(2, 10));
        assert!(range.includes(3, 10));
        assert_eq!(range.page_count(10), 4);
    }

    #[test]
    fn test_page_range_parse() {
        let range = PageRange::parse("1-5").unwrap();
        assert_eq!(range.page_count(10), 5);

        let range = PageRange::parse("1,3,5").unwrap();
        assert_eq!(range.page_count(10), 3);

        let range = PageRange::parse("1-3,5,7-9").unwrap();
        assert_eq!(range.page_count(10), 7);

        let range = PageRange::parse("all").unwrap();
        assert!(matches!(range, PageRange::All));

        assert!(PageRange::parse("invalid").is_err());
    }

    // PrintSettings tests
    #[test]
    fn test_print_settings_default() {
        let settings = PrintSettings::default();
        assert_eq!(settings.paper_size, PaperSize::A4);
        assert_eq!(settings.orientation, Orientation::Portrait);
        assert_eq!(settings.scale, 100);
        assert_eq!(settings.copies, 1);
    }

    #[test]
    fn test_print_settings_builder() {
        let settings = PrintSettings::new()
            .with_paper_size(PaperSize::Letter)
            .with_orientation(Orientation::Landscape)
            .with_scale(75)
            .with_copies(2);

        assert_eq!(settings.paper_size, PaperSize::Letter);
        assert_eq!(settings.orientation, Orientation::Landscape);
        assert_eq!(settings.scale, 75);
        assert_eq!(settings.copies, 2);
    }

    #[test]
    fn test_print_settings_scale_clamped() {
        let settings = PrintSettings::new().with_scale(250);
        assert_eq!(settings.scale, 200);

        let settings = PrintSettings::new().with_scale(0);
        assert_eq!(settings.scale, 1);
    }

    #[test]
    fn test_print_settings_page_dimensions() {
        let settings = PrintSettings::new()
            .with_paper_size(PaperSize::A4)
            .with_orientation(Orientation::Landscape);

        let (w, h) = settings.page_dimensions_mm();
        assert!(w > h); // Landscape should be wider
    }

    // PrintJobStatus tests
    #[test]
    fn test_job_status_terminal() {
        assert!(!PrintJobStatus::Queued.is_terminal());
        assert!(!PrintJobStatus::Printing.is_terminal());
        assert!(!PrintJobStatus::Paused.is_terminal());
        assert!(PrintJobStatus::Completed.is_terminal());
        assert!(PrintJobStatus::Cancelled.is_terminal());
        assert!(PrintJobStatus::Failed.is_terminal());
    }

    #[test]
    fn test_job_status_active() {
        assert!(PrintJobStatus::Queued.is_active());
        assert!(PrintJobStatus::Printing.is_active());
        assert!(PrintJobStatus::Paused.is_active());
        assert!(!PrintJobStatus::Completed.is_active());
    }

    // PrintJob tests
    #[test]
    fn test_print_job_creation() {
        let job = PrintJob::new("Test Document".to_string(), 10, PrintSettings::default());
        assert_eq!(job.title, "Test Document");
        assert_eq!(job.total_pages, 10);
        assert_eq!(job.printed_pages, 0);
        assert_eq!(job.status, PrintJobStatus::Queued);
    }

    #[test]
    fn test_print_job_progress() {
        let mut job = PrintJob::new("Test".to_string(), 10, PrintSettings::default());
        assert!((job.progress() - 0.0).abs() < 0.01);

        job.start();
        job.page_printed();
        job.page_printed();
        assert!((job.progress() - 0.2).abs() < 0.01);
        assert_eq!(job.progress_percent(), 20);
    }

    #[test]
    fn test_print_job_lifecycle() {
        let mut job = PrintJob::new("Test".to_string(), 2, PrintSettings::default());

        assert_eq!(job.status, PrintJobStatus::Queued);

        job.start();
        assert_eq!(job.status, PrintJobStatus::Printing);

        job.pause();
        assert_eq!(job.status, PrintJobStatus::Paused);

        job.resume();
        assert_eq!(job.status, PrintJobStatus::Printing);

        job.page_printed();
        job.page_printed();
        assert_eq!(job.status, PrintJobStatus::Completed);
    }

    #[test]
    fn test_print_job_cancel() {
        let mut job = PrintJob::new("Test".to_string(), 10, PrintSettings::default());
        job.start();
        job.cancel();
        assert_eq!(job.status, PrintJobStatus::Cancelled);
    }

    #[test]
    fn test_print_job_fail() {
        let mut job = PrintJob::new("Test".to_string(), 10, PrintSettings::default());
        job.start();
        job.fail("Paper jam".to_string());
        assert_eq!(job.status, PrintJobStatus::Failed);
        assert_eq!(job.error_message, Some("Paper jam".to_string()));
    }

    // PrintPreview tests
    #[test]
    fn test_print_preview_creation() {
        let preview = PrintPreview::new();
        assert_eq!(preview.total_pages(), 1);
        assert_eq!(preview.current_page(), 1);
        assert!((preview.zoom() - 1.0).abs() < 0.01);
        assert!(!preview.is_visible());
    }

    #[test]
    fn test_print_preview_navigation() {
        let mut preview = PrintPreview::new();
        preview.set_total_pages(5);

        assert_eq!(preview.current_page(), 1);

        preview.next_page();
        assert_eq!(preview.current_page(), 2);

        preview.prev_page();
        assert_eq!(preview.current_page(), 1);

        preview.prev_page(); // Should stay at 1
        assert_eq!(preview.current_page(), 1);

        preview.set_current_page(5);
        assert_eq!(preview.current_page(), 5);

        preview.next_page(); // Should stay at 5
        assert_eq!(preview.current_page(), 5);
    }

    #[test]
    fn test_print_preview_zoom() {
        let mut preview = PrintPreview::new();

        preview.zoom_in();
        assert!((preview.zoom() - 1.25).abs() < 0.01);

        preview.zoom_out();
        assert!((preview.zoom() - 1.0).abs() < 0.01);

        preview.set_zoom(5.0); // Should clamp to 4.0
        assert!((preview.zoom() - 4.0).abs() < 0.01);

        preview.set_zoom(0.1); // Should clamp to 0.25
        assert!((preview.zoom() - 0.25).abs() < 0.01);

        preview.reset_zoom();
        assert!((preview.zoom() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_print_preview_visibility() {
        let mut preview = PrintPreview::new();
        assert!(!preview.is_visible());

        preview.show();
        assert!(preview.is_visible());

        preview.hide();
        assert!(!preview.is_visible());
    }

    #[test]
    fn test_print_preview_margins_toggle() {
        let mut preview = PrintPreview::new();
        assert!(preview.showing_margins());

        preview.toggle_margins();
        assert!(!preview.showing_margins());

        preview.toggle_margins();
        assert!(preview.showing_margins());
    }

    // PrintManager tests
    #[test]
    fn test_print_manager_creation() {
        let manager = PrintManager::new();
        assert!(manager.jobs().is_empty());
    }

    #[test]
    fn test_print_manager_create_job() {
        let mut manager = PrintManager::new();
        let id = manager.create_job("Test Doc".to_string(), 5);

        let job = manager.get_job(id).unwrap();
        assert_eq!(job.title, "Test Doc");
        assert_eq!(job.total_pages, 5);
    }

    #[test]
    fn test_print_manager_cancel_job() {
        let mut manager = PrintManager::new();
        let id = manager.create_job("Test".to_string(), 5);

        assert!(manager.cancel_job(id));

        let job = manager.get_job(id).unwrap();
        assert_eq!(job.status, PrintJobStatus::Cancelled);
    }

    #[test]
    fn test_print_manager_clear_completed() {
        let mut manager = PrintManager::new();
        let id1 = manager.create_job("Test1".to_string(), 5);
        let _id2 = manager.create_job("Test2".to_string(), 5);

        manager.cancel_job(id1);
        assert_eq!(manager.jobs().len(), 2);

        manager.clear_completed_jobs();
        assert_eq!(manager.jobs().len(), 1);
    }

    #[test]
    fn test_print_manager_preview() {
        let mut manager = PrintManager::new();
        assert!(!manager.preview().is_visible());

        manager.show_preview();
        assert!(manager.preview().is_visible());

        manager.hide_preview();
        assert!(!manager.preview().is_visible());
    }

    // Platform function tests (stubs return default values)
    #[test]
    fn test_get_available_printers() {
        let result = get_available_printers();
        // On supported platforms, should return at least one printer
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_default_printer() {
        let result = get_default_printer();
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        assert!(result.is_ok());
    }

    // Serialization tests
    #[test]
    fn test_paper_size_serialization() {
        let size = PaperSize::A4;
        let json = serde_json::to_string(&size).unwrap();
        let restored: PaperSize = serde_json::from_str(&json).unwrap();
        assert_eq!(size, restored);
    }

    #[test]
    fn test_print_settings_serialization() {
        let settings = PrintSettings::new()
            .with_paper_size(PaperSize::Letter)
            .with_scale(75);

        let json = serde_json::to_string(&settings).unwrap();
        let restored: PrintSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.paper_size, restored.paper_size);
        assert_eq!(settings.scale, restored.scale);
    }

    #[test]
    fn test_print_job_serialization() {
        let job = PrintJob::new("Test".to_string(), 10, PrintSettings::default());
        let json = serde_json::to_string(&job).unwrap();
        let restored: PrintJob = serde_json::from_str(&json).unwrap();

        assert_eq!(job.title, restored.title);
        assert_eq!(job.total_pages, restored.total_pages);
    }
}
