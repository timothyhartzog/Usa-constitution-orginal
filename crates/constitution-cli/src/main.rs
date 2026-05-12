//! Constitution Archive CLI.
//!
//! Subcommands:
//! - `build` — read the Python-pipeline JSON corpus + the process timeline
//!   JSON, build the inverted index, write the binary archive.
//! - `search` — load an archive and run a BM25 query.
//! - `process` — list / look up timeline events.
//! - `stats` — print archive statistics.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use constitution_archive::{Archive, Chunk, Filter, FilterValue, ProcessTimeline, SearchOptions};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(name = "constitution-archive", version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Build the binary archive from JSON corpus + timeline.
    Build {
        /// Path to `data/chunks/constitution_full_corpus.json`.
        #[arg(long, default_value = "data/chunks/constitution_full_corpus.json")]
        corpus: PathBuf,
        /// Path to `data/process_timeline.json`.
        #[arg(long, default_value = "data/process_timeline.json")]
        timeline: PathBuf,
        /// Output path for the binary archive.
        #[arg(long, default_value = "data/index/constitution_archive.bin")]
        output: PathBuf,
    },
    /// Run a BM25 search against an existing archive.
    Search {
        /// Path to the binary archive.
        #[arg(long, default_value = "data/index/constitution_archive.bin")]
        archive: PathBuf,
        /// Limit on returned hits.
        #[arg(long, default_value_t = 10)]
        limit: usize,
        /// Only include this collection (repeatable).
        #[arg(long)]
        collection: Vec<String>,
        /// Author substring (repeatable).
        #[arg(long)]
        author: Vec<String>,
        /// Only include chunks whose date starts with this prefix.
        #[arg(long)]
        date_prefix: Option<String>,
        /// Fuzzy expansion radius (Levenshtein distance, 0 disables).
        #[arg(long)]
        fuzzy: Option<u32>,
        /// The query.
        query: String,
    },
    /// Type-ahead suggestions: indexed terms sharing a prefix.
    Suggest {
        /// Path to the binary archive.
        #[arg(long, default_value = "data/index/constitution_archive.bin")]
        archive: PathBuf,
        /// Maximum suggestions returned.
        #[arg(long, default_value_t = 10)]
        limit: usize,
        /// Prefix to match.
        prefix: String,
    },
    /// Show indexed terms within a Levenshtein distance of a query term.
    Fuzzy {
        /// Path to the binary archive.
        #[arg(long, default_value = "data/index/constitution_archive.bin")]
        archive: PathBuf,
        /// Maximum edit distance.
        #[arg(long, default_value_t = 2)]
        max_distance: u32,
        /// Maximum results returned.
        #[arg(long, default_value_t = 10)]
        limit: usize,
        /// Term to look up.
        term: String,
    },
    /// Inspect the citation graph (clause / essay / person references).
    Citations {
        /// Path to the binary archive.
        #[arg(long, default_value = "data/index/constitution_archive.bin")]
        archive: PathBuf,
        #[command(subcommand)]
        cmd: CitationsCmd,
    },
    /// List or look up timeline events.
    Process {
        /// Path to the binary archive.
        #[arg(long, default_value = "data/index/constitution_archive.bin")]
        archive: PathBuf,
        #[command(subcommand)]
        cmd: ProcessCmd,
    },
    /// Print archive statistics.
    Stats {
        /// Path to the binary archive.
        #[arg(long, default_value = "data/index/constitution_archive.bin")]
        archive: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum ProcessCmd {
    /// List all events.
    List,
    /// Look up a single event by id.
    Get { id: String },
    /// Free-text search across event text.
    Find { query: String },
    /// Show all events in a phase ("convention", "ratification", …).
    Phase { name: String },
}

#[derive(Subcommand, Debug)]
enum CitationsCmd {
    /// List the most-cited targets (clauses, essays, persons).
    Top {
        /// Maximum entries returned.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// List the outgoing citations of a single chunk.
    From {
        /// Chunk id.
        chunk_id: String,
    },
    /// List every chunk that cites `target_key`.
    /// `target_key` is canonical: clause:I.8, essay:federalist:10, person:madison.
    To {
        /// Maximum hits returned.
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// Canonical target key.
        target_key: String,
    },
    /// Dump the top-N co-occurrence citation graph as JSON.
    Graph {
        /// Number of top targets to include.
        #[arg(long, default_value_t = 25)]
        top: usize,
    },
}

/// Schema of `data/chunks/constitution_full_corpus.json`.
#[derive(Debug, Deserialize)]
struct CorpusFile {
    chunks: Vec<Chunk>,
}

fn load_corpus(path: &PathBuf) -> Result<Vec<Chunk>> {
    let bytes = fs::read(path).with_context(|| format!("reading corpus {:?}", path))?;
    let file: CorpusFile =
        serde_json::from_slice(&bytes).with_context(|| format!("parsing corpus {:?}", path))?;
    Ok(file.chunks)
}

fn load_timeline(path: &PathBuf) -> Result<ProcessTimeline> {
    if !path.exists() {
        eprintln!("warning: {:?} not found, building empty timeline", path);
        return Ok(ProcessTimeline::default());
    }
    let bytes = fs::read(path).with_context(|| format!("reading timeline {:?}", path))?;
    Ok(ProcessTimeline::from_json(&bytes)?)
}

fn cmd_build(corpus: PathBuf, timeline: PathBuf, output: PathBuf) -> Result<()> {
    let chunks = load_corpus(&corpus)?;
    let timeline = load_timeline(&timeline)?;
    let n_chunks = chunks.len();
    let n_events = timeline.len();

    let archive = Archive::build(chunks, timeline);
    let bytes = archive.to_bytes()?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {:?}", parent))?;
    }
    fs::write(&output, &bytes).with_context(|| format!("writing {:?}", output))?;

    println!(
        "wrote {:?}: {} chunks, {} events, {} bytes",
        output,
        n_chunks,
        n_events,
        bytes.len()
    );
    Ok(())
}

fn open_archive(path: &PathBuf) -> Result<Archive> {
    let bytes = fs::read(path).with_context(|| format!("reading archive {:?}", path))?;
    Archive::load(&bytes).map_err(|e| anyhow!("loading archive: {e}"))
}

fn cmd_search(
    archive_path: PathBuf,
    limit: usize,
    collections: Vec<String>,
    authors: Vec<String>,
    date_prefix: Option<String>,
    fuzzy: Option<u32>,
    query: String,
) -> Result<()> {
    let archive = open_archive(&archive_path)?;
    let mut filter = Filter::default();
    if !collections.is_empty() {
        filter = filter.with(FilterValue::Collection(collections));
    }
    if !authors.is_empty() {
        filter = filter.with(FilterValue::Author(authors));
    }
    if let Some(p) = date_prefix {
        filter = filter.with(FilterValue::DatePrefix(p));
    }
    let opts = SearchOptions {
        limit,
        min_score: 0.0,
        fuzzy_distance: fuzzy.unwrap_or(0),
        snippet_window: 240,
    };
    let hits = archive.search(&query, &filter, &opts);
    if hits.is_empty() {
        println!("(no results)");
        return Ok(());
    }
    for h in hits {
        let chunk = archive.chunk(&h.chunk_id)?;
        let preview = if h.snippet.text.is_empty() {
            chunk.ensured_preview()
        } else {
            let mut s = String::new();
            if h.snippet.leading_ellipsis {
                s.push('…');
            }
            s.push_str(&h.snippet.text);
            if h.snippet.trailing_ellipsis {
                s.push('…');
            }
            s
        };
        println!(
            "{:>6.2}  {}  [{}] {}\n        {}\n",
            h.score, chunk.date, chunk.source_collection, chunk.title, preview
        );
    }
    Ok(())
}

fn cmd_suggest(archive_path: PathBuf, limit: usize, prefix: String) -> Result<()> {
    let archive = open_archive(&archive_path)?;
    for term in archive.suggest(&prefix, limit) {
        println!("{term}");
    }
    Ok(())
}

fn cmd_fuzzy(archive_path: PathBuf, max_distance: u32, limit: usize, term: String) -> Result<()> {
    let archive = open_archive(&archive_path)?;
    for (t, d) in archive.fuzzy_terms(&term, max_distance, limit) {
        println!("{d}  {t}");
    }
    Ok(())
}

fn cmd_process(archive_path: PathBuf, cmd: ProcessCmd) -> Result<()> {
    let archive = open_archive(&archive_path)?;
    let timeline = archive.timeline();
    match cmd {
        ProcessCmd::List => {
            for e in &timeline.events {
                println!("{}  [{:?}]  {}  ({})", e.date, e.phase, e.title, e.id);
            }
        }
        ProcessCmd::Get { id } => {
            let e = timeline.get(&id).map_err(|e| anyhow!("{e}"))?;
            println!("{}", serde_json::to_string_pretty(e)?);
        }
        ProcessCmd::Find { query } => {
            for e in timeline.search(&query) {
                println!("{}  {}  ({})", e.date, e.title, e.id);
            }
        }
        ProcessCmd::Phase { name } => {
            let p = serde_json::from_value(serde_json::Value::String(name.clone()))
                .map_err(|e| anyhow!("invalid phase {name}: {e}"))?;
            for e in timeline.by_phase(p) {
                println!("{}  {}  ({})", e.date, e.title, e.id);
            }
        }
    }
    Ok(())
}

fn cmd_stats(archive_path: PathBuf) -> Result<()> {
    let archive = open_archive(&archive_path)?;
    let s = archive.stats();
    println!("{}", serde_json::to_string_pretty(&s)?);
    Ok(())
}

fn cmd_citations(archive_path: PathBuf, cmd: CitationsCmd) -> Result<()> {
    let archive = open_archive(&archive_path)?;
    match cmd {
        CitationsCmd::Top { limit } => {
            for (key, count) in archive.top_citation_targets(limit) {
                println!("{count:>5}  {key}");
            }
        }
        CitationsCmd::From { chunk_id } => {
            let citations = archive
                .citations_from(&chunk_id)
                .map_err(|e| anyhow!("{e}"))?;
            if citations.is_empty() {
                println!("(no outgoing citations)");
                return Ok(());
            }
            for c in citations {
                println!(
                    "{:>6}  {:<28}  {}",
                    c.byte_offset,
                    c.target.key(),
                    c.matched_text
                );
            }
        }
        CitationsCmd::To { limit, target_key } => {
            let mut hits = archive.cited_by(&target_key);
            if hits.is_empty() {
                println!("(no chunks cite {target_key})");
                return Ok(());
            }
            hits.sort_by(|a, b| a.0.date.cmp(&b.0.date));
            for (chunk, c) in hits.into_iter().take(limit) {
                println!(
                    "{:<46}  {}  [{}] {}",
                    chunk.chunk_id, chunk.date, chunk.source_collection, c.matched_text
                );
            }
        }
        CitationsCmd::Graph { top } => {
            let view = archive.citation_graph_view(top);
            println!("{}", serde_json::to_string_pretty(&view)?);
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Build {
            corpus,
            timeline,
            output,
        } => cmd_build(corpus, timeline, output),
        Cmd::Search {
            archive,
            limit,
            collection,
            author,
            date_prefix,
            fuzzy,
            query,
        } => cmd_search(
            archive,
            limit,
            collection,
            author,
            date_prefix,
            fuzzy,
            query,
        ),
        Cmd::Suggest {
            archive,
            limit,
            prefix,
        } => cmd_suggest(archive, limit, prefix),
        Cmd::Fuzzy {
            archive,
            max_distance,
            limit,
            term,
        } => cmd_fuzzy(archive, max_distance, limit, term),
        Cmd::Process { archive, cmd } => cmd_process(archive, cmd),
        Cmd::Stats { archive } => cmd_stats(archive),
        Cmd::Citations { archive, cmd } => cmd_citations(archive, cmd),
    }
}
