//! Explicitly invoked documentation asset generator; uses the real frontend widgets.
use super::*;
use std::fmt::Write as _;

#[test]
#[ignore = "regenerates docs/images/termatrix.svg; run explicitly for documentation updates"]
fn render_readme_preview() {
    let settings = Settings::default();
    let mut scene = Scene::new(42);
    let area = Rect::new(0, 0, 136, 42);
    let (panel, canvas) = regions(area, true);
    scene.resize(canvas.width as usize / 2, canvas.height as usize);
    let mut history = crate::metrics::DensityHistory::default();
    for tick in 0..4000 {
        scene.advance(0.01, &settings);
        history.sample(tick as f64 * 0.01, &scene);
    }
    let points = history.points(40.);
    let mut terminal =
        Terminal::new(ratatui::backend::TestBackend::new(area.width, area.height)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(Rain(&scene), canvas);
            let (controls, chart) = panel_regions(panel);
            frame.render_widget(
                Controls {
                    settings: &settings,
                    selected: 0,
                    traces: scene.traces.len(),
                    paused: false,
                    path: std::path::Path::new("~/.config/termatrix/profiles/default.toml"),
                    status: &SaveStatus::Config,
                    pending: false,
                },
                controls,
            );
            frame.render_widget(density_chart(&points, history.current()), chart.unwrap());
        })
        .unwrap();
    let buffer = terminal.backend().buffer();
    assert!(buffer.content.iter().any(|cell| cell
        .symbol()
        .chars()
        .any(|c| ('\u{2801}'..='\u{28ff}').contains(&c))));
    let mut svg=String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1360\" height=\"840\" viewBox=\"0 0 1360 840\"><title>termatrix green rain and live controls</title><desc>Deterministic preview rendered from the actual Ratatui buffer, seed 42 after 40 simulated seconds.</desc><rect width=\"1360\" height=\"840\" fill=\"#111111\"/><g font-family=\"monospace\" font-size=\"16\">");
    let rgb = |color: Color| match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (238, 238, 238),
    };
    for y in 0..area.height {
        let mut x = 0;
        while x < area.width {
            let cell = &buffer[(x, y)];
            if let Color::Rgb(r, g, b) = cell.bg {
                write!(svg,"<rect x=\"{}\" y=\"{}\" width=\"10\" height=\"20\" fill=\"#{r:02x}{g:02x}{b:02x}\"/>",x*10,y*20).unwrap();
            }
            let symbol = cell.symbol();
            if symbol != " " {
                let (mut r, mut g, mut b) = rgb(cell.fg);
                if cell.modifier.contains(Modifier::DIM) {
                    r = (r as f64 * 0.65) as u8;
                    g = (g as f64 * 0.65) as u8;
                    b = (b as f64 * 0.65) as u8;
                }
                let escaped = symbol
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                let width = unicode_width::UnicodeWidthStr::width(symbol).max(1) as u16;
                let weight = if cell.modifier.contains(Modifier::BOLD) {
                    "bold"
                } else {
                    "normal"
                };
                write!(svg,"<text x=\"{}\" y=\"{}\" fill=\"#{r:02x}{g:02x}{b:02x}\" font-weight=\"{weight}\" textLength=\"{}\" lengthAdjust=\"spacingAndGlyphs\">{escaped}</text>",x*10,y*20+16,width*10).unwrap();
                x += width;
            } else {
                x += 1;
            }
        }
    }
    svg.push_str("</g></svg>\n");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/images/termatrix.svg");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, svg).unwrap();
}
