use eframe::egui;
use hmir_core::telemetry::TelemetryEvent;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub name: String,
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    CommandCenter,
    ModelVault,
    SystemLogs,
}

pub enum DashboardCmd {
    RefreshModels,
    SwitchModel(String),
    RestartNode,
    ToggleNode(bool),
    OpenDir(String),
    BrowseModel,
    DownloadModel {
        repo_id: String,
        folder_name: String,
    },
    DismountModel,
}

pub struct DashboardApp {
    telemetry_receiver: broadcast::Receiver<TelemetryEvent>,
    cmd_sender: mpsc::Sender<DashboardCmd>,
    current_tab: Tab,
    mini_mode: bool,

    // Telemetry
    live_temp: f64,
    live_ram: f64,
    live_ram_total: f64,
    live_vram: f64,
    live_vram_total: f64,
    live_vram_dedicated: f64,
    live_vram_shared: f64,
    live_npu_vram: f64,
    live_tps: f64,
    live_npu: f64,
    live_uptime: u64,
    live_kv: f32,
    live_disk_free: f64,
    live_disk_total: f64,
    live_gpu_temp: f64,

    // Hardware Names
    cpu_name: String,
    gpu_name: String,
    npu_name: String,

    // Download Tracking
    dl_active: bool,
    dl_model: String,
    dl_status: String,
    dl_progress: f32,

    active_model: String,
    installed_models: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    api_active: bool,
    selected_model_in_drop: String,
    raw_logs: String,

    // Technical Specs
    cpu_cores: u32,
    cpu_threads: u32,
    cpu_l3: f64,
    gpu_driver: String,
    npu_driver: String,
    ram_speed: u32,
    disk_model: String,
}

impl DashboardApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        rx: broadcast::Receiver<TelemetryEvent>,
        cmd_tx: mpsc::Sender<DashboardCmd>,
        models_shared: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    ) -> Self {
        let mut log_dir = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        log_dir.push("hmir");
        log_dir.push("logs");
        let _ = std::fs::create_dir_all(&log_dir);

        let mut style = (*_cc.egui_ctx.style()).clone();
        style.visuals.window_rounding = 8.0.into();
        style.visuals.window_fill = egui::Color32::from_rgb(10, 10, 12);
        style.visuals.panel_fill = egui::Color32::from_rgb(10, 10, 12);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(15, 15, 18);
        style.visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_gray(60));

        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(25, 25, 30);
        style.visuals.widgets.inactive.rounding = 6.0.into();
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 40, 45);
        style.visuals.widgets.hovered.rounding = 6.0.into();
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 200, 255);
        style.visuals.widgets.active.rounding = 6.0.into();

        style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 180, 240);
        _cc.egui_ctx.set_style(style);

        Self {
            telemetry_receiver: rx,
            cmd_sender: cmd_tx,
            current_tab: Tab::CommandCenter,
            mini_mode: false,
            live_temp: 0.0,
            live_vram: 0.0,
            live_vram_total: 0.1,
            live_vram_dedicated: 0.0,
            live_vram_shared: 0.0,
            live_npu_vram: 0.0,
            live_tps: 0.0,
            live_npu: 0.0,
            live_uptime: 0,
            live_kv: 0.0,
            live_ram: 0.0,
            live_ram_total: 0.1,
            live_gpu_temp: 0.0,
            live_disk_free: 0.0,
            live_disk_total: 0.1,
            cpu_name: "Detecting...".into(),
            gpu_name: "Detecting...".into(),
            npu_name: "Detecting...".into(),
            dl_active: false,
            dl_model: String::new(),
            dl_status: String::new(),
            dl_progress: 0.0,
            active_model: "NONE".to_string(),
            installed_models: models_shared,
            api_active: false,
            selected_model_in_drop: String::new(),
            raw_logs: "No logs loaded...".to_string(),
            cpu_cores: 0,
            cpu_threads: 0,
            cpu_l3: 0.0,
            gpu_driver: "Unknown".into(),
            npu_driver: "Unknown".into(),
            ram_speed: 0,
            disk_model: "Unknown".into(),
        }
    }

    fn draw_metric_card(ui: &mut egui::Ui, title: &str, value: &str, color: egui::Color32) {
        let card_color = egui::Color32::from_rgb(20, 20, 24);
        egui::Frame::none()
            .fill(card_color)
            .rounding(8.0)
            .inner_margin(12.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 35, 40)))
            .show(ui, |ui| {
                ui.set_min_width(120.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(title)
                            .size(11.0)
                            .color(egui::Color32::from_gray(140))
                            .strong(),
                    );
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(value).size(26.0).strong().color(color));
                });
            });
    }

    fn draw_pill_status(ui: &mut egui::Ui, active: bool) {
        let (text, color, bg) = if active {
            (
                "â— SYSTEM ONLINE",
                egui::Color32::from_rgb(0, 255, 128),
                egui::Color32::from_rgb(0, 40, 20),
            )
        } else {
            (
                "â— SYSTEM OFFLINE",
                egui::Color32::from_rgb(255, 80, 80),
                egui::Color32::from_rgb(40, 10, 10),
            )
        };

        egui::Frame::none()
            .fill(bg)
            .rounding(12.0)
            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(text).size(11.0).color(color).strong());
            });
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(event) = self.telemetry_receiver.try_recv() {
            match event {
                TelemetryEvent::HardwareState {
                    cpu_temp,
                    gpu_temp,
                    ram_used,
                    tps,
                    npu_util,
                    vram_used,
                    vram_total,
                    gpu_vram_dedicated,
                    gpu_vram_shared,
                    npu_vram_used,
                    node_uptime,
                    kv_cache,
                    cpu_name,
                    gpu_name,
                    npu_name,
                    ram_total,
                    disk_free,
                    engine_status,
                    cpu_cores,
                    cpu_threads,
                    cpu_l3_cache_mb,
                    gpu_driver,
                    npu_driver,
                    disk_total,
                    disk_model,
                    ram_speed_mts,
                    ..
                } => {
                    self.active_model =
                        if engine_status == "Ejected" || engine_status == "Unmounted" {
                            "NONE".to_string()
                        } else {
                            self.active_model.clone()
                        };
                    self.live_temp = cpu_temp;
                    self.live_gpu_temp = gpu_temp;
                    self.live_ram = ram_used;
                    self.live_ram_total = ram_total;
                    self.live_vram = vram_used;
                    self.live_vram_total = vram_total;
                    self.live_vram_dedicated = gpu_vram_dedicated;
                    self.live_vram_shared = gpu_vram_shared;
                    self.live_npu_vram = npu_vram_used;
                    self.live_tps = tps;
                    self.live_npu = npu_util;
                    self.live_uptime = node_uptime;
                    self.live_kv = kv_cache;
                    self.cpu_name = cpu_name;
                    self.gpu_name = gpu_name;
                    self.npu_name = npu_name;
                    self.live_disk_free = disk_free;
                    self.live_disk_total = disk_total;
                    self.cpu_cores = cpu_cores;
                    self.cpu_threads = cpu_threads;
                    self.cpu_l3 = cpu_l3_cache_mb;
                    self.gpu_driver = gpu_driver;
                    self.npu_driver = npu_driver;
                    self.ram_speed = ram_speed_mts;
                    self.disk_model = disk_model;
                    self.api_active = true;
                }
                TelemetryEvent::DownloadStatus {
                    model,
                    status,
                    progress,
                } => {
                    self.dl_active = status != "Completed" && status != "Failed";
                    self.dl_model = model;
                    self.dl_status = status;
                    self.dl_progress = progress;
                }
                TelemetryEvent::ModelMounted { name, .. } => {
                    self.active_model = name.to_uppercase();
                    self.api_active = true;
                }
                _ => {}
            }
        }

        // Left Navigation Rail
        egui::SidePanel::left("nav_v2")
            .width_range(70.0..=70.0)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(12, 12, 15))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 25, 30))),
            )
            .show(ctx, |ui| {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    if ui
                        .selectable_label(
                            self.current_tab == Tab::CommandCenter,
                            egui::RichText::new("ðŸ“Š").size(24.0),
                        )
                        .on_hover_text("Command Center")
                        .clicked()
                    {
                        self.current_tab = Tab::CommandCenter;
                    }
                    ui.add_space(25.0);
                    if ui
                        .selectable_label(
                            self.current_tab == Tab::ModelVault,
                            egui::RichText::new("ðŸ“¦").size(24.0),
                        )
                        .on_hover_text("Model Vault")
                        .clicked()
                    {
                        self.current_tab = Tab::ModelVault;
                    }
                    ui.add_space(25.0);
                    if ui
                        .selectable_label(
                            self.current_tab == Tab::SystemLogs,
                            egui::RichText::new("ðŸ“„").size(24.0),
                        )
                        .on_hover_text("System Logs")
                        .clicked()
                    {
                        self.current_tab = Tab::SystemLogs;
                    }

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        let icon = if self.mini_mode {
                            "ðŸ—–"
                        } else {
                            "ðŸ——"
                        };
                        if ui.button(egui::RichText::new(icon).size(18.0)).clicked() {
                            self.mini_mode = !self.mini_mode;
                            let new_size = if self.mini_mode {
                                egui::vec2(400.0, 600.0)
                            } else {
                                egui::vec2(850.0, 600.0)
                            };
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(new_size));
                        }
                        ui.add_space(15.0);
                    });
                });
            });

        // Top Header
        egui::TopBottomPanel::top("header_v2")
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(18, 18, 22))
                    .inner_margin(egui::Margin::symmetric(15.0, 10.0))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 30, 35))),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/logo_small.png"))
                            .max_width(24.0),
                    );
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new("HMIR ELITE")
                            .size(16.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.add_space(15.0);
                    Self::draw_pill_status(ui, self.api_active);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let start_stop_text = if self.api_active {
                            "â¹ STOP"
                        } else {
                            "â–¶ START"
                        };
                        let btn_color = if self.api_active {
                            egui::Color32::from_rgb(200, 50, 50)
                        } else {
                            egui::Color32::from_rgb(0, 180, 100)
                        };
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(start_stop_text)
                                        .color(egui::Color32::WHITE)
                                        .strong(),
                                )
                                .fill(btn_color),
                            )
                            .clicked()
                        {
                            self.api_active = !self.api_active;
                            let _ = self
                                .cmd_sender
                                .try_send(DashboardCmd::ToggleNode(self.api_active));
                        }

                        ui.add_space(10.0);
                        if ui
                            .button(egui::RichText::new("ðŸ”„ Restart").strong())
                            .clicked()
                        {
                            let _ = self.cmd_sender.try_send(DashboardCmd::RestartNode);
                        }

                        ui.add_space(5.0);
                        if ui
                            .button(egui::RichText::new("ðŸ’¬ Launch Web Chat").strong())
                            .on_hover_text("Open Unified Web Portal (8080)")
                            .clicked()
                        {
                            let _ = std::process::Command::new("powershell")
                                .arg("-Command")
                                .arg("Start-Process 'http://127.0.0.1:8080'")
                                .spawn();
                        }
                    });
                });
            });

        // Main Content Area
        egui::CentralPanel::default().frame(egui::Frame::none().fill(egui::Color32::from_rgb(8, 8, 10)).inner_margin(20.0)).show(ctx, |ui| {
            match self.current_tab {
                Tab::CommandCenter => {
                    // SECTION 1: TOP-LEVEL SLICING
                    ui.horizontal(|ui| {
                        Self::draw_metric_card(ui, "THROUGHPUT", &format!("{:.1} T/s", self.live_tps), egui::Color32::from_rgb(0, 240, 255));
                        ui.add_space(10.0);
                        Self::draw_metric_card(ui, "AI BOOST", &format!("{:.0}%", self.live_npu), egui::Color32::from_rgb(0, 255, 150));
                        ui.add_space(10.0);
                        Self::draw_metric_card(ui, "THERMALS", &format!("{:.1}Â°C", self.live_temp), if self.live_temp > 85.0 { egui::Color32::RED } else { egui::Color32::from_gray(200) });
                        ui.add_space(10.0);
                        Self::draw_metric_card(ui, "KV CACHE", &format!("{:.1}%", self.live_kv), egui::Color32::from_rgb(200, 150, 0));
                    });

                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("SILICON HEALTH TILES (GiB)").size(14.0).color(egui::Color32::from_gray(160)).strong());
                    ui.add_space(10.0);

                    // SECTION 2: SILICON HEALTH TILES
                    ui.horizontal(|ui| {
                        // CPU TILE
                        egui::Frame::none().fill(egui::Color32::from_rgb(18, 18, 22)).rounding(8.0).inner_margin(12.0).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 35, 40))).show(ui, |ui| {
                            ui.set_min_width(260.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("CENTRAL PROCESSOR").size(11.0).color(egui::Color32::from_gray(120)).strong());
                                ui.label(egui::RichText::new(&self.cpu_name).strong().color(egui::Color32::WHITE).size(13.0));
                                ui.label(egui::RichText::new(format!("{} Cores / {} Threads | {:.1} MB L3", self.cpu_cores, self.cpu_threads, self.cpu_l3)).size(10.0).color(egui::Color32::from_gray(100)));
                                ui.add_space(8.0);

                                let ram_pct = (self.live_ram / self.live_ram_total.max(0.1)) as f32;
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("SYSTEM RAM ({} MT/s)", self.ram_speed)).size(10.0).color(egui::Color32::GRAY));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(format!("{:.1} / {:.1} GiB", self.live_ram / 1024.0 / 1024.0 / 1024.0, self.live_ram_total / 1024.0 / 1024.0 / 1024.0)).size(11.0).strong());
                                    });
                                });
                                ui.add_space(4.0);
                                ui.add_sized([ui.available_width(), 4.0], egui::ProgressBar::new(ram_pct).fill(egui::Color32::from_rgb(200, 200, 200)));
                            });
                        });

                        ui.add_space(10.0);

                        // GPU TILE
                        egui::Frame::none().fill(egui::Color32::from_rgb(18, 18, 22)).rounding(8.0).inner_margin(12.0).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 35, 40))).show(ui, |ui| {
                            ui.set_min_width(260.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("GRAPHICS ENGINE").size(11.0).color(egui::Color32::from_gray(120)).strong());
                                ui.label(egui::RichText::new(&self.gpu_name).strong().color(egui::Color32::WHITE).size(13.0));
                                ui.label(egui::RichText::new(format!("Driver: {}", self.gpu_driver)).size(10.0).color(egui::Color32::from_gray(100)));
                                ui.add_space(8.0);

                                // Dedicated VRAM
                                let ded_pct = (self.live_vram_dedicated / self.live_vram_total.max(0.1)) as f32;
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("DEDICATED").size(10.0).color(egui::Color32::GRAY));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(format!("{:.1} / {:.1} GiB", self.live_vram_dedicated / 1024.0 / 1024.0 / 1024.0, self.live_vram_total / 1024.0 / 1024.0 / 1024.0)).size(11.0).strong().color(egui::Color32::from_rgb(180, 100, 255)));
                                    });
                                });
                                ui.add_space(2.0);
                                ui.add_sized([ui.available_width(), 4.0], egui::ProgressBar::new(ded_pct).fill(egui::Color32::from_rgb(180, 100, 255)));

                                ui.add_space(6.0);
                                // Shared VRAM
                                let shr_pct = (self.live_vram_shared / self.live_ram_total.max(0.1)) as f32;
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("SHARED").size(10.0).color(egui::Color32::GRAY));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(format!("{:.1} GiB", self.live_vram_shared / 1024.0 / 1024.0 / 1024.0)).size(11.0).strong().color(egui::Color32::from_rgb(100, 150, 255)));
                                    });
                                });
                                ui.add_sized([ui.available_width(), 4.0], egui::ProgressBar::new(shr_pct).fill(egui::Color32::from_rgb(100, 150, 255)));
                            });
                        });

                        ui.add_space(10.0);

                        // NPU TILE
                        egui::Frame::none().fill(egui::Color32::from_rgb(18, 18, 22)).rounding(8.0).inner_margin(12.0).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 35, 40))).show(ui, |ui| {
                            ui.set_min_width(260.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("NEURAL ACCELERATOR").size(11.0).color(egui::Color32::from_gray(120)).strong());
                                ui.label(egui::RichText::new(&self.npu_name).strong().color(egui::Color32::from_rgb(0, 255, 150)).size(13.0));
                                ui.label(egui::RichText::new(format!("Driver: {}", self.npu_driver)).size(10.0).color(egui::Color32::from_gray(100)));
                                ui.add_space(8.0);

                                let npu_pct = (self.live_npu_vram / self.live_ram_total.max(0.1)) as f32;
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("ALLOCATED POOL").size(10.0).color(egui::Color32::GRAY));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(format!("{:.1} GiB", self.live_npu_vram / 1024.0 / 1024.0 / 1024.0)).size(11.0).strong().color(egui::Color32::from_rgb(0, 255, 150)));
                                    });
                                });
                                ui.add_space(4.0);
                                ui.add_sized([ui.available_width(), 4.0], egui::ProgressBar::new(npu_pct).fill(egui::Color32::from_rgb(0, 255, 150)));
                            });
                        });

                        ui.add_space(10.0);

                        // STORAGE TILE
                        egui::Frame::none().fill(egui::Color32::from_rgb(18, 18, 22)).rounding(8.0).inner_margin(12.0).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 35, 40))).show(ui, |ui| {
                            ui.set_min_width(260.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("STORAGE VAULT").size(11.0).color(egui::Color32::from_gray(120)).strong());
                                ui.label(egui::RichText::new(&self.disk_model).strong().color(egui::Color32::WHITE).size(13.0));
                                ui.add_space(8.0);

                                let disk_pct = ((self.live_disk_total - self.live_disk_free) / self.live_disk_total.max(0.1)) as f32;
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("AVAILABLE").size(10.0).color(egui::Color32::GRAY));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(format!("{:.1} / {:.1} GB", self.live_disk_free, self.live_disk_total)).size(11.0).strong().color(egui::Color32::from_rgb(0, 200, 255)));
                                    });
                                });
                                ui.add_space(4.0);
                                ui.add_sized([ui.available_width(), 4.0], egui::ProgressBar::new(disk_pct).fill(egui::Color32::from_rgb(0, 200, 255)));
                            });
                        });
                    });

                    if self.dl_active {
                        ui.add_space(15.0);
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(30, 20, 10))
                            .rounding(6.0)
                            .inner_margin(10.0)
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 150, 0)))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("ðŸ“¥").size(16.0));
                                    ui.label(egui::RichText::new(format!("Downloading {}: {}", self.dl_model, self.dl_status)).color(egui::Color32::WHITE));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add(egui::Spinner::new());
                                    });
                                });
                            });
                    }

                    ui.add_space(25.0);
                    ui.separator();
                    ui.add_space(15.0);

                    // Orchestration Controls
                    ui.label(egui::RichText::new("ORCHESTRATION LAYER").size(14.0).color(egui::Color32::from_gray(160)).strong());
                    ui.add_space(10.0);

                    egui::Frame::none().fill(egui::Color32::from_rgb(18, 18, 22)).rounding(8.0).inner_margin(15.0).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 30, 35))).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let models = self.installed_models.lock().unwrap().clone();
                            if self.selected_model_in_drop.is_empty() && !models.is_empty() {
                                self.selected_model_in_drop = models[0].clone();
                            }

                        ui.columns(2, |cols| {
                            // Column 1: Engine Select
                            cols[0].vertical(|ui| {
                                ui.label(egui::RichText::new("TARGET ENGINE").size(11.0).color(egui::Color32::GRAY));
                                egui::ComboBox::from_id_source("drop_v2")
                                    .selected_text(&self.selected_model_in_drop)
                                    .width(ui.available_width() - 10.0)
                                    .show_ui(ui, |ui| {
                                        for model in models {
                                            ui.selectable_value(&mut self.selected_model_in_drop, model.clone(), &model);
                                        }
                                    });

                                ui.add_space(10.0);
                                if ui.add_sized([ui.available_width(), 32.0], egui::Button::new(egui::RichText::new("âš¡ MOUNT").strong().color(egui::Color32::BLACK)).fill(egui::Color32::from_rgb(0, 200, 255))).clicked() && !self.selected_model_in_drop.is_empty() {
                                    let _ = self.cmd_sender.try_send(DashboardCmd::SwitchModel(self.selected_model_in_drop.clone()));
                                }
                            });

                            // Column 2: Unmount / Eject
                            cols[1].vertical(|ui| {
                                ui.label(egui::RichText::new("ORCHESTRATION").size(11.0).color(egui::Color32::GRAY));
                                ui.add_space(2.0);
                                if ui.add_sized([ui.available_width(), 32.0], egui::Button::new(egui::RichText::new("â UNMOUNT ACTIVE").strong().color(egui::Color32::WHITE)).fill(egui::Color32::from_rgb(60, 60, 70))).clicked() {
                                    let _ = self.cmd_sender.try_send(DashboardCmd::DismountModel);
                                }
                                ui.add_space(10.0);
                                ui.label(egui::RichText::new("Active Engine is released to background pool.").size(9.0).italics().color(egui::Color32::from_gray(80)));
                            });
                        });
                        });

                        ui.add_space(15.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("ACTIVE:").color(egui::Color32::GRAY).strong());
                            ui.label(egui::RichText::new(&self.active_model).color(egui::Color32::from_rgb(0, 255, 150)).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(format!("GPU: {:.1}Â°C", self.live_gpu_temp)).color(egui::Color32::from_rgb(180, 140, 255)));
                                ui.separator();
                                ui.label(egui::RichText::new(format!("CPU: {:.1}Â°C", self.live_temp)).color(egui::Color32::from_gray(180)));
                                ui.separator();
                                ui.label(egui::RichText::new(format!("Uptime: {}s", self.live_uptime)).color(egui::Color32::from_gray(180)));
                                ui.separator();
                                ui.label(egui::RichText::new(format!("KV Cache: {:.1}%", self.live_kv)).color(egui::Color32::from_gray(180)));
                            });
                        });
                    });
                }
                Tab::ModelVault => {
                    ui.horizontal(|ui| {
                        ui.heading(egui::RichText::new("MODEL VAULT").size(20.0).color(egui::Color32::WHITE));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new("ðŸ“ Explorer").strong()).clicked() { let _ = self.cmd_sender.try_send(DashboardCmd::BrowseModel); }
                        });
                    });
                    ui.add_space(15.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // SUGGESTIONS SECTION
                        ui.label(egui::RichText::new("RECOMMENDED FOR YOUR NPU").size(14.0).color(egui::Color32::from_rgb(0, 200, 255)).strong());
                        ui.add_space(10.0);

                        let is_apple = self.npu_name.to_lowercase().contains("apple");
                        let suggestions = if is_apple {
                            vec![
                                ("Llama 3.1 8B (MLX)", "mlx-community/Llama-3.1-8B-Instruct-4bit", "Llama-3.1-8B-MLX"),
                                ("Mistral Nemo (MLX)", "mlx-community/Mistral-Nemo-Instruct-2407-4bit", "Mistral-Nemo-MLX"),
                                ("Qwen 2.5 7B (MLX)", "mlx-community/Qwen2.5-7B-Instruct-4bit", "Qwen-2.5-7B-MLX"),
                            ]
                        } else {
                            vec![
                                ("Qwen 2.5 1.5B (INT4-OV)", "OpenVINO/qwen2.5-1.5b-instruct-int4-ov", "qwen2.5-1.5b-ov"),
                                ("Phi-3 Mini (INT4-OV)", "OpenVINO/phi-3-mini-4k-instruct-int4-ov", "phi-3-mini-ov"),
                                ("Llama 3.1 8B (INT4-OV)", "OpenVINO/llama-3.1-8b-instruct-int4-ov", "llama-3.1-8b-ov"),
                            ]
                        };

                        ui.horizontal_wrapped(|ui| {
                            for (name, repo, folder) in suggestions {
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(25, 25, 30))
                                    .rounding(8.0)
                                    .inner_margin(12.0)
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 45, 50)))
                                    .show(ui, |ui| {
                                        ui.set_max_width(240.0);
                                        ui.vertical(|ui| {
                                            ui.label(egui::RichText::new(name).strong().color(egui::Color32::WHITE));
                                            ui.label(egui::RichText::new("NPU Optimized").size(10.0).color(egui::Color32::GRAY));
                                            ui.add_space(10.0);
                                            if ui.add(egui::Button::new(egui::RichText::new("âš¡ INSTALL").strong().color(egui::Color32::BLACK)).fill(egui::Color32::from_rgb(0, 255, 128))).clicked() {
                                                let _ = self.cmd_sender.try_send(DashboardCmd::DownloadModel { repo_id: repo.to_string(), folder_name: folder.to_string() });
                                            }
                                        });
                                    });
                                ui.add_space(10.0);
                            }
                        });


                        ui.add_space(30.0);
                        ui.separator();
                        ui.add_space(15.0);
                        ui.label(egui::RichText::new("INSTALLED MODELS").size(14.0).color(egui::Color32::from_gray(160)).strong());
                        ui.add_space(10.0);

                        let models = self.installed_models.lock().unwrap().clone();
                        for model in &models {
                            egui::Frame::none().fill(egui::Color32::from_rgb(20, 20, 24)).rounding(6.0).inner_margin(12.0).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 35, 40))).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("ðŸ§ ").size(20.0));
                                    ui.add_space(10.0);
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(model.as_str()).color(egui::Color32::WHITE).strong());
                                        ui.label(egui::RichText::new("Local LLM Repository").size(11.0).color(egui::Color32::GRAY));
                                    });
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add(egui::Button::new(egui::RichText::new("âš¡ Load").strong())).clicked() {
                                            let _ = self.cmd_sender.try_send(DashboardCmd::SwitchModel(model.clone()));
                                        }
                                    });
                                });
                            });
                            ui.add_space(10.0);
                        }
                        if models.is_empty() {
                            ui.label(egui::RichText::new("No local models found.").color(egui::Color32::GRAY));
                        }
                    });
                }
                Tab::SystemLogs => {
                    ui.horizontal(|ui| {
                        ui.heading(egui::RichText::new("SYSTEM LOGS").size(20.0).color(egui::Color32::WHITE));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new("ðŸ“ Open Logs").strong()).clicked() { let _ = self.cmd_sender.try_send(DashboardCmd::OpenDir("logs".to_string())); }
                        });
                    });
                    ui.add_space(15.0);

                    let mut log_path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                    log_path.push("hmir");
                    log_path.push("logs");
                    log_path.push("api.log");

                    if let Ok(content) = std::fs::read_to_string(&log_path) {
                        let len = content.len();
                        let start = len.saturating_sub(4000);
                        self.raw_logs = content[start..].to_string();
                    }

                    egui::Frame::none().fill(egui::Color32::from_rgb(15, 15, 18)).rounding(6.0).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 30, 35))).show(ui, |ui| {
                        egui::ScrollArea::both().stick_to_bottom(true).show(ui, |ui| {
                            let mut text = self.raw_logs.clone();
                            ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut text)
                                .font(egui::TextStyle::Monospace)
                                .text_color(egui::Color32::from_rgb(180, 180, 190))
                                .frame(false));
                        });
                    });
                }
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([850.0, 600.0])
            .with_transparent(false),
        ..Default::default()
    };

    let (tx, rx) = broadcast::channel(1024);
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<DashboardCmd>(32);
    let models_shared = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let models_for_bg = models_shared.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let client = reqwest::Client::new();
            let client_cmd = client.clone();
            let models_cmd = models_for_bg;
            let tx_for_ping = tx.clone();

            tokio::spawn(async move {
                while let Some(cmd) = cmd_rx.recv().await {
                    match cmd {
                        DashboardCmd::SwitchModel(name) => {
                            let _ = client_cmd.post("http://127.0.0.1:8080/v1/engine/switch").json(&serde_json::json!({ "name": name })).send().await;
                        }
                        DashboardCmd::RestartNode => {
                            let _ = std::process::Command::new("taskkill").args(["/F", "/IM", "hmir-api.exe"]).output();
                            let mut hmir_exe = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                            hmir_exe.push(".hmir");
                            hmir_exe.push("hmir-api.exe");
                            let _ = std::process::Command::new("powershell").arg("-Command").arg(format!("Start-Process '{}' -WindowStyle Hidden", hmir_exe.display())).spawn();
                        }
                        DashboardCmd::ToggleNode(active) => {
                            if !active {
                                let _ = std::process::Command::new("taskkill").args(["/F", "/IM", "hmir-api.exe"]).output();
                            } else {
                                let mut hmir_exe = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                                hmir_exe.push(".hmir");
                                hmir_exe.push("hmir-api.exe");
                                let _ = std::process::Command::new("powershell").arg("-Command").arg(format!("Start-Process '{}' -WindowStyle Hidden", hmir_exe.display())).spawn();
                            }
                        }
                        DashboardCmd::OpenDir(target) => {
                            let mut base_path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                            base_path.push("hmir");
                            let path = if target == "models" { base_path.join("models") } else { base_path.join("logs") };
                            let _ = std::process::Command::new("explorer").arg(path).spawn();
                        }
                        DashboardCmd::BrowseModel => {
                            let mut base_path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                            base_path.push("hmir");
                            base_path.push("models");
                            let _ = std::process::Command::new("explorer").arg(base_path).spawn();
                        }
                        DashboardCmd::DownloadModel { repo_id, folder_name } => {
                            let _ = client_cmd.post("http://127.0.0.1:8080/v1/models/download")
                                .json(&serde_json::json!({ "repo_id": repo_id, "folder_name": folder_name }))
                                .send().await;
                        }
                        DashboardCmd::DismountModel => {
                            let _ = client_cmd.post("http://127.0.0.1:8080/v1/engine/eject").send().await;
                        }
                        DashboardCmd::RefreshModels => { /* handled by the periodic model polling loop below */ }
                    }
                }
            });

            let client_tel = client.clone();
            tokio::spawn(async move {
                loop {
                    if let Ok(response) = client_tel.get("http://127.0.0.1:8080/v1/telemetry").send().await {
                        let mut stream = response.bytes_stream();
                        use futures_util::StreamExt;
                        while let Some(item) = stream.next().await {
                            if let Ok(bytes) = item {
                                let data = String::from_utf8_lossy(&bytes);
                                for line in data.lines() {
                                    if let Some(stripped) = line.strip_prefix("data:") {
                                        let json_str = stripped.trim();
                                        if let Ok(event) = serde_json::from_str::<hmir_core::telemetry::TelemetryEvent>(json_str) {
                                            let _ = tx_for_ping.send(event);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                }
            });

            let client_models = reqwest::Client::new();
            loop {
                match client_models.get("http://127.0.0.1:8080/v1/models/installed").send().await {
                    Ok(res) => {
                        match res.json::<Vec<String>>().await {
                            Ok(models) => {
                                let mut guard = models_cmd.lock().unwrap();
                                *guard = models;
                            },
                            Err(e) => {
                                let log_dir = format!("{}\\hmir\\logs", std::env::var("LOCALAPPDATA").unwrap_or_default());
                                let _ = std::fs::create_dir_all(&log_dir);
                                let _ = std::fs::write(format!("{}\\dashboard_error.log", log_dir), format!("Dashboard Model Parse Error: {:?}", e));
                            }
                        }
                    },
                    Err(e) => {
                        let log_dir = format!("{}\\hmir\\logs", std::env::var("LOCALAPPDATA").unwrap_or_default());
                        let _ = std::fs::create_dir_all(&log_dir);
                        let _ = std::fs::write(format!("{}\\dashboard_error.log", log_dir), format!("Dashboard Network Error: {:?}", e));
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
            }
        });
    });

    eframe::run_native(
        "HMIR ELITE",
        options,
        Box::new(|cc| Box::new(DashboardApp::new(cc, rx, cmd_tx, models_shared))),
    )
}
