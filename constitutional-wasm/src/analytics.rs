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

    pub fn analyze_temporal_network(&self) -> serde_json::Value {
        // Build temporal network of clause mentions over time
        let mut temporal_data: HashMap<String, Vec<String>> = HashMap::new();

        for chunk in &self.chunks {
            let date_key = chunk.date.chars().take(4).collect::<String>(); // Year only
            temporal_data.entry(date_key)
                .or_insert_with(Vec::new)
                .extend(chunk.constitutional_clause_tags.iter().cloned());
        }

        let mut years: Vec<String> = temporal_data.keys().cloned().collect();
        years.sort();

        // Calculate year-to-year clause evolution
        let empty_vec = Vec::new();
        let mut evolution = Vec::new();
        for year in years.iter() {
            let clauses = temporal_data.get(year).unwrap_or(&empty_vec);
            let mut clause_counts: HashMap<String, usize> = HashMap::new();
            for clause in clauses {
                *clause_counts.entry(clause.clone()).or_insert(0) += 1;
            }

            evolution.push(json!({
                "year": year,
                "clauses": clause_counts,
            }));
        }

        json!({
            "timeline": years,
            "evolution": evolution,
        })
    }

    pub fn analyze_clause_debate_network(&self) -> serde_json::Value {
        // Build network of clauses discussed together (co-occurrence)
        let mut co_occurrence: HashMap<(String, String), usize> = HashMap::new();

        for chunk in &self.chunks {
            let clauses = &chunk.constitutional_clause_tags;
            for i in 0..clauses.len() {
                for j in (i + 1)..clauses.len() {
                    let mut pair = [clauses[i].clone(), clauses[j].clone()];
                    pair.sort();
                    *co_occurrence.entry((pair[0].clone(), pair[1].clone())).or_insert(0) += 1;
                }
            }
        }

        // Build node and link data for visualization
        let mut nodes: HashMap<String, usize> = HashMap::new();
        for chunk in &self.chunks {
            for clause in &chunk.constitutional_clause_tags {
                *nodes.entry(clause.clone()).or_insert(0) += 1;
            }
        }

        let node_list: Vec<serde_json::Value> = nodes.into_iter()
            .map(|(name, count)| json!({
                "id": name,
                "value": count,
                "category": "clause",
            }))
            .collect();

        let link_list: Vec<serde_json::Value> = co_occurrence.into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|((source, target), count)| json!({
                "source": source,
                "target": target,
                "value": count,
            }))
            .collect();

        json!({
            "nodes": node_list,
            "links": link_list,
        })
    }

    pub fn analyze_author_influence(&self) -> Vec<serde_json::Value> {
        // Rank authors by influence: clauses mentioned × document coverage × word count
        let mut author_influence: HashMap<String, (usize, usize, usize, HashSet<String>)> = HashMap::new();

        for chunk in &self.chunks {
            let entry = author_influence.entry(chunk.author.clone())
                .or_insert((0, 0, 0, HashSet::new()));

            entry.0 += chunk.constitutional_clause_tags.len(); // Clause mentions
            entry.1 += 1; // Document count
            entry.2 += chunk.word_count; // Total words
            entry.3.insert(chunk.document_id.clone()); // Unique documents
        }

        let mut results: Vec<serde_json::Value> = author_influence.into_iter()
            .map(|(author, (clause_mentions, doc_count, word_count, unique_docs))| {
                // Influence score: clauses × document coverage
                let influence_score = (clause_mentions as f32) * (unique_docs.len() as f32).log10() + 1.0;

                json!({
                    "author": author,
                    "influence_score": influence_score,
                    "clause_mentions": clause_mentions,
                    "documents": unique_docs.len(),
                    "total_words": word_count,
                    "avg_words_per_doc": if doc_count == 0 { 0 } else { word_count / doc_count },
                })
            })
            .collect();

        results.sort_by(|a, b| {
            let score_a = a.get("influence_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let score_b = b.get("influence_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        results.into_iter().take(20).collect()
    }

    pub fn analyze_semantic_similarity_clusters(&self) -> serde_json::Value {
        // Cluster chunks by clause/issue similarity
        let mut clusters: HashMap<String, Vec<String>> = HashMap::new();

        for chunk in &self.chunks {
            // Use primary clause as cluster key
            if let Some(primary_clause) = chunk.constitutional_clause_tags.first() {
                clusters.entry(primary_clause.clone())
                    .or_insert_with(Vec::new)
                    .push(chunk.chunk_id.clone());
            }
        }

        let mut result = Vec::new();
        for (clause, chunk_ids) in clusters {
            result.push(json!({
                "clause": clause,
                "size": chunk_ids.len(),
                "chunks": chunk_ids,
            }));
        }

        json!({
            "clusters": result,
            "total_clusters": result.len(),
        })
    }

    pub fn analyze_ratification_tracking(&self) -> serde_json::Value {
        // Track which states/ratifiers are mentioned by clause
        let mut ratification_data: HashMap<String, Vec<String>> = HashMap::new();

        // Extract state references from text (simple keyword matching)
        let states = vec!["Virginia", "Pennsylvania", "New York", "Massachusetts", "Maryland",
                         "Connecticut", "New Jersey", "Delaware", "Georgia", "South Carolina",
                         "North Carolina", "New Hampshire", "Rhode Island"];

        for chunk in &self.chunks {
            let text_lower = chunk.text.to_lowercase();
            let mut mentioned_states = Vec::new();

            for state in &states {
                if text_lower.contains(&state.to_lowercase()) {
                    mentioned_states.push(state.to_string());
                }
            }

            for clause in &chunk.constitutional_clause_tags {
                ratification_data.entry(clause.clone())
                    .or_insert_with(Vec::new)
                    .extend(mentioned_states.clone());
            }
        }

        // Deduplicate and count
        let mut result = Vec::new();
        for (clause, states) in ratification_data {
            let mut state_counts: HashMap<String, usize> = HashMap::new();
            for state in states {
                *state_counts.entry(state).or_insert(0) += 1;
            }

            result.push(json!({
                "clause": clause,
                "states": state_counts,
                "states_mentioned": state_counts.len(),
            }));
        }

        json!({
            "ratification": result,
        })
    }
}
