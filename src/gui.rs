//! The window itself (built with the `gui` feature): it draws the state of `window::Window` and passes on what the person does. Everything that can be decided without a screen is
//! decided there, in code that is tested. The window has two pages: the table of the tools with its form, and the calculator of feeds and speeds.

use crate::calc::{
    CHIP_FRACTION_LIMITS, DRILL_PLUNGE_LIMITS, MATERIALS, MAX_PASS_LIMITS, PASS_FRACTION_LIMITS,
    PLUNGE_RATIO_LIMITS, SURFACE_SPEED_LIMITS, WORKING_LIMITS,
};
use crate::lang::{fill, Lang, Msg};
use crate::model::{Wear, KINDS};
use crate::settings::MIN_WINDOW;
use crate::window::{fmt_num, kind_msg, material_msg, wear_text, SortKey, Tab, WearFilter, Window};
use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, Stroke};
use std::path::PathBuf;
use std::sync::Arc;

// the colours of the theme Forge, the one the other programs of the suite start with: amber on charcoal
const PANEL: Color32 = Color32::from_rgb(0x17, 0x13, 0x10);
const RAISED: Color32 = Color32::from_rgb(0x22, 0x1c, 0x17);
const TEXT: Color32 = Color32::from_rgb(0xf4, 0xec, 0xe0);
const DIM: Color32 = Color32::from_rgb(0x9c, 0x8d, 0x7a);
const ACCENT: Color32 = Color32::from_rgb(0xff, 0xb0, 0x00);
const OK: Color32 = Color32::from_rgb(0x9b, 0xe3, 0x4a);
const DANGER: Color32 = Color32::from_rgb(0xff, 0x54, 0x70);

const PROGRAM_NAME: &str = "EH3D-TOOL-MANAGER";

struct App {
    win: Window,
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).strong().color(ACCENT));
}

/// A label and a box to write in, side by side (two cells of a grid).
fn text_field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.label(label);
    ui.add(egui::TextEdit::singleline(value).desired_width(150.0));
}

/// A label and a number that is changed by dragging or by writing, kept inside its limits (one row of a grid).
fn number_row(ui: &mut egui::Ui, label: &str, value: &mut f64, limits: (f64, f64), speed: f64) {
    ui.label(label);
    ui.add(
        egui::DragValue::new(value)
            .speed(speed)
            .range(limits.0..=limits.1)
            .max_decimals(4),
    );
    ui.end_row();
}

impl App {
    fn new(win: Window) -> App {
        App { win }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let lang = self.win.lang;
        egui::Panel::top("top").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(PROGRAM_NAME)
                        .strong()
                        .size(18.0)
                        .color(ACCENT),
                );
                ui.separator();
                ui.selectable_value(&mut self.win.tab, Tab::Tools, Msg::TabTools.tr(lang));
                ui.selectable_value(&mut self.win.tab, Tab::Calc, Msg::TabCalc.tr(lang));
                if self.win.dirty {
                    ui.separator();
                    ui.label(egui::RichText::new(Msg::Unsaved.tr(lang)).color(ACCENT));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("language")
                        .selected_text(self.win.lang.name())
                        .show_ui(ui, |ui| {
                            for l in Lang::ALL {
                                ui.selectable_value(&mut self.win.lang, l, l.name());
                            }
                        });
                    ui.label(egui::RichText::new(Msg::Language.tr(lang)).color(DIM));
                });
            });
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let lang = self.win.lang;
        egui::Panel::bottom("status").show(ui, |ui| {
            let colour = if self.win.status.is_problem() {
                DANGER
            } else {
                OK
            };
            ui.label(egui::RichText::new(self.win.status.text(lang)).color(colour));
        });
    }

    fn files_section(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.horizontal(|ui| {
            ui.label(Msg::CribFile.tr(lang));
            ui.add(egui::TextEdit::singleline(&mut self.win.file_text).desired_width(420.0));
            if ui.button(Msg::Open.tr(lang)).clicked() {
                self.win.open();
            }
            if ui.button(Msg::Save.tr(lang)).clicked() {
                self.win.save();
            }
            if ui.button(Msg::NewCrib.tr(lang)).clicked() {
                self.win.new_crib();
            }
        });
        ui.horizontal(|ui| {
            ui.label(Msg::CsvFile.tr(lang));
            ui.add(egui::TextEdit::singleline(&mut self.win.csv_text).desired_width(420.0));
            if ui.button(Msg::ImportCsv.tr(lang)).clicked() {
                self.win.import_csv();
            }
            if ui.button(Msg::ExportCsv.tr(lang)).clicked() {
                self.win.export_csv();
            }
            ui.checkbox(&mut self.win.overwrite_csv, Msg::Overwrite.tr(lang));
        });
    }

    fn filters_section(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.horizontal_wrapped(|ui| {
            ui.label(Msg::Search.tr(lang));
            ui.add(egui::TextEdit::singleline(&mut self.win.search).desired_width(160.0));
            let kind_text = match self.win.kind_filter {
                Some(k) => kind_msg(k).tr(lang),
                None => Msg::AllKinds.tr(lang),
            };
            egui::ComboBox::from_id_salt("kind_filter")
                .selected_text(kind_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.win.kind_filter, None, Msg::AllKinds.tr(lang));
                    for k in KINDS {
                        ui.selectable_value(
                            &mut self.win.kind_filter,
                            Some(k),
                            kind_msg(k).tr(lang),
                        );
                    }
                });
            ui.label(Msg::WearLabel.tr(lang));
            egui::ComboBox::from_id_salt("wear_filter")
                .selected_text(self.win.wear_filter.msg().tr(lang))
                .show_ui(ui, |ui| {
                    for f in WearFilter::ALL {
                        ui.selectable_value(&mut self.win.wear_filter, f, f.msg().tr(lang));
                    }
                });
            ui.label(Msg::SortBy.tr(lang));
            egui::ComboBox::from_id_salt("sort_key")
                .selected_text(self.win.sort.msg().tr(lang))
                .show_ui(ui, |ui| {
                    for s in SortKey::ALL {
                        ui.selectable_value(&mut self.win.sort, s, s.msg().tr(lang));
                    }
                });
            ui.checkbox(&mut self.win.descending, Msg::Descending.tr(lang));
        });
    }

    fn table(&mut self, ui: &mut egui::Ui, lang: Lang) {
        let rows = self.win.visible();
        let total = self.win.crib.tools.len();
        let shown = fill(
            Msg::Showing.tr(lang),
            &[rows.len().to_string().as_str(), total.to_string().as_str()],
        );
        ui.label(egui::RichText::new(shown).color(DIM));
        if total == 0 {
            ui.label(egui::RichText::new(Msg::NoTools.tr(lang)).color(DIM));
        } else if rows.is_empty() {
            ui.label(egui::RichText::new(Msg::NoMatch.tr(lang)).color(DIM));
        }
        let selected = self.win.selected;
        let (mut choose, mut edit, mut remove): (Option<u32>, Option<u32>, Option<u32>) =
            (None, None, None);
        egui::Grid::new("tools")
            .striped(true)
            .num_columns(11)
            .show(ui, |ui| {
                for head in [
                    Msg::ColId,
                    Msg::ColName,
                    Msg::KindLabel,
                    Msg::ColDiameter,
                    Msg::ColFlutes,
                    Msg::ColAngle,
                    Msg::ColHours,
                    Msg::ColHoles,
                    Msg::WearLabel,
                ] {
                    ui.label(egui::RichText::new(head.tr(lang)).color(DIM));
                }
                ui.label("");
                ui.label("");
                ui.end_row();
                for &i in &rows {
                    let Some(t) = self.win.crib.tools.get(i) else {
                        continue;
                    };
                    if ui
                        .selectable_label(selected == Some(t.id), t.id.to_string())
                        .clicked()
                    {
                        choose = Some(t.id);
                    }
                    ui.label(t.name.as_str());
                    ui.label(kind_msg(t.kind).tr(lang));
                    ui.label(fmt_num(t.diameter_mm));
                    ui.label(t.flutes.to_string());
                    ui.label(t.tip_angle_deg.map(fmt_num).unwrap_or_default());
                    ui.label(fmt_num(t.hours));
                    ui.label(t.holes.to_string());
                    let wear = t.wear();
                    let colour = match wear {
                        Wear::Good(_) => OK,
                        Wear::Soon(_) => ACCENT,
                        Wear::Replace => DANGER,
                    };
                    ui.label(egui::RichText::new(wear_text(wear, lang)).color(colour));
                    if ui.small_button(Msg::Edit.tr(lang)).clicked() {
                        edit = Some(t.id);
                    }
                    if ui.small_button(Msg::Remove.tr(lang)).clicked() {
                        remove = Some(t.id);
                    }
                    ui.end_row();
                }
            });
        if let Some(id) = choose {
            self.win.selected = Some(id);
        }
        if let Some(id) = edit {
            self.win.edit(id);
        }
        if let Some(id) = remove {
            self.win.remove(id);
        }
    }

    fn form(&mut self, ui: &mut egui::Ui, lang: Lang) {
        let editing = self.win.draft.id != 0;
        let title = if editing {
            fill(
                Msg::FormEdit.tr(lang),
                &[self.win.draft.id.to_string().as_str()],
            )
        } else {
            Msg::FormNew.tr(lang).to_string()
        };
        heading(ui, &title);
        ui.separator();
        let d = &mut self.win.draft;
        egui::Grid::new("form").num_columns(4).show(ui, |ui| {
            text_field(ui, Msg::ColName.tr(lang), &mut d.name);
            ui.label(Msg::KindLabel.tr(lang));
            egui::ComboBox::from_id_salt("form_kind")
                .selected_text(kind_msg(d.kind).tr(lang))
                .show_ui(ui, |ui| {
                    for k in KINDS {
                        ui.selectable_value(&mut d.kind, k, kind_msg(k).tr(lang));
                    }
                });
            ui.end_row();
            text_field(ui, Msg::ColDiameter.tr(lang), &mut d.diameter_mm);
            text_field(ui, Msg::ColFlutes.tr(lang), &mut d.flutes);
            ui.end_row();
            text_field(ui, Msg::FTipAngle.tr(lang), &mut d.tip_angle_deg);
            text_field(ui, Msg::FLength.tr(lang), &mut d.length_mm);
            ui.end_row();
            text_field(ui, Msg::FShank.tr(lang), &mut d.shank_mm);
            text_field(ui, Msg::FMaterial.tr(lang), &mut d.material);
            ui.end_row();
            text_field(ui, Msg::ColHours.tr(lang), &mut d.hours);
            text_field(ui, Msg::FMeters.tr(lang), &mut d.meters);
            ui.end_row();
            text_field(ui, Msg::ColHoles.tr(lang), &mut d.holes);
            text_field(ui, Msg::FMaxHours.tr(lang), &mut d.max_hours);
            ui.end_row();
            text_field(ui, Msg::FMaxHoles.tr(lang), &mut d.max_holes);
            ui.end_row();
        });
        let apply_text = if editing {
            Msg::SaveChanges
        } else {
            Msg::AddTool
        };
        ui.horizontal(|ui| {
            if ui.button(apply_text.tr(lang)).clicked() {
                self.win.apply_draft();
            }
            if ui.button(Msg::ClearForm.tr(lang)).clicked() {
                self.win.clear_form();
            }
        });
    }

    fn use_section(&mut self, ui: &mut egui::Ui, lang: Lang) {
        let Some(id) = self.win.selected_tool().map(|t| t.id) else {
            return;
        };
        heading(
            ui,
            &fill(Msg::UseTitle.tr(lang), &[id.to_string().as_str()]),
        );
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(Msg::ColHours.tr(lang));
            ui.add(egui::TextEdit::singleline(&mut self.win.use_hours).desired_width(70.0));
            ui.label(Msg::FMeters.tr(lang));
            ui.add(egui::TextEdit::singleline(&mut self.win.use_meters).desired_width(70.0));
            ui.label(Msg::ColHoles.tr(lang));
            ui.add(egui::TextEdit::singleline(&mut self.win.use_holes).desired_width(70.0));
            if ui.button(Msg::AddUse.tr(lang)).clicked() {
                self.win.record_use();
            }
        });
    }

    fn tools_page(&mut self, ui: &mut egui::Ui, lang: Lang) {
        self.files_section(ui, lang);
        ui.separator();
        self.filters_section(ui, lang);
        ui.add_space(4.0);
        self.table(ui, lang);
        ui.add_space(12.0);
        self.form(ui, lang);
        ui.add_space(12.0);
        self.use_section(ui, lang);
    }

    fn calc_inputs(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.label(egui::RichText::new(Msg::CalcIntro.tr(lang)).color(DIM));
        let mut pick = None;
        egui::ComboBox::from_id_salt("calc_from_crib")
            .selected_text(Msg::FromCrib.tr(lang))
            .show_ui(ui, |ui| {
                for t in &self.win.crib.tools {
                    if ui
                        .selectable_label(false, format!("{} {}", t.id, t.name))
                        .clicked()
                    {
                        pick = Some(t.id);
                    }
                }
            });
        if let Some(id) = pick {
            self.win.selected = Some(id);
            self.win.calc_from_selected();
        }
        let c = &mut self.win.calc;
        egui::Grid::new("calc_inputs")
            .num_columns(4)
            .show(ui, |ui| {
                ui.label(Msg::KindLabel.tr(lang));
                egui::ComboBox::from_id_salt("calc_kind")
                    .selected_text(kind_msg(c.kind).tr(lang))
                    .show_ui(ui, |ui| {
                        for k in KINDS {
                            ui.selectable_value(&mut c.kind, k, kind_msg(k).tr(lang));
                        }
                    });
                text_field(ui, Msg::ColDiameter.tr(lang), &mut c.diameter_mm);
                ui.end_row();
                text_field(ui, Msg::ColFlutes.tr(lang), &mut c.flutes);
                text_field(ui, Msg::FTipAngle.tr(lang), &mut c.tip_angle_deg);
                ui.end_row();
                ui.label(Msg::WorkMaterial.tr(lang));
                egui::ComboBox::from_id_salt("calc_material")
                    .selected_text(material_msg(c.material).tr(lang))
                    .show_ui(ui, |ui| {
                        for m in MATERIALS {
                            ui.selectable_value(&mut c.material, m, material_msg(m).tr(lang));
                        }
                    });
                text_field(ui, Msg::MaxRpm.tr(lang), &mut c.max_rpm);
                ui.end_row();
            });
    }

    fn calc_result(&self, ui: &mut egui::Ui, lang: Lang) {
        heading(ui, Msg::Result.tr(lang));
        ui.separator();
        let calculation = match self.win.calc.compute() {
            Ok(c) => c,
            Err(e) => {
                ui.colored_label(DANGER, e.as_str());
                return;
            }
        };
        let cut = calculation.cut;
        egui::Grid::new("calc_result")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(egui::RichText::new(Msg::ResRpm.tr(lang)).color(DIM));
                ui.label(format!("{} rpm", fmt_num(cut.rpm)));
                ui.end_row();
                ui.label(egui::RichText::new(Msg::ResFeed.tr(lang)).color(DIM));
                ui.label(format!("{} mm/min", fmt_num(cut.feed_mm_min)));
                ui.end_row();
                ui.label(egui::RichText::new(Msg::ResPlunge.tr(lang)).color(DIM));
                ui.label(format!("{} mm/min", fmt_num(cut.plunge_mm_min)));
                ui.end_row();
                ui.label(egui::RichText::new(Msg::ResPass.tr(lang)).color(DIM));
                ui.label(format!("{} mm", fmt_num(cut.pass_depth_mm)));
                ui.end_row();
            });
        if cut.rpm_limited {
            ui.colored_label(ACCENT, Msg::SpindleLimits.tr(lang));
        }
        if calculation.trace.uses_point {
            ui.label(egui::RichText::new(Msg::PointNote.tr(lang)).color(DIM));
        }
        ui.add_space(8.0);
        heading(ui, Msg::Formulas.tr(lang));
        ui.separator();
        for line in calculation.explain(lang) {
            ui.label(egui::RichText::new(line).monospace());
        }
    }

    fn calc_params(&mut self, ui: &mut egui::Ui, lang: Lang) {
        let mut reset = false;
        ui.collapsing(Msg::Parameters.tr(lang), |ui| {
            let p = &mut self.win.calc.params;
            egui::Grid::new("calc_materials")
                .striped(true)
                .num_columns(3)
                .show(ui, |ui| {
                    ui.label("");
                    ui.label(egui::RichText::new(Msg::ParSurface.tr(lang)).color(DIM));
                    ui.label(egui::RichText::new(Msg::ParChip.tr(lang)).color(DIM));
                    ui.end_row();
                    for (k, m) in MATERIALS.into_iter().enumerate() {
                        ui.label(material_msg(m).tr(lang));
                        ui.add(
                            egui::DragValue::new(&mut p.surface_speed[k])
                                .speed(1.0)
                                .range(SURFACE_SPEED_LIMITS.0..=SURFACE_SPEED_LIMITS.1),
                        );
                        ui.add(
                            egui::DragValue::new(&mut p.chip_fraction[k])
                                .speed(0.001)
                                .range(CHIP_FRACTION_LIMITS.0..=CHIP_FRACTION_LIMITS.1)
                                .max_decimals(4),
                        );
                        ui.end_row();
                    }
                });
            egui::Grid::new("calc_factors")
                .num_columns(2)
                .show(ui, |ui| {
                    number_row(
                        ui,
                        Msg::ParWorkingMin.tr(lang),
                        &mut p.working_min_mm,
                        WORKING_LIMITS,
                        0.01,
                    );
                    number_row(
                        ui,
                        Msg::ParWorkingMax.tr(lang),
                        &mut p.working_max_mm,
                        WORKING_LIMITS,
                        0.01,
                    );
                    number_row(
                        ui,
                        Msg::ParMinWorking.tr(lang),
                        &mut p.min_working_mm,
                        WORKING_LIMITS,
                        0.01,
                    );
                    number_row(
                        ui,
                        Msg::ParDrillPlunge.tr(lang),
                        &mut p.drill_plunge,
                        DRILL_PLUNGE_LIMITS,
                        0.001,
                    );
                    number_row(
                        ui,
                        Msg::ParPlungeRatio.tr(lang),
                        &mut p.plunge_ratio,
                        PLUNGE_RATIO_LIMITS,
                        0.01,
                    );
                    number_row(
                        ui,
                        Msg::ParPassFraction.tr(lang),
                        &mut p.pass_fraction,
                        PASS_FRACTION_LIMITS,
                        0.01,
                    );
                    number_row(
                        ui,
                        Msg::ParMaxPass.tr(lang),
                        &mut p.max_pass_mm,
                        MAX_PASS_LIMITS,
                        0.05,
                    );
                });
            if ui.button(Msg::ResetParams.tr(lang)).clicked() {
                reset = true;
            }
        });
        if reset {
            self.win.reset_params();
        }
    }

    fn calc_page(&mut self, ui: &mut egui::Ui, lang: Lang) {
        self.calc_inputs(ui, lang);
        ui.add_space(8.0);
        self.calc_result(ui, lang);
        ui.add_space(8.0);
        self.calc_params(ui, lang);
    }

    fn centre(&mut self, ui: &mut egui::Ui) {
        let lang = self.win.lang;
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.win.tab {
                Tab::Tools => self.tools_page(ui, lang),
                Tab::Calc => self.calc_page(ui, lang),
            });
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.win.save();
        }
        self.top_bar(ui);
        self.status_bar(ui);
        self.centre(ui);
    }
}

/// The icon of the program for the window, from the file of the brand.
fn window_icon() -> Option<egui::IconData> {
    let bytes: &[u8] = include_bytes!("../images/brand/png/icon-256.png");
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    buf.truncate(info.buffer_size());
    Some(egui::IconData {
        rgba: buf,
        width: info.width,
        height: info.height,
    })
}

/// The fonts of the system that draw Japanese and Chinese are added to the ones of egui, which have no such letters.
fn install_fonts(ctx: &egui::Context) {
    let candidates = [
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/YuGothM.ttc",
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/simsun.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/System/Library/Fonts/PingFang.ttc",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = FontDefinitions::default();
            fonts
                .font_data
                .insert("cjk".into(), Arc::new(FontData::from_owned(bytes)));
            for family in [FontFamily::Monospace, FontFamily::Proportional] {
                fonts.families.entry(family).or_default().push("cjk".into());
            }
            ctx.set_fonts(fonts);
            return;
        }
    }
}

/// The colours of the theme Forge for the controls of egui.
fn apply_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = PANEL;
    v.window_fill = RAISED;
    v.extreme_bg_color = Color32::from_rgb(0x14, 0x11, 0x0e);
    v.override_text_color = Some(TEXT);
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(0xff, 0xb0, 0x00, 110);
    v.selection.stroke = Stroke::new(1.5, ACCENT);
    v.widgets.inactive.bg_stroke =
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(0xff, 0xb0, 0x00, 110));
    v.widgets.inactive.weak_bg_fill = RAISED;
    v.widgets.hovered.bg_stroke = Stroke::new(1.5, ACCENT);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x33, 0x28, 0x1c);
    v.widgets.active.bg_stroke = Stroke::new(2.0, ACCENT);
    v.widgets.inactive.corner_radius = 6.0.into();
    v.widgets.hovered.corner_radius = 6.0.into();
    v.widgets.active.corner_radius = 6.0.into();
    ctx.set_visuals(v);
}

/// Opens the window for the program whose files are in `data_dir`. The size of the window, the language and the numbers of the calculator come from `tools.cfg` there.
pub fn run(data_dir: PathBuf) -> eframe::Result {
    let win = Window::new(data_dir);
    let size = [
        win.settings.window_width as f32,
        win.settings.window_height as f32,
    ];
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(size)
        .with_min_inner_size([MIN_WINDOW.0 as f32, MIN_WINDOW.1 as f32])
        .with_title(PROGRAM_NAME);
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        PROGRAM_NAME,
        options,
        Box::new(move |cc| {
            install_fonts(&cc.egui_ctx);
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(App::new(win)))
        }),
    )
}
