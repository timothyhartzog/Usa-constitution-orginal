use std::collections::{HashMap, HashSet};
use serde_json::json;
use serde::{Serialize, Deserialize};
use crate::Chunk;
use crate::utils::tokenize;

#[derive(Debug)]
pub struct Analytics {
    chunks: Vec<Chunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClauseAnalysis {
    pub clause: String,
    pub mentions: usize,
    pub documents: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueAnalysis {
    pub issue: String,
    pub mentions: usize,
    pub documents: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorAnalysis {
    pub author: String,
    pub chunk_count: usize,
    pub total_words: usize,
    pub unique_clauses: usize,
    pub unique_issues: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionAnalysis {
    pub collection: String,
    pub chunk_count: usize,
    pub total_words: usize,
    pub documents: usize,
}

impl Analytics {
    pub fn new() -> Self {
        Analytics {
            chunks: Vec::new(),
        }
    }

    pub fn load_corpus(&mut self, json_str: &str) -> Result<String, String> {
        let corpus: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse corpus JSON: {}", e))?;

        let chunks_array = corpus.get("chunks")
            .and_then(|c| c.as_array())
            .ok_or("Corpus must have 'chunks' array")?;

        self.chunks.clear();

        for chunk_json in chunks_array {
            let chunk: Chunk = serde_json::from_value(chunk_json.clone())
                .map_err(|e| format!("Failed to parse chunk: {}", e))?;
            self.chunks.push(chunk);
        }

        Ok(format!("Loaded {} chunks for analytics", self.chunks.len()))
    }

    pub fn analyze_overview(&self) -> serde_json::Value {
        let total_words: usize = self.chunks.iter().map(|c| c.word_count).sum();
        let sizes: Vec<usize> = self.chunks.iter().map(|c| c.word_count).collect();

        let authors: HashSet<String> = self.chunks.iter().map(|c| c.author.clone()).collect();
        let documents: HashSet<String> = self.chunks.iter().map(|c| c.document_id.clone()).collect();
        let collections: HashSet<String> = self.chunks.iter()
            .map(|c| c.chunk_id.split('_').next().unwrap_or("unknown").to_string())
            .collect();

        let clauses: HashSet<String> = self.chunks.iter()
            .flat_map(|c| c.constitutional_clause_tags.iter())
            .cloned()
            .collect();

        let issues: HashSet<String> = self.chunks.iter()
            .flat_map(|c| c.issue_tags.iter())
            .cloned()
            .collect();

        let avg = if self.chunks.is_empty() { 0.0 } else { total_words as f32 / self.chunks.len() as f32 };

        json!({
            "total_chunks": self.chunks.len(),
            "total_documents": documents.len(),
            "total_words": total_words,
            "average_chunk_size": avg,
            "min_chunk_size": sizes.iter().cloned().min().unwrap_or(0),
            "max_chunk_size": sizes.iter().cloned().max().unwrap_or(0),
            "unique_authors": authors.len(),
            "unique_collections": collections.len(),
            "total_clause_tags": clauses.len(),
            "total_issue_tags": issues.len(),
        })
    }

    pub fn analyze_clauses(&self, top_n: usize) -> Vec<ClauseAnalysis> {
        let mut clause_counts: HashMap<String, (usize, HashSet<String>)> = HashMap::new();

        for chunk in &self.chunks {
            for clause in &chunk.constitutional_clause_tags {
                let entry = clause_counts.entry(clause.clone()).or_insert((0, HashSet::new()));
                entry.0 += 1;
                entry.1.insert(chunk.document_id.clone());
            }
        }

        let mut results: Vec<ClauseAnalysis> = clause_counts
            .into_iter()
            .map(|(clause, (mentions, docs))| ClauseAnalysis {
                clause,
                mentions,
                documents: docs.len(),
            })
            .collect();

        results.sort_by(|a, b| b.mentions.cmp(&a.mentions));
        results.truncate(top_n);
        results
    }

    pub fn analyze_issues(&self, top_n: usize) -> Vec<IssueAnalysis> {
        let mut issue_counts: HashMap<String, (usize, HashSet<String>)> = HashMap::new();

        for chunk in &self.chunks {
            for issue in &chunk.issue_tags {
                let entry = issue_counts.entry(issue.clone()).or_insert((0, HashSet::new()));
                entry.0 += 1;
                entry.1.insert(chunk.document_id.clone());
            }
        }

        let mut results: Vec<IssueAnalysis> = issue_counts
            .into_iter()
            .map(|(issue, (mentions, docs))| IssueAnalysis {
                issue,
                mentions,
                documents: docs.len(),
            })
            .collect();

        results.sort_by(|a, b| b.mentions.cmp(&a.mentions));
        results.truncate(top_n);
        results
    }

    pub fn analyze_authors(&self) -> Vec<AuthorAnalysis> {
        let mut author_stats: HashMap<String, (usize, usize, HashSet<String>, HashSet<String>)> = HashMap::new();

        for chunk in &self.chunks {
            let entry = author_stats.entry(chunk.author.clone())
                .or_insert((0, 0, HashSet::new(), HashSet::new()));
            entry.0 += 1;
            entry.1 += chunk.word_count;
            entry.2.extend(chunk.constitutional_clause_tags.iter().cloned());
            entry.3.extend(chunk.issue_tags.iter().cloned());
        }

        let mut results: Vec<AuthorAnalysis> = author_stats
            .into_iter()
            .map(|(author, (chunks, words, clauses, issues))| AuthorAnalysis {
                author,
                chunk_count: chunks,
                total_words: words,
                unique_clauses: clauses.len(),
                unique_issues: issues.len(),
            })
            .collect();

        results.sort_by(|a, b| b.chunk_count.cmp(&a.chunk_count));
        results
    }

    pub fn analyze_collections(&self) -> Vec<CollectionAnalysis> {
        let mut collection_stats: HashMap<String, (usize, usize, HashSet<String>)> = HashMap::new();

        for chunk in &self.chunks {
            let collection = chunk.chunk_id.split('_').next().unwrap_or("unknown").to_string();
            let entry = collection_stats.entry(collection)
                .or_insert((0, 0, HashSet::new()));
            entry.0 += 1;
            entry.1 += chunk.word_count;
            entry.2.insert(chunk.document_id.clone());
        }

        let mut results: Vec<CollectionAnalysis> = collection_stats
            .into_iter()
            .map(|(collection, (chunks, words, docs))| CollectionAnalysis {
                collection,
                chunk_count: chunks,
                total_words: words,
                documents: docs.len(),
            })
            .collect();

        results.sort_by(|a, b| b.chunk_count.cmp(&a.chunk_count));
        results
    }

    pub fn analyze_word_frequency(&self, top_n: usize) -> Vec<(String, usize)> {
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for chunk in &self.chunks {
            let tokens = tokenize(&chunk.text);
            for token in tokens {
                if token.len() >= 3 {
                    *word_counts.entry(token.to_lowercase()).or_insert(0) += 1;
                }
            }
        }

        let mut results: Vec<(String, usize)> = word_counts.into_iter().collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(top_n);
        results
    }

    pub fn analyze_clause_issue_matrix(&self) -> serde_json::Value {
        let mut matrix: HashMap<String, HashMap<String, usize>> = HashMap::new();

        for chunk in &self.chunks {
            for clause in &chunk.constitutional_clause_tags {
                let clause_row = matrix.entry(clause.clone()).or_insert_with(HashMap::new);
                for issue in &chunk.issue_tags {
                    *clause_row.entry(issue.clone()).or_insert(0) += 1;
                }
            }
        }

        json!(matrix)
    }

    pub fn analyze_author_clause_matrix(&self) -> serde_json::Value {
        let mut matrix: HashMap<String, HashMap<String, usize>> = HashMap::new();

        for chunk in &self.chunks {
            let author_row = matrix.entry(chunk.author.clone()).or_insert_with(HashMap::new);
            for clause in &chunk.constitutional_clause_tags {
                *author_row.entry(clause.clone()).or_insert(0) += 1;
            }
        }

        json!(matrix)
    }

    pub fn filter_chunks(&self, filters: &HashMap<String, Vec<String>>) -> Vec<Chunk> {
        self.chunks.iter()
            .filter(|chunk| {
                if let Some(collections) = filters.get("collections") {
                    let collection = chunk.chunk_id.split('_').next().unwrap_or("unknown");
                    if !collections.contains(&collection.to_string()) {
                        return false;
                    }
                }

                if let Some(authors) = filters.get("authors") {
                    if !authors.contains(&chunk.author) {
                        return false;
                    }
                }

                if let Some(clauses) = filters.get("clauses") {
                    if !chunk.constitutional_clause_tags.iter().any(|c| clauses.contains(c)) {
                        return false;
                    }
                }

                if let Some(issues) = filters.get("issues") {
                    if !chunk.issue_tags.iter().any(|i| issues.contains(i)) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    pub fn compare_authors(&self, author1: &str, author2: &str) -> serde_json::Value {
        let chunks1: Vec<&Chunk> = self.chunks.iter().filter(|c| c.author == author1).collect();
        let chunks2: Vec<&Chunk> = self.chunks.iter().filter(|c| c.author == author2).collect();

        let clauses1: HashSet<String> = chunks1.iter()
            .flat_map(|c| c.constitutional_clause_tags.iter())
            .cloned()
            .collect();
        let clauses2: HashSet<String> = chunks2.iter()
            .flat_map(|c| c.constitutional_clause_tags.iter())
            .cloned()
            .collect();

        let shared_clauses = clauses1.intersection(&clauses2).cloned().collect::<Vec<_>>();
        let unique_to_author1 = clauses1.difference(&clauses2).cloned().collect::<Vec<_>>();
        let unique_to_author2 = clauses2.difference(&clauses1).cloned().collect::<Vec<_>>();

        let total_clauses = clauses1.union(&clauses2).count();
        let agreement_score = if total_clauses == 0 { 0.0 } else { shared_clauses.len() as f32 / total_clauses as f32 };

        json!({
            "author1": {
                "name": author1,
                "chunk_count": chunks1.len(),
                "total_words": chunks1.iter().map(|c| c.word_count).sum::<usize>(),
                "unique_clauses": clauses1.len(),
            },
            "author2": {
                "name": author2,
                "chunk_count": chunks2.len(),
                "total_words": chunks2.iter().map(|c| c.word_count).sum::<usize>(),
                "unique_clauses": clauses2.len(),
            },
            "shared_clauses": shared_clauses,
            "unique_to_author1": unique_to_author1,
            "unique_to_author2": unique_to_author2,
            "agreement_score": agreement_score,
        })
    }
}
