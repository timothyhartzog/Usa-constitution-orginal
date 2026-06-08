#!/usr/bin/env python3
"""
Prepare deployment: Generate analytics and verify all files are ready.

Run this before deploying to Cloudflare Pages or Render.
"""

import json
from pathlib import Path
from scripts.analytics_engine import CorpusAnalytics

PROJECT_ROOT = Path(__file__).parent.parent
ANALYTICS_OUTPUT = PROJECT_ROOT / "analytics" / "data"
FRONTEND_DIR = PROJECT_ROOT / "frontend"
ANALYTICS_DIR = PROJECT_ROOT / "analytics"

def generate_analytics():
    """Generate all analytics data."""
    print("📊 Generating analytics...")
    try:
        analytics = CorpusAnalytics()
        analytics.analyze_all()
        print("✅ Analytics generated successfully")
        return True
    except Exception as e:
        print(f"❌ Failed to generate analytics: {e}")
        return False

def verify_files():
    """Verify all required files exist."""
    print("\n📋 Verifying deployment files...")

    required_files = [
        # Frontend
        "frontend/index.html",
        "frontend/search.js",
        "frontend/ui.js",
        "frontend/styles.css",
        # Analytics
        "analytics/dashboard.html",
        "analytics/dashboard.css",
        "analytics/dashboard.js",
        "analytics/analytics-loader.js",
        "analytics/chart-renderer.js",
        # Corpus
        "data/chunks/constitution_full_corpus.json",
        "data/index/search_index.json",
        # Config
        "config/sources_manifest.json",
        "config/constitutional_clauses.json",
    ]

    analytics_files = [
        "corpus_overview",
        "clause_analysis",
        "issue_analysis",
        "collection_analysis",
        "author_analysis",
        "document_analysis",
        "clause_issue_matrix",
        "author_clause_matrix",
        "word_analysis",
        "chunk_size_distribution",
        "document_relationships",
        "temporal_analysis",
    ]

    missing = []

    for file in required_files:
        path = PROJECT_ROOT / file
        if path.exists():
            print(f"  ✅ {file}")
        else:
            print(f"  ❌ {file}")
            missing.append(file)

    print("\n📈 Analytics files:")
    for key in analytics_files:
        path = ANALYTICS_OUTPUT / f"{key}.json"
        if path.exists():
            size = path.stat().st_size / 1024
            print(f"  ✅ {key}.json ({size:.1f} KB)")
        else:
            print(f"  ❌ {key}.json")
            missing.append(f"analytics/data/{key}.json")

    if missing:
        print(f"\n⚠️  Missing {len(missing)} file(s)")
        return False
    else:
        print("\n✅ All files verified")
        return True

def print_deployment_info():
    """Print deployment information."""
    print("\n📡 Deployment Information:")
    print("\nCloudflare Pages:")
    print("  1. Go to https://dash.cloudflare.com")
    print("  2. Click Pages → Create Project")
    print("  3. Connect GitHub repository")
    print("  4. Select branch: claude/constitutional-research-system-LopyL")
    print("  5. Build settings:")
    print("     - Framework: None")
    print("     - Build command: (leave empty)")
    print("     - Output directory: /")
    print("  6. Deploy")
    print("\nRender:")
    print("  1. Go to https://render.com")
    print("  2. Click Web Service")
    print("  3. Connect GitHub repository")
    print("  4. Settings:")
    print("     - Name: constitutional-analytics-api")
    print("     - Environment: Python 3")
    print("     - Build: pip install -r requirements.txt && python scripts/analytics_engine.py")
    print("     - Start: gunicorn analytics.api:app --bind 0.0.0.0:$PORT")
    print("  5. Plan: Free")
    print("  6. Deploy")

def print_summary(analytics_ok, files_ok):
    """Print summary."""
    print("\n" + "="*60)
    print("DEPLOYMENT PREPARATION SUMMARY")
    print("="*60)
    print(f"Analytics generated: {'✅ Yes' if analytics_ok else '❌ No'}")
    print(f"Files verified: {'✅ Yes' if files_ok else '❌ No'}")
    print("="*60)

    if analytics_ok and files_ok:
        print("\n🚀 Ready for deployment!")
        print("\nNext steps:")
        print("  1. git add analytics/data/")
        print("  2. git commit -m 'Generate analytics for deployment'")
        print("  3. git push origin claude/constitutional-research-system-LopyL")
        print("  4. Deploy to Cloudflare Pages and Render (see above)")
        return True
    else:
        print("\n⚠️  Fix missing files before deploying")
        return False

def main():
    """Main entry point."""
    print("🔧 Constitutional Research System - Deployment Preparation\n")

    # Generate analytics
    analytics_ok = generate_analytics()

    # Verify files
    files_ok = verify_files()

    # Print deployment info
    print_deployment_info()

    # Print summary
    ready = print_summary(analytics_ok, files_ok)

    return 0 if ready else 1

if __name__ == "__main__":
    import sys
    sys.exit(main())
