use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use dpi::PhysicalSize;
use servo::{
    ConsoleLogLevel, EventLoopWaker, JSValue, LoadStatus, Preferences, RenderingContext, Servo, ServoBuilder,
    SoftwareRenderingContext, WebView, WebViewBuilder, WebViewDelegate,
};
use url::Url;

const EXTRACT_JS: &str = include_str!("extract.js");

struct Args {
    target: String,
    width: u32,
    height: u32,
    png: Option<String>,
    json: bool,
    cols: usize,
    wait_ms: u64,
}

fn parse_args() -> Args {
    let mut args = Args { target: String::new(), width: 1280, height: 800, png: None, json: false, cols: 100, wait_ms: 0 };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--width" => args.width = it.next().and_then(|v| v.parse().ok()).expect("--width N"),
            "--height" => args.height = it.next().and_then(|v| v.parse().ok()).expect("--height N"),
            "--png" => args.png = it.next(),
            "--json" => args.json = true,
            "--wait" => args.wait_ms = it.next().and_then(|v| v.parse().ok()).expect("--wait MS"),
            "--cols" => args.cols = it.next().and_then(|v| v.parse().ok()).expect("--cols N"),
            "--help" | "-h" => {
                args.target.clear();
                break;
            },
            _ if a.starts_with("--") => {
                eprintln!("unknown option {a}");
                std::process::exit(2);
            },
            _ => args.target = a,
        }
    }
    if args.target.is_empty() {
        eprintln!("usage: senga <file.html|url> [--width 1280] [--height 800] [--png out.png] [--cols 100] [--wait 500] [--json]");
        std::process::exit(2);
    }
    args
}

fn to_url(target: &str) -> Url {
    if let Ok(url) = Url::parse(target) {
        return url;
    }
    let path = std::fs::canonicalize(target).expect("file not found");
    Url::from_file_path(path).expect("bad path")
}

#[derive(Clone)]
struct Waker(Arc<AtomicBool>);
impl EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }
    fn wake(&self) {}
}

#[derive(Default)]
struct Delegate {
    loaded: Cell<bool>,
    console: RefCell<Vec<(ConsoleLogLevel, String)>>,
}
impl WebViewDelegate for Delegate {
    fn show_console_message(&self, _: WebView, level: ConsoleLogLevel, message: String) {
        self.console.borrow_mut().push((level, message));
    }
    fn notify_new_frame_ready(&self, webview: WebView) {
        webview.paint();
    }
    fn notify_load_status_changed(&self, _: WebView, status: LoadStatus) {
        if status == LoadStatus::Complete {
            self.loaded.set(true);
        }
    }
}

fn spin(servo: &Servo, until: impl Fn() -> bool) {
    while !until() {
        servo.spin_event_loop();
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn main() {
    let args = parse_args();
    rustls::crypto::aws_lc_rs::default_provider().install_default().expect("crypto provider");

    let rendering_context = Rc::new(
        SoftwareRenderingContext::new(PhysicalSize { width: args.width, height: args.height })
            .expect("software rendering context"),
    );
    rendering_context.make_current().expect("make current");

    let mut preferences = Preferences::default();
    preferences.network_http_proxy_uri = String::new();
    preferences.network_https_proxy_uri = String::new();

    let servo = ServoBuilder::default()
        .preferences(preferences)
        .event_loop_waker(Box::new(Waker(Arc::new(AtomicBool::new(false)))))
        .build();

    let delegate = Rc::new(Delegate::default());
    let webview = WebViewBuilder::new(&servo, rendering_context.clone())
        .url(to_url(&args.target))
        .delegate(delegate.clone())
        .build();
    webview.show();

    spin(&servo, || delegate.loaded.get());
    // Some pages keep arranging themselves after `load` (hydration, fetched lists).
    // There is no signal for "done", so the caller may ask for a pause.
    if args.wait_ms > 0 {
        let until = std::time::Instant::now() + Duration::from_millis(args.wait_ms);
        spin(&servo, || std::time::Instant::now() >= until);
    }

    // A screenshot request waits for fonts, images and pending frames. We use it
    // as the "layout is settled" signal even when nobody asked for the picture.
    let shot: Rc<RefCell<Option<Result<servo::RgbaImage, _>>>> = Rc::new(RefCell::new(None));
    let shot_out = shot.clone();
    webview.take_screenshot(None, move |result| *shot_out.borrow_mut() = Some(result));
    spin(&servo, || shot.borrow().is_some());
    if let Some(path) = &args.png {
        match shot.borrow_mut().take().unwrap() {
            Ok(image) => image.save(path).expect("write png"),
            Err(e) => eprintln!("screenshot failed: {e:?}"),
        }
    }

    let out: Rc<RefCell<Option<Result<JSValue, _>>>> = Rc::new(RefCell::new(None));
    let out_cb = out.clone();
    webview.evaluate_javascript(EXTRACT_JS, move |r| *out_cb.borrow_mut() = Some(r));
    spin(&servo, || out.borrow().is_some());

    let json = match out.borrow_mut().take().unwrap() {
        Ok(JSValue::String(s)) => s,
        other => {
            eprintln!("extract.js did not return a string: {other:?}");
            std::process::exit(1);
        },
    };

    if args.json {
        println!("{json}");
    } else {
        let page: senga::Page = serde_json::from_str(&json).expect("parse layout json");
        let opts = senga::Options { cols: args.cols, ..Default::default() };
        print!("{}", senga::render_with(&page, &opts));
        // What the page said while it was coming up. Errors here usually explain
        // an empty wireframe better than the wireframe can.
        let console = delegate.console.borrow();
        if !console.is_empty() {
            println!("\n## Console  ({} messages)", console.len());
            for (level, message) in console.iter().take(30) {
                let line: String = message.lines().next().unwrap_or("").chars().take(300).collect();
                println!("- {level:?}: {line}");
            }
        }
    }
    drop(webview);
    drop(servo);
}
