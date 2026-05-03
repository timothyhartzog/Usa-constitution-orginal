#!/usr/bin/env python3
"""
Constitutional Corpus Analytics Engine

Analyzes the corpus to generate statistics, distributions, and insights
for visualization and reporting.
"""

import json
from pathlib import Path
from collections import Counter, defaultdict
from datetime import datetime
import statistics

PROJECT_ROOT = Path(__file__).parent.parent
DATA_CHUNKS = PROJECT_ROOT / "data" / "chunks"
CORPUS_FILE = DATA_CHUNKS / "constitution_full_corpus.json"
ANALYTICS_OUTPUT = PROJECT_ROOT / "analytics" / "data"

class CorpusAnalytics:
    """Comprehensive corpus analysis engine."""

    def __init__(self):
        self.corpus = self._load_corpus()
        self.chunks = self.corpus['chunks']
        self.analytics = {}

    def _load_corpus(self):
        """Load the corpus file."""
        with open(CORPUS_FILE) as f:
            return json.load(f)

    def analyze_all(self):
        """Run all analytics."""
        print("\nGenerating corpus analytics...")

        self.analytics['corpus_overview'] = self.analyze_corpus_overview()
        self.analytics['clause_analysis'] = self.analyze_clauses()
        self.analytics['issue_analysis'] = self.analyze_issues()
        self.analytics['collection_analysis'] = self.analyze_collections()
        self.analytics['author_analysis'] = self.analyze_authors()
        self.analytics['document_analysis'] = self.analyze_documents()
        self.analytics['clause_issue_matrix'] = self.analyze_clause_issue_matrix()
        self.analytics['author_clause_matrix'] = self.analyze_author_clause_matrix()
        self.analytics['word_analysis'] = self.analyze_word_frequency()
        self.analytics['chunk_size_distribution'] = self.analyze_chunk_sizes()
        self.analytics['document_relationships'] = self.analyze_document_relationships()
        self.analytics['temporal_analysis'] = self.analyze_temporal()

        self._save_analytics()
        self._print_summary()

    def analyze_corpus_overview(self):
        """Overall corpus statistics."""
        total_words = sum(c['word_count'] for c in self.chunks)
        word_counts = [c['word_count'] for c in self.chunks]

        return {
            'total_chunks': len(self.chunks),
            'total_documents': len(set(c['document_id'] for c in self.chunks)),
            'total_words': total_words,
            'total_collections': len(set(c['collection'] for c in self.chunks)),
            'average_chunk_size': statistics.mean(word_counts),
            'median_chunk_size': statistics.median(word_counts),
            'min_chunk_size': min(word_counts),
            'max_chunk_size': max(word_counts),
            'std_dev_chunk_size': statistics.stdev(word_counts) if len(word_counts) > 1 else 0,
            'total_clause_tags': len(set(tag for c in self.chunks for tag in c.get('constitutional_clause_tags', []))),
            'total_issue_tags': len(set(tag for c in self.chunks for tag in c.get('issue_tags', [])))
        }

    def analyze_clauses(self):
        """Constitutional clause analysis."""
        clause_counts = Counter()
        clause_documents = defaultdict(set)
        clause_issues = defaultdict(set)

        for chunk in self.chunks:
            for clause in chunk.get('constitutional_clause_tags', []):
                clause_counts[clause] += 1
                clause_documents[clause].add(chunk['document_id'])
                for issue in chunk.get('issue_tags', []):
                    clause_issues[clause].add(issue)

        return {
            'top_clauses': [
                {
                    'clause': clause,
                    'count': count,
                    'documents': len(clause_documents[clause]),
                    'related_issues': list(clause_issues[clause])
                }
                for clause, count in clause_counts.most_common(30)
            ],
            'clause_distribution': {
                'total_unique': len(clause_counts),
                'median': statistics.median(clause_counts.values()) if clause_counts else 0,
                'average': statistics.mean(clause_counts.values()) if clause_counts else 0
            }
        }

    def analyze_issues(self):
        """Thematic issue analysis."""
        issue_counts = Counter()
        issue_documents = defaultdict(set)
        issue_clauses = defaultdict(set)

        for chunk in self.chunks:
            for issue in chunk.get('issue_tags', []):
                issue_counts[issue] += 1
                issue_documents[issue].add(chunk['document_id'])
                for clause in chunk.get('constitutional_clause_tags', []):
                    issue_clauses[issue].add(clause)

        return {
            'top_issues': [
                {
                    'issue': issue,
                    'count': count,
                    'documents': len(issue_documents[issue]),
                    'related_clauses': list(issue_clauses[issue])
                }
                for issue, count in issue_counts.most_common(20)
            ],
            'issue_distribution': {
                'total_unique': len(issue_counts),
                'median': statistics.median(issue_counts.values()) if issue_counts else 0,
                'average': statistics.mean(issue_counts.values()) if issue_counts else 0
            }
        }

    def analyze_collections(self):
        """Collection-level analysis."""
        collection_data = defaultdict(lambda: {'count': 0, 'words': 0, 'documents': set()})

        for chunk in self.chunks:
            coll = chunk['collection']
            collection_data[coll]['count'] += 1
            collection_data[coll]['words'] += chunk['word_count']
            collection_data[coll]['documents'].add(chunk['document_id'])

        return [
            {
                'collection': coll,
                'chunks': data['count'],
                'total_words': data['words'],
                'documents': len(data['documents']),
                'average_chunk_size': data['words'] / data['count'] if data['count'] > 0 else 0
            }
            for coll, data in sorted(collection_data.items(), key=lambda x: x[1]['count'], reverse=True)
        ]

    def analyze_authors(self):
        """Author analysis."""
        author_data = defaultdict(lambda: {'count': 0, 'words': 0, 'documents': set(), 'clauses': set()})

        for chunk in self.chunks:
            author = chunk['author']
            author_data[author]['count'] += 1
            author_data[author]['words'] += chunk['word_count']
            author_data[author]['documents'].add(chunk['document_id'])
            for clause in chunk.get('constitutional_clause_tags', []):
                author_data[author]['clauses'].add(clause)

        return [
            {
                'author': author,
                'chunks': data['count'],
                'total_words': data['words'],
                'documents': len(data['documents']),
                'unique_clauses': len(data['clauses']),
                'average_chunk_size': data['words'] / data['count'] if data['count'] > 0 else 0
            }
            for author, data in sorted(author_data.items(), key=lambda x: x[1]['count'], reverse=True)
        ]

    def analyze_documents(self):
        """Document-level analysis."""
        doc_data = defaultdict(lambda: {'count': 0, 'words': 0, 'clauses': set(), 'issues': set()})

        for chunk in self.chunks:
            doc = chunk['document_id']
            doc_data[doc]['count'] += 1
            doc_data[doc]['words'] += chunk['word_count']
            for clause in chunk.get('constitutional_clause_tags', []):
                doc_data[doc]['clauses'].add(clause)
            for issue in chunk.get('issue_tags', []):
                doc_data[doc]['issues'].add(issue)

        return [
            {
                'document_id': doc,
                'title': self.chunks[[c['document_id'] for c in self.chunks].index(doc)]['title'],
                'chunks': data['count'],
                'total_words': data['words'],
                'unique_clauses': len(data['clauses']),
                'unique_issues': len(data['issues']),
                'average_chunk_size': data['words'] / data['count'] if data['count'] > 0 else 0
            }
            for doc, data in sorted(doc_data.items(), key=lambda x: x[1]['count'], reverse=True)
        ]

    def analyze_clause_issue_matrix(self):
        """Clause-Issue co-occurrence matrix."""
        matrix = defaultdict(lambda: defaultdict(int))

        for chunk in self.chunks:
            clauses = chunk.get('constitutional_clause_tags', [])
            issues = chunk.get('issue_tags', [])
            for clause in clauses:
                for issue in issues:
                    matrix[clause][issue] += 1

        # Convert to list format
        return {
            'clauses': sorted(set(c for row in matrix.values() for c in row.keys())),
            'issues': sorted(set(issue for issues in matrix.values() for issue in issues.keys())),
            'matrix': [
                {
                    'clause': clause,
                    'issues': dict(matrix[clause])
                }
                for clause in sorted(matrix.keys())
            ]
        }

    def analyze_author_clause_matrix(self):
        """Author-Clause co-occurrence matrix."""
        matrix = defaultdict(lambda: defaultdict(int))

        for chunk in self.chunks:
            author = chunk['author']
            for clause in chunk.get('constitutional_clause_tags', []):
                matrix[author][clause] += 1

        return {
            'authors': sorted(matrix.keys()),
            'clauses': sorted(set(c for row in matrix.values() for c in row.keys())),
            'matrix': [
                {
                    'author': author,
                    'clauses': dict(matrix[author])
                }
                for author in sorted(matrix.keys())
            ]
        }

    def analyze_word_frequency(self):
        """Word frequency analysis (top terms)."""
        word_counts = Counter()

        for chunk in self.chunks:
            words = chunk['text'].lower().split()
            # Simple filtering: skip common stop words
            stop_words = {
                'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for',
                'of', 'with', 'by', 'from', 'be', 'is', 'are', 'was', 'were', 'been',
                'have', 'has', 'had', 'do', 'does', 'did', 'will', 'would', 'should',
                'could', 'may', 'might', 'must', 'can', 'shall', 'that', 'this',
                'which', 'who', 'whom', 'what', 'where', 'when', 'why', 'how'
            }
            for word in words:
                # Clean punctuation
                clean_word = ''.join(c for c in word if c.isalpha()).lower()
                if clean_word and len(clean_word) > 3 and clean_word not in stop_words:
                    word_counts[clean_word] += 1

        return {
            'top_words': [
                {'word': word, 'count': count}
                for word, count in word_counts.most_common(50)
            ]
        }

    def analyze_chunk_sizes(self):
        """Chunk size distribution analysis."""
        categories = {
            'small (0-200)': 0,
            'medium (200-400)': 0,
            'large (400-600)': 0,
            'xlarge (600+)': 0
        }

        for chunk in self.chunks:
            wc = chunk['word_count']
            if wc < 200:
                categories['small (0-200)'] += 1
            elif wc < 400:
                categories['medium (200-400)'] += 1
            elif wc < 600:
                categories['large (400-600)'] += 1
            else:
                categories['xlarge (600+)'] += 1

        return categories

    def analyze_document_relationships(self):
        """Document similarity (by shared clauses/issues)."""
        docs = {}
        for chunk in self.chunks:
            doc_id = chunk['document_id']
            if doc_id not in docs:
                docs[doc_id] = {
                    'clauses': set(),
                    'issues': set(),
                    'title': chunk['title']
                }
            docs[doc_id]['clauses'].update(chunk.get('constitutional_clause_tags', []))
            docs[doc_id]['issues'].update(chunk.get('issue_tags', []))

        # Calculate similarity
        relationships = []
        doc_list = list(docs.items())
        for i, (doc1_id, doc1_data) in enumerate(doc_list):
            for doc2_id, doc2_data in doc_list[i+1:]:
                shared_clauses = len(doc1_data['clauses'] & doc2_data['clauses'])
                shared_issues = len(doc1_data['issues'] & doc2_data['issues'])
                total_shared = shared_clauses + shared_issues

                if total_shared > 0:
                    relationships.append({
                        'document_1': doc1_id,
                        'document_2': doc2_id,
                        'shared_clauses': shared_clauses,
                        'shared_issues': shared_issues,
                        'similarity_score': total_shared
                    })

        return sorted(relationships, key=lambda x: x['similarity_score'], reverse=True)

    def analyze_with_filters(self, collections=None, authors=None, start_date=None, end_date=None):
        """Analyze corpus with filters applied."""
        filtered_chunks = self.chunks

        if collections:
            filtered_chunks = [c for c in filtered_chunks if c.get('collection') in collections]

        if authors:
            filtered_chunks = [c for c in filtered_chunks if c.get('author') in authors]

        if start_date:
            filtered_chunks = [c for c in filtered_chunks if c.get('date', '') >= start_date]

        if end_date:
            filtered_chunks = [c for c in filtered_chunks if c.get('date', '') <= end_date]

        original_chunks = self.chunks
        self.chunks = filtered_chunks

        result = {
            'corpus_overview': self.analyze_corpus_overview(),
            'clause_analysis': self.analyze_clauses(),
            'issue_analysis': self.analyze_issues(),
            'collection_analysis': self.analyze_collections(),
            'author_analysis': self.analyze_authors()
        }

        self.chunks = original_chunks
        return result

    def analyze_temporal(self):
        """Analyze clause mentions over time by document/collection."""
        timeline = defaultdict(lambda: defaultdict(int))

        for chunk in self.chunks:
            doc_id = chunk.get('document_id', 'unknown')
            date = chunk.get('date', 'unknown')

            for clause in chunk.get('constitutional_clause_tags', []):
                timeline[date][clause] += 1

        result = []
        for date in sorted(timeline.keys()):
            for clause, count in timeline[date].items():
                result.append({
                    'date': date,
                    'clause': clause,
                    'mentions': count
                })

        return sorted(result, key=lambda x: x['date'])

    def compare_authors(self, author1, author2, clause=None):
        """Compare two authors' positions and focus areas."""
        author1_chunks = [c for c in self.chunks if c.get('author') == author1]
        author2_chunks = [c for c in self.chunks if c.get('author') == author2]

        def get_author_stats(chunks, name):
            clauses = Counter()
            issues = Counter()
            for chunk in chunks:
                for clause in chunk.get('constitutional_clause_tags', []):
                    clauses[clause] += 1
                for issue in chunk.get('issue_tags', []):
                    issues[issue] += 1

            return {
                'name': name,
                'chunks': len(chunks),
                'words': sum(c.get('word_count', 0) for c in chunks),
                'top_clauses': [{'clause': c, 'count': count} for c, count in clauses.most_common(10)],
                'top_issues': [{'issue': i, 'count': count} for i, count in issues.most_common(8)]
            }

        author1_stats = get_author_stats(author1_chunks, author1)
        author2_stats = get_author_stats(author2_chunks, author2)

        shared_clauses = set(c['clause'] for c in author1_stats['top_clauses']) & \
                         set(c['clause'] for c in author2_stats['top_clauses'])
        shared_issues = set(i['issue'] for i in author1_stats['top_issues']) & \
                        set(i['issue'] for i in author2_stats['top_issues'])

        return {
            'author1': author1_stats,
            'author2': author2_stats,
            'shared_clauses': list(shared_clauses),
            'shared_issues': list(shared_issues),
            'agreement_score': (len(shared_clauses) + len(shared_issues)) /
                               (len(author1_stats['top_clauses']) + len(author2_stats['top_clauses']) +
                                len(author1_stats['top_issues']) + len(author2_stats['top_issues']) + 0.1)
        }

    def generate_report(self):
        """Generate a comprehensive analytics report."""
        overview = self.analyze_corpus_overview()
        clauses = self.analyze_clauses()
        issues = self.analyze_issues()

        report = {
            'title': 'Constitutional Research Corpus Analytics Report',
            'generated_at': datetime.now().isoformat(),
            'executive_summary': {
                'total_chunks': overview['total_chunks'],
                'total_documents': overview['total_documents'],
                'total_words': overview['total_words'],
                'total_clauses_referenced': overview['total_clause_tags'],
                'total_issues_discussed': overview['total_issue_tags'],
                'average_chunk_size': round(overview['average_chunk_size'], 1)
            },
            'key_findings': {
                'most_discussed_clause': clauses['top_clauses'][0] if clauses['top_clauses'] else None,
                'most_discussed_issue': issues['top_issues'][0] if issues['top_issues'] else None,
                'clause_coverage': clauses['clause_distribution'],
                'issue_coverage': issues['issue_distribution']
            },
            'document_statistics': self.analyze_documents(),
            'collection_statistics': self.analyze_collections(),
            'author_statistics': self.analyze_authors()
        }

        return report

    def _save_analytics(self):
        """Save analytics to JSON files."""
        ANALYTICS_OUTPUT.mkdir(parents=True, exist_ok=True)

        # Save comprehensive analytics
        analytics_file = ANALYTICS_OUTPUT / "analytics.json"
        with open(analytics_file, 'w') as f:
            json.dump(self.analytics, f, indent=2, default=str)

        # Save individual analytics files for efficiency
        for key, data in self.analytics.items():
            file_path = ANALYTICS_OUTPUT / f"{key}.json"
            with open(file_path, 'w') as f:
                json.dump(data, f, indent=2, default=str)

        print(f"Analytics saved to {ANALYTICS_OUTPUT}")

    def _print_summary(self):
        """Print analytics summary."""
        overview = self.analytics['corpus_overview']
        print(f"\n{'='*70}")
        print("CORPUS ANALYTICS SUMMARY")
        print(f"{'='*70}")
        print(f"Total Chunks: {overview['total_chunks']:,}")
        print(f"Total Documents: {overview['total_documents']}")
        print(f"Total Words: {overview['total_words']:,}")
        print(f"Average Chunk Size: {overview['average_chunk_size']:.0f} words")
        print(f"Total Unique Clauses: {overview['total_clause_tags']}")
        print(f"Total Unique Issues: {overview['total_issue_tags']}")

        clauses = self.analytics['clause_analysis']['top_clauses']
        if clauses:
            print(f"\nTop Clause: {clauses[0]['clause']} ({clauses[0]['count']} mentions)")

        issues = self.analytics['issue_analysis']['top_issues']
        if issues:
            print(f"Top Issue: {issues[0]['issue']} ({issues[0]['count']} mentions)")

        print(f"{'='*70}\n")

def main():
    """Main entry point."""
    analytics = CorpusAnalytics()
    analytics.analyze_all()

if __name__ == "__main__":
    main()
