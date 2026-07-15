import os
import argparse
from pathlib import Path

try:
    from huggingface_hub import hf_hub_download
except ImportError:
    print("Please install huggingface_hub: pip install huggingface_hub")
    exit(1)

PROJECT_ROOT = Path(__file__).parent.parent
MODELS_DIR = PROJECT_ROOT / "data" / "models"

def download_model(repo_id, filename):
    MODELS_DIR.mkdir(parents=True, exist_ok=True)
    print(f"Downloading {filename} from {repo_id}...")
    print(f"This may take a few minutes as the file is typically 2GB - 4GB.")
    
    path = hf_hub_download(
        repo_id=repo_id,
        filename=filename,
        local_dir=MODELS_DIR,
        local_dir_use_symlinks=False
    )
    print(f"✅ Successfully downloaded to {path}")
    return path

def main():
    parser = argparse.ArgumentParser(description="Download Quantized LLM for Local RAG")
    parser.add_argument("--repo", type=str, default="TheBloke/Phi-3-mini-4k-instruct-GGUF", help="HuggingFace Repo ID")
    parser.add_argument("--filename", type=str, default="phi-3-mini-4k-instruct.Q4_K_M.gguf", help="Specific GGUF file to download")
    args = parser.parse_args()
    
    download_model(args.repo, args.filename)

if __name__ == '__main__':
    main()
