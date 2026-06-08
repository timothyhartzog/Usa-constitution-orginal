use dioxus::prelude::*;

use crate::state::use_archive;

const REGION_COLORS: &[(&str, &str)] = &[
    ("Africa", "#c4452f"),
    ("Americas", "#2563eb"),
    ("Asia", "#d97706"),
    ("Europe", "#16a34a"),
    ("Oceania", "#7c3aed"),
];

fn region_color(region: &str) -> &'static str {
    REGION_COLORS
        .iter()
        .find(|(r, _)| *r == region)
        .map(|(_, c)| *c)
        .unwrap_or("#888")
}

#[component]
pub fn WorldMapSvg(selected_country: Option<String>) -> Element {
    let archive_state = use_archive();
    let state = archive_state.read();

    let counts_by_region: Vec<(String, usize)> = {
        let mut map = std::collections::HashMap::new();
        for m in &state.world_meta {
            *map.entry(m.region.clone()).or_insert(0) += 1;
        }
        let mut v: Vec<_> = map.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    };

    let selected_region = selected_country.as_ref().and_then(|c| {
        state
            .world_meta
            .iter()
            .find(|m| m.country_id == *c)
            .map(|m| m.region.clone())
    });

    rsx! {
        div { class: "world-map-container",
            svg {
                width: "100%",
                height: "400",
                view_box: "0 0 700 400",
                class: "world-map-svg",

                rect { x: "0", y: "0", width: "700", height: "400", fill: "#f0f4f8", rx: "8" }

                // Stylized continent outlines (very rough)
                // Africa
                path {
                    d: "M 360 180 Q 380 160 410 170 Q 430 200 425 250 Q 415 290 380 295 Q 350 290 345 240 Z",
                    fill: "#e8eef3", stroke: "#d0d8e0", stroke_width: "1",
                }
                // Americas (rough)
                path {
                    d: "M 100 100 Q 110 80 140 95 Q 160 130 150 180 Q 145 220 170 280 Q 160 320 130 310 Q 115 270 120 220 Q 95 180 100 100 Z",
                    fill: "#e8eef3", stroke: "#d0d8e0", stroke_width: "1",
                }
                // Asia
                path {
                    d: "M 460 100 Q 530 80 580 110 Q 600 150 580 200 Q 540 230 480 220 Q 450 180 460 100 Z",
                    fill: "#e8eef3", stroke: "#d0d8e0", stroke_width: "1",
                }
                // Europe
                path {
                    d: "M 360 100 Q 420 90 460 110 Q 460 150 410 160 Q 370 155 360 130 Z",
                    fill: "#e8eef3", stroke: "#d0d8e0", stroke_width: "1",
                }
                // Oceania
                path {
                    d: "M 555 300 Q 595 290 615 310 Q 615 340 580 340 Q 555 330 555 300 Z",
                    fill: "#e8eef3", stroke: "#d0d8e0", stroke_width: "1",
                }

                // Region bubbles
                {
                    let positions: &[(&str, f64, f64)] = &[
                        ("Africa", 385.0, 230.0),
                        ("Americas", 130.0, 200.0),
                        ("Asia", 525.0, 160.0),
                        ("Europe", 405.0, 125.0),
                        ("Oceania", 580.0, 320.0),
                    ];
                    rsx! {
                        for &(region, cx, cy) in positions {
                            {
                                let count = counts_by_region
                                    .iter()
                                    .find(|(r, _)| r == region)
                                    .map(|(_, c)| *c)
                                    .unwrap_or(0);
                                let r = 18.0 + (count as f64).sqrt() * 5.0;
                                let color = region_color(region);
                                let is_active = selected_region.as_deref() == Some(region);
                                let stroke = if is_active { "#000" } else { color };
                                let stroke_w = if is_active { 3.0 } else { 1.0 };
                                let opacity = if selected_region.is_some() && !is_active { 0.35 } else { 0.55 };
                                rsx! {
                                    circle {
                                        cx: "{cx}", cy: "{cy}", r: "{r}",
                                        fill: "{color}",
                                        stroke: "{stroke}",
                                        stroke_width: "{stroke_w}",
                                        opacity: "{opacity}",
                                    }
                                    text {
                                        x: "{cx}", y: "{cy - 3.0}",
                                        text_anchor: "middle",
                                        font_size: "13",
                                        font_weight: "700",
                                        fill: "#fff",
                                        "{region}"
                                    }
                                    text {
                                        x: "{cx}", y: "{cy + 12.0}",
                                        text_anchor: "middle",
                                        font_size: "11",
                                        fill: "#fff",
                                        "{count} constitutions"
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(ref country_id) = selected_country {
                    {
                        let country_label = country_id.replace('_', " ");
                        rsx! {
                            text {
                                x: "350",
                                y: "385",
                                text_anchor: "middle",
                                font_size: "13",
                                font_weight: "600",
                                fill: "#28483a",
                                "Selected: {country_label}"
                            }
                        }
                    }
                }
            }

            div { class: "map-legend",
                for &(region, color) in REGION_COLORS {
                    span { class: "legend-item",
                        span {
                            class: "legend-dot",
                            style: "background: {color};",
                        }
                        "{region}"
                    }
                }
            }
        }
    }
}
