//! Build `constitution_archive.bin` from the clean text files in `data/clean/`
//! and the process timeline in `data/process_timeline.json`.
//!
//! Usage:
//!   cargo run --bin build-archive
//!   cargo run --bin build-archive -- --clean-dir data/clean --output data/index/constitution_archive.bin

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use constitution_archive::{Archive, Chunk, ProcessTimeline};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(
    name = "build-archive",
    about = "Build a binary archive from clean text files for the WASM app"
)]
struct Args {
    /// Directory containing clean `.txt` files.
    #[arg(long, default_value = "data/clean")]
    clean_dir: PathBuf,

    /// Path to `data/process_timeline.json`.
    #[arg(long, default_value = "data/process_timeline.json")]
    timeline: PathBuf,

    /// Path to `data/world_constitutions_acquisition_report.json`.
    #[arg(
        long,
        default_value = "data/world_constitutions_acquisition_report.json"
    )]
    world_report: PathBuf,

    /// Output path for the binary archive.
    #[arg(long, default_value = "data/index/constitution_archive.bin")]
    output: PathBuf,

    /// Output path for the world metadata JSON.
    #[arg(long, default_value = "crates/constitution-app/assets/world_meta.json")]
    world_meta_output: PathBuf,

    /// Sliding window size in words.
    #[arg(long, default_value_t = 300)]
    window_size: usize,

    /// Sliding window stride in words.
    #[arg(long, default_value_t = 200)]
    stride: usize,

    /// Web-optimized build: excludes large corpora (founders correspondence,
    /// letters of delegates, Elliot's debates) to keep the archive under ~30MB.
    #[arg(long)]
    web: bool,
}

// ---------------------------------------------------------------------------
// World-constitution report types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct WorldReport {
    documents: Vec<WorldReportDoc>,
}

#[derive(Debug, Deserialize)]
struct WorldReportDoc {
    document_id: String,
    constitute_id: String,
    country_id: String,
    country: String,
    region: String,
    #[serde(default)]
    word_count: u64,
}

/// Matches `WorldConstitutionMeta` in `constitution-app/src/state.rs`.
#[derive(Debug, Serialize)]
struct WorldMetaOut {
    document_id: String,
    constitute_id: String,
    country_id: String,
    country: String,
    region: String,
    status: String,
    word_count: u64,
}

// ---------------------------------------------------------------------------
// Collection classification
// ---------------------------------------------------------------------------

fn classify_collection(stem: &str) -> &'static str {
    if stem.starts_with("world_constitution_") {
        return "comparative_constitutions_world";
    }
    if stem.starts_with("eu_constitution") {
        return "comparative_constitutions_eu";
    }
    if stem == "us_constitution_1787" {
        return "constitution";
    }
    if stem == "federalist_papers_complete" {
        return "federalist_papers";
    }
    if stem.contains("brutus")
        || stem.contains("cato")
        || stem.contains("centinel")
        || stem.contains("federal_farmer")
    {
        return "anti_federalist";
    }
    if stem.contains("madisons_notes") || stem.contains("madison_debates") {
        return "madisons_notes";
    }
    if stem.contains("elliots_debates") {
        return "elliots_debates";
    }
    if stem.contains("bill_of_rights") {
        return "bill_of_rights";
    }
    if stem.contains("founders_correspondence")
        || stem.starts_with("hamilton_")
        || stem.starts_with("jefferson_")
        || stem.contains("madison_correspondence")
        || stem.starts_with("washington_")
        || stem.starts_with("adams_")
        || stem.starts_with("jay_")
        || stem.starts_with("franklin_")
        || stem.starts_with("mason_")
        || stem.starts_with("henry_")
    {
        return "founders_correspondence";
    }
    if stem.contains("letters_delegates") {
        return "letters_delegates_congress";
    }
    if stem.contains("farrand_records") {
        return "madisons_notes";
    }
    "other"
}

/// Derive a human-readable title from the file stem.
fn title_from_stem(stem: &str) -> String {
    stem.replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    format!("{}{}", upper, chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Try to extract a year from the file stem.  Looks for a 4-digit number
/// that could be a year (1700–2099).
fn year_from_stem(stem: &str) -> String {
    for part in stem.split('_') {
        if part.len() == 4 {
            if let Ok(n) = part.parse::<u32>() {
                if (1700..2100).contains(&n) {
                    return n.to_string();
                }
            }
        }
    }
    String::new()
}

/// Derive a rough author from file stem and collection.
fn author_from_stem(stem: &str, collection: &str) -> String {
    if collection == "comparative_constitutions_world"
        || collection == "comparative_constitutions_eu"
    {
        return String::new();
    }
    if stem.starts_with("hamilton_") {
        return "Alexander Hamilton".to_string();
    }
    if stem.starts_with("jefferson_") {
        return "Thomas Jefferson".to_string();
    }
    if stem.starts_with("madison_") || stem.contains("madisons_notes") {
        return "James Madison".to_string();
    }
    if stem.starts_with("washington_") {
        return "George Washington".to_string();
    }
    if stem.starts_with("adams_") {
        return "John Adams".to_string();
    }
    if stem.starts_with("jay_") {
        return "John Jay".to_string();
    }
    if stem.starts_with("franklin_") {
        return "Benjamin Franklin".to_string();
    }
    if stem.starts_with("mason_") {
        return "George Mason".to_string();
    }
    if stem.starts_with("henry_") {
        return "Patrick Henry".to_string();
    }
    if stem.contains("brutus") {
        return "Brutus".to_string();
    }
    if stem.contains("cato") {
        return "Cato".to_string();
    }
    if stem.contains("centinel") {
        return "Centinel".to_string();
    }
    if stem.contains("federal_farmer") {
        return "Federal Farmer".to_string();
    }
    if stem == "federalist_papers_complete" {
        return "Hamilton, Madison, Jay".to_string();
    }
    String::new()
}

/// Derive a coarse document type.
fn doc_type_from_collection(collection: &str) -> &'static str {
    match collection {
        "constitution" | "bill_of_rights" => "foundational_document",
        "federalist_papers" | "anti_federalist" => "essay",
        "madisons_notes" | "elliots_debates" => "debate_record",
        "founders_correspondence" => "letter",
        "letters_delegates_congress" => "letter",
        "comparative_constitutions_world" | "comparative_constitutions_eu" => "constitution",
        _ => "document",
    }
}

// ---------------------------------------------------------------------------
// Sliding-window chunking
// ---------------------------------------------------------------------------

fn chunk_text(
    text: &str,
    document_id: &str,
    title: &str,
    author: &str,
    date: &str,
    collection: &str,
    document_type: &str,
    window_size: usize,
    stride: usize,
) -> Vec<Chunk> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0usize;
    let mut chunk_num = 0u32;

    loop {
        let end = (start + window_size).min(words.len());
        let chunk_words = &words[start..end];
        let chunk_text = chunk_words.join(" ");
        let wc = chunk_words.len() as u32;

        let preview: String = chunk_text.chars().take(200).collect();
        let preview = if chunk_text.chars().count() > 200 {
            format!("{preview}\u{2026}")
        } else {
            preview
        };

        let chunk_id = format!("{}_{:04}", document_id, chunk_num);

        chunks.push(Chunk {
            chunk_id,
            document_id: document_id.to_string(),
            title: title.to_string(),
            author: author.to_string(),
            date: date.to_string(),
            source_collection: collection.to_string(),
            source_url: String::new(),
            document_type: document_type.to_string(),
            issue_tags: Vec::new(),
            constitutional_clause_tags: Vec::new(),
            text: chunk_text,
            word_count: wc,
            preview,
        });

        chunk_num += 1;

        if end >= words.len() {
            break;
        }
        start += stride;
        // Ensure we always emit the tail
        if start >= words.len() {
            break;
        }
    }

    chunks
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Collect all .txt files from data/clean/
    let mut txt_files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(&args.clean_dir)
        .with_context(|| format!("reading directory {:?}", args.clean_dir))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("txt") {
            txt_files.push(path);
        }
    }
    txt_files.sort();
    println!(
        "found {} .txt files in {:?}",
        txt_files.len(),
        args.clean_dir
    );

    // 2. Load world report metadata for enrichment (country name, etc.)
    let world_meta_map: HashMap<String, &WorldReportDoc>;
    let world_report_bytes;
    let world_report: WorldReport;

    let have_world_report = args.world_report.exists();
    if have_world_report {
        world_report_bytes = fs::read(&args.world_report)
            .with_context(|| format!("reading {:?}", args.world_report))?;
        world_report = serde_json::from_slice(&world_report_bytes)
            .with_context(|| format!("parsing {:?}", args.world_report))?;
        world_meta_map = world_report
            .documents
            .iter()
            .map(|d| (d.document_id.clone(), d))
            .collect();
    } else {
        eprintln!(
            "warning: {:?} not found, world constitution metadata will be minimal",
            args.world_report
        );
        world_report = WorldReport {
            documents: Vec::new(),
        };
        world_meta_map = HashMap::new();
    }

    // 3. Chunk all files
    let mut all_chunks: Vec<Chunk> = Vec::new();
    let mut file_count = 0u32;

    for path in &txt_files {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let text = fs::read_to_string(path).with_context(|| format!("reading {:?}", path))?;

        let collection = classify_collection(stem);

        if args.web {
            let skip = matches!(
                collection,
                "founders_correspondence" | "letters_delegates_congress" | "elliots_debates"
            );
            if skip {
                continue;
            }
        }

        let document_id = stem.to_string();
        let document_type = doc_type_from_collection(collection);

        // Derive metadata
        let title = if let Some(meta) = world_meta_map.get(stem) {
            format!("Constitution of {}", meta.country)
        } else {
            title_from_stem(stem)
        };

        let author = author_from_stem(stem, collection);
        let date = year_from_stem(stem);

        let chunks = chunk_text(
            &text,
            &document_id,
            &title,
            &author,
            &date,
            collection,
            document_type,
            args.window_size,
            args.stride,
        );

        all_chunks.extend(chunks);
        file_count += 1;
    }

    println!(
        "created {} chunks from {} files (window={}, stride={})",
        all_chunks.len(),
        file_count,
        args.window_size,
        args.stride
    );

    // 4. Load process timeline
    let timeline = if args.timeline.exists() {
        let bytes =
            fs::read(&args.timeline).with_context(|| format!("reading {:?}", args.timeline))?;
        ProcessTimeline::from_json(&bytes)?
    } else {
        eprintln!(
            "warning: {:?} not found, using empty timeline",
            args.timeline
        );
        ProcessTimeline::default()
    };
    println!("loaded {} timeline events", timeline.len());

    // 5. Build archive
    let archive = Archive::build(all_chunks, timeline);
    let stats = archive.stats();
    let bytes = archive.to_bytes()?;

    // 6. Write binary archive
    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {:?}", parent))?;
    }
    fs::write(&args.output, &bytes).with_context(|| format!("writing {:?}", args.output))?;

    // 7. Generate world_meta.json
    let world_meta_out: Vec<WorldMetaOut> = world_report
        .documents
        .iter()
        .map(|d| WorldMetaOut {
            document_id: d.document_id.clone(),
            constitute_id: d.constitute_id.clone(),
            country_id: d.country_id.clone(),
            country: d.country.clone(),
            region: d.region.clone(),
            status: "downloaded".to_string(),
            word_count: d.word_count,
        })
        .collect();

    if let Some(parent) = args.world_meta_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {:?}", parent))?;
    }
    let meta_json = serde_json::to_string_pretty(&world_meta_out)?;
    fs::write(&args.world_meta_output, &meta_json)
        .with_context(|| format!("writing {:?}", args.world_meta_output))?;

    // 8. Print stats
    println!();
    println!("=== Archive Stats ===");
    println!("  chunks:      {}", stats.chunks);
    println!("  documents:   {}", stats.documents);
    println!("  terms:       {}", stats.terms);
    println!("  events:      {}", stats.events);
    println!("  collections: {}", stats.collections);
    println!("  authors:     {}", stats.authors);
    println!("  citations:   {}", stats.citations);
    println!(
        "  binary size: {:.2} MB",
        bytes.len() as f64 / (1024.0 * 1024.0)
    );
    println!();
    println!("wrote {:?}", args.output);
    println!(
        "wrote {:?} ({} entries)",
        args.world_meta_output,
        world_meta_out.len()
    );

    Ok(())
}
