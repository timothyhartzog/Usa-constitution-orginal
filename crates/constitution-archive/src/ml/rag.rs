use super::MlError;
use crate::ChunkId;
use candle_core::Device;
use std::path::{Path, PathBuf};

pub struct RagConfig {
    pub max_new_tokens: usize,
    pub temperature: f64,
    pub top_p: f64,
    pub seed: u64,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            max_new_tokens: 512,
            temperature: 0.1,
            top_p: 0.9,
            seed: 299792458,
        }
    }
}

/// Local LLM Generation Engine for Citation-Grounded Q&A
pub struct RagEngine {
    model_path: Option<PathBuf>,
    tokenizer_path: Option<PathBuf>,
}

impl RagEngine {
    pub fn load(
        model_path: &Path,
        tokenizer_path: &Path,
        _device: &Device,
    ) -> Result<Self, MlError> {
        let mp = if model_path.exists() {
            Some(model_path.to_path_buf())
        } else {
            None
        };
        let tp = if tokenizer_path.exists() {
            Some(tokenizer_path.to_path_buf())
        } else {
            None
        };
        Ok(Self {
            model_path: mp,
            tokenizer_path: tp,
        })
    }

    /// Generate an answer based on grounded context. Returns a stream/iterator of tokens in a real app,
    /// but for this API stub we just return a String.
    pub fn generate_answer(
        &self,
        query: &str,
        contexts: &[(ChunkId, String)],
        config: &RagConfig,
    ) -> Result<String, MlError> {
        let mut prompt = format!(
            "System: Answer the question strictly using the provided context.\n\nContext:\n"
        );
        for (id, ctx) in contexts {
            prompt.push_str(&format!("[{}]\n{}\n\n", id, ctx));
        }
        prompt.push_str(&format!("Question: {}\nAnswer: ", query));

        // Fallback stub response
        Ok(format!("[RAG Stub Response] I have analyzed {} provided chunks to answer: {}. Generated with temp={} and max_tokens={}.", 
            contexts.len(), query, config.temperature, config.max_new_tokens))
    }

    pub fn generate_stream(
        &self,
        query: &str,
        contexts: &[(ChunkId, String)],
        config: RagConfig,
        tx: tokio::sync::mpsc::Sender<Result<String, String>>,
    ) {
        let mut prompt = format!("<|system|>\nYou are an expert constitutional historian. Answer the question strictly using the provided context. Never use outside knowledge. If the context does not contain the answer, say 'I cannot answer this based on the provided text.'\n\nContext:\n");
        for (id, ctx) in contexts {
            prompt.push_str(&format!("--- [{}] ---\n{}\n\n", id, ctx));
        }
        prompt.push_str(&format!(
            "<|end|>\n<|user|>\n{}<|end|>\n<|assistant|>\n",
            query
        ));

        let query_clone = query.to_string();
        let contexts_len = contexts.len();

        tokio::spawn(async move {
            let client = reqwest::Client::new();

            // Construct the payload for Llama.cpp /completion endpoint
            let payload = serde_json::json!({
                "prompt": prompt,
                "n_predict": config.max_new_tokens,
                "temperature": config.temperature,
                "top_p": config.top_p,
                "stream": true,
                "stop": ["<|end|>"]
            });

            // Try connecting to the local Llama.cpp server
            match client
                .post("http://127.0.0.1:8081/completion")
                .json(&payload)
                .send()
                .await
            {
                Ok(mut response) => {
                    use futures_util::StreamExt;

                    let mut is_first = true;
                    while let Some(chunk_res) = response.chunk().await.ok().flatten() {
                        if let Ok(text) = String::from_utf8(chunk_res.to_vec()) {
                            // Parse SSE data
                            for line in text.lines() {
                                if line.starts_with("data: ") {
                                    let data_str = &line[6..];
                                    if data_str.trim() == "[DONE]" {
                                        break;
                                    }
                                    if let Ok(parsed) =
                                        serde_json::from_str::<serde_json::Value>(data_str)
                                    {
                                        if let Some(content) =
                                            parsed.get("content").and_then(|c| c.as_str())
                                        {
                                            if is_first {
                                                let _ = tx
                                                    .send(Ok(format!(
                                                        "**[Live AI Response]**\n\n{}",
                                                        content
                                                    )))
                                                    .await;
                                                is_first = false;
                                            } else {
                                                let _ = tx.send(Ok(content.to_string())).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    // Fallback stub if Llama.cpp is not running
                    let stub = format!("**[Offline Mode - Llama.cpp server not found on port 8081]**\n\nI have analyzed {} provided chunks to answer: {}. Generated with temp={} and max_tokens={}.", 
                        contexts_len, query_clone, config.temperature, config.max_new_tokens);
                    for word in stub.split_whitespace() {
                        let _ = tx.send(Ok(format!("{} ", word))).await;
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                }
            }
        });
    }
}
