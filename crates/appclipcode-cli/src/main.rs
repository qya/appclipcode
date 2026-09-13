use appclipcode::{
    generate, generate_png, generate_png_with_template, generate_with_template, read_svg,
    templates, CodeType, Options,
};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

fn print_usage() {
    eprintln!(
        r#"Usage:
  appclipcodegen generate <URL> [-o FILE] [--index N] [--fg HEX --bg HEX] [--type cam|nfc] [--size N]
  appclipcodegen gen <URL> [-o FILE] [--index N] [--fg HEX --bg HEX] [--type cam|nfc] [--size N]
  appclipcodegen scan <FILE.svg> [--info] Decode URL or raw payload from an App Clip Code SVG
  appclipcodegen templates                List available color templates

Options:
  <URL>           URL to encode (must be https://)
  -o, --output    Output file path (.svg or .png). Default: stdout (SVG)
  --index N       Template color index (0-17, default: 0)
  --fg HEX        Foreground color as 6- or 8-digit hex (e.g. 000000 or 00000080)
  --bg HEX        Background color as 6- or 8-digit hex (e.g. FFFFFF or FFFFFF80)
  --type TYPE     Code type: cam (default) or nfc
  --size N        Output image size in pixels for PNG (default: 800)
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        process::exit(1);
    }

    match args[0].as_str() {
        "generate" | "gen" => cmd_generate(&args[1..]),
        "scan" | "read" => cmd_scan(&args[1..]),
        "templates" => cmd_templates(),
        "-h" | "--help" | "help" => print_usage(),
        url if url.starts_with("https://") || url.starts_with("http://") => {
            cmd_generate(&args)
        }
        other => {
            eprintln!("unknown command: {}\n", other);
            print_usage();
            process::exit(1);
        }
    }
}

fn cmd_templates() {
    for t in templates() {
        println!(
            "Index: {:2}  Foreground: {}  Background: {}  Third: {}",
            t.index,
            t.foreground.hex(),
            t.background.hex(),
            t.third.hex()
        );
    }
}

fn cmd_scan(args: &[String]) {
    let mut file_path: Option<String> = None;
    let mut info = false;

    for arg in args {
        match arg.as_str() {
            "--info" | "-v" | "--verbose" => info = true,
            s if !s.starts_with('-') && file_path.is_none() => file_path = Some(s.to_string()),
            _ => {}
        }
    }

    let file_path = match file_path {
        Some(p) => p,
        None => {
            eprintln!("error: scan requires a file path");
            eprintln!("usage: appclipcodegen scan <FILE.svg> [--info]");
            process::exit(1);
        }
    };

    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading file {}: {}", file_path, e);
            process::exit(1);
        }
    };

    if info {
        match appclipcode::read_svg_barcode(&content) {
            Ok(barcode) => {
                println!("Version:  {}", barcode.version);
                println!("Inverted: {}", barcode.inverted);
                println!(
                    "Payload:  {} ({} bytes)",
                    appclipcode::hex_encode(&barcode.payload),
                    barcode.payload.len()
                );
                if let Some(ref url) = barcode.url {
                    println!("URL:      {}", url);
                }
            }
            Err(e) => {
                eprintln!("error scanning SVG: {}", e);
                process::exit(1);
            }
        }
    } else {
        match read_svg(&content) {
            Ok(res) => println!("{}", res),
            Err(e) => {
                eprintln!("error scanning SVG: {}", e);
                process::exit(1);
            }
        }
    }
}

fn cmd_generate(args: &[String]) {
    let mut url: Option<String> = None;
    let mut output: Option<String> = None;
    let mut index: Option<usize> = None;
    let mut fg: Option<String> = None;
    let mut bg: Option<String> = None;
    let mut code_type_str: Option<String> = None;
    let mut size: Option<u32> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--index" | "-index" => {
                if i + 1 < args.len() {
                    index = args[i + 1].parse::<usize>().ok();
                    i += 1;
                }
            }
            "--fg" | "-fg" => {
                if i + 1 < args.len() {
                    fg = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--bg" | "-bg" => {
                if i + 1 < args.len() {
                    bg = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--type" | "-type" => {
                if i + 1 < args.len() {
                    code_type_str = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--size" | "-size" => {
                if i + 1 < args.len() {
                    size = args[i + 1].parse::<u32>().ok();
                    i += 1;
                }
            }
            "-url" | "--url" => {
                if i + 1 < args.len() {
                    url = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            arg if !arg.starts_with('-') && url.is_none() => {
                url = Some(arg.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    let url = match url {
        Some(u) => u,
        None => {
            eprintln!("error: URL is required (must be https://)\n");
            print_usage();
            process::exit(1);
        }
    };

    let code_type = CodeType::from_str_opt(code_type_str.as_deref());
    let opts = Some(Options { code_type });

    let is_png = output
        .as_ref()
        .map(|p| p.ends_with(".png") || p.ends_with(".PNG"))
        .unwrap_or(false);

    if is_png {
        let png_bytes = match (&fg, &bg) {
            (Some(f), Some(b)) => generate_png(&url, f, b, opts, size),
            (None, None) => generate_png_with_template(&url, index.unwrap_or(0), opts, size),
            _ => {
                eprintln!("error: specify either both --fg and --bg, or --index");
                process::exit(1);
            }
        };

        let bytes = match png_bytes {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error: {}", e);
                process::exit(1);
            }
        };

        let out_path = output.as_ref().unwrap();
        if let Err(e) = fs::write(out_path, &bytes) {
            eprintln!("error writing file {}: {}", out_path, e);
            process::exit(1);
        }
        eprintln!("App Clip Code PNG successfully generated: {}", out_path);
    } else {
        let svg_res = match (&fg, &bg) {
            (Some(f), Some(b)) => generate(&url, f, b, opts),
            (None, None) => generate_with_template(&url, index.unwrap_or(0), opts),
            _ => {
                eprintln!("error: specify either both --fg and --bg, or --index");
                process::exit(1);
            }
        };

        let svg = match svg_res {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {}", e);
                process::exit(1);
            }
        };

        if let Some(ref out_path) = output {
            if let Err(e) = fs::write(out_path, &svg) {
                eprintln!("error writing file {}: {}", out_path, e);
                process::exit(1);
            }
            eprintln!("App Clip Code SVG successfully generated: {}", out_path);
        } else {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            let _ = handle.write_all(svg.as_bytes());
        }
    }
}
