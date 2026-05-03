#!/usr/bin/env python3
"""
Constitutional Analytics API

REST API for serving analytics data to frontend dashboard.
"""

from flask import Flask, jsonify, request
import json
from pathlib import Path
from datetime import datetime

app = Flask(__name__)
PROJECT_ROOT = Path(__file__).parent.parent
ANALYTICS_DATA = PROJECT_ROOT / "analytics" / "data"

def load_analytics(key):
    """Load analytics JSON file."""
    file_path = ANALYTICS_DATA / f"{key}.json"
    if file_path.exists():
        with open(file_path) as f:
            return json.load(f)
    return None

@app.route('/api/health', methods=['GET'])
def health():
    """Health check endpoint."""
    return jsonify({
        'status': 'healthy',
        'timestamp': datetime.now().isoformat()
    })

@app.route('/api/analytics/overview', methods=['GET'])
def get_overview():
    """Get corpus overview statistics."""
    data = load_analytics('corpus_overview')
    return jsonify(data or {})

@app.route('/api/analytics/clauses', methods=['GET'])
def get_clauses():
    """Get constitutional clause analysis."""
    data = load_analytics('clause_analysis')
    return jsonify(data or {})

@app.route('/api/analytics/issues', methods=['GET'])
def get_issues():
    """Get thematic issue analysis."""
    data = load_analytics('issue_analysis')
    return jsonify(data or {})

@app.route('/api/analytics/collections', methods=['GET'])
def get_collections():
    """Get collection-level analysis."""
    data = load_analytics('collection_analysis')
    return jsonify(data or [])

@app.route('/api/analytics/authors', methods=['GET'])
def get_authors():
    """Get author analysis."""
    data = load_analytics('author_analysis')
    return jsonify(data or [])

@app.route('/api/analytics/documents', methods=['GET'])
def get_documents():
    """Get document-level analysis."""
    data = load_analytics('document_analysis')
    return jsonify(data or [])

@app.route('/api/analytics/clause-issue-matrix', methods=['GET'])
def get_clause_issue_matrix():
    """Get clause-issue co-occurrence matrix."""
    data = load_analytics('clause_issue_matrix')
    return jsonify(data or {})

@app.route('/api/analytics/author-clause-matrix', methods=['GET'])
def get_author_clause_matrix():
    """Get author-clause co-occurrence matrix."""
    data = load_analytics('author_clause_matrix')
    return jsonify(data or {})

@app.route('/api/analytics/word-frequency', methods=['GET'])
def get_word_frequency():
    """Get word frequency analysis."""
    data = load_analytics('word_analysis')
    return jsonify(data or {})

@app.route('/api/analytics/chunk-sizes', methods=['GET'])
def get_chunk_sizes():
    """Get chunk size distribution."""
    data = load_analytics('chunk_size_distribution')
    return jsonify(data or {})

@app.route('/api/analytics/document-relationships', methods=['GET'])
def get_document_relationships():
    """Get document relationship analysis."""
    data = load_analytics('document_relationships')
    return jsonify(data or [])

@app.route('/api/analytics/all', methods=['GET'])
def get_all():
    """Get all analytics data."""
    all_data = {}

    analytics_files = [
        'corpus_overview', 'clause_analysis', 'issue_analysis',
        'collection_analysis', 'author_analysis', 'document_analysis',
        'clause_issue_matrix', 'author_clause_matrix', 'word_analysis',
        'chunk_size_distribution', 'document_relationships'
    ]

    for key in analytics_files:
        data = load_analytics(key)
        if data:
            all_data[key] = data

    return jsonify(all_data)

@app.errorhandler(404)
def not_found(error):
    """Handle 404 errors."""
    return jsonify({'error': 'Endpoint not found'}), 404

@app.errorhandler(500)
def server_error(error):
    """Handle 500 errors."""
    return jsonify({'error': 'Internal server error'}), 500

if __name__ == '__main__':
    app.run(debug=True, port=5000)
