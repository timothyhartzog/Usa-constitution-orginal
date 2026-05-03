#!/usr/bin/env python3
"""
Advanced Analytics Engine - Temporal, Network, and Influence Analysis
Generates supplementary analytics for deep research insights.
"""

import json
from pathlib import Path
from collections import defaultdict
from datetime import datetime

PROJECT_ROOT = Path(__file__).parent.parent
ANALYTICS_OUTPUT = PROJECT_ROOT / "analytics" / "data"
CORPUS_PATH = PROJECT_ROOT / "data" / "chunks" / "constitution_full_corpus.json"


class AdvancedAnalytics:
    def __init__(self, corpus_path=CORPUS_PATH):
        self.corpus = None
        self.chunks = []
        self.load_corpus(corpus_path)

    def load_corpus(self, path):
        """Load corpus data."""
        print(f"📖 Loading corpus from {path}...")
        with open(path) as f:
            self.corpus = json.load(f)
            self.chunks = self.corpus.get("chunks", [])
        print(f"✅ Loaded {len(self.chunks)} chunks")

    def analyze_temporal_network(self):
        """Analyze how constitutional ideas evolved over time."""
        print("\n⏳ Analyzing temporal network...")

        temporal_data = defaultdict(list)

        for chunk in self.chunks:
            year = chunk.get("date", "0000")[:4]
            for clause in chunk.get("constitutional_clause_tags", []):
                temporal_data[year].append(clause)

        # Sort by year
        years = sorted(temporal_data.keys())

        evolution = []
        for year in years:
            clause_counts = defaultdict(int)
            for clause in temporal_data[year]:
                clause_counts[clause] += 1

            evolution.append({
                "year": year,
                "clauses": dict(clause_counts),
                "total_mentions": sum(clause_counts.values()),
            })

        result = {
            "timeline": years,
            "evolution": evolution,
            "total_years": len(years),
        }

        return result

    def analyze_clause_debate_network(self):
        """Analyze which clauses were debated together."""
        print("\n🔗 Analyzing clause debate network...")

        co_occurrence = defaultdict(int)
        clause_mentions = defaultdict(int)

        for chunk in self.chunks:
            clauses = chunk.get("constitutional_clause_tags", [])

            # Count clause mentions
            for clause in clauses:
                clause_mentions[clause] += 1

            # Count co-occurrences
            for i, c1 in enumerate(clauses):
                for c2 in clauses[i+1:]:
                    pair = tuple(sorted([c1, c2]))
                    co_occurrence[pair] += 1

        # Build nodes
        nodes = [
            {
                "id": clause,
                "value": count,
                "category": "clause",
            }
            for clause, count in clause_mentions.items()
        ]

        # Build links
        links = [
            {
                "source": pair[0],
                "target": pair[1],
                "value": count,
            }
            for pair, count in co_occurrence.items()
            if count > 0
        ]

        # Sort links by strength
        links.sort(key=lambda x: -x["value"])

        result = {
            "nodes": nodes,
            "links": links[:50],  # Top 50 relationships
            "total_nodes": len(nodes),
            "total_links": len(links),
        }

        return result

    def analyze_author_influence(self):
        """Rank authors by their influence on final document."""
        print("\n👑 Analyzing author influence...")

        author_stats = defaultdict(lambda: {
            "clause_mentions": 0,
            "documents": set(),
            "total_words": 0,
            "issue_mentions": 0,
        })

        for chunk in self.chunks:
            author = chunk.get("author", "Unknown")
            stats = author_stats[author]

            stats["clause_mentions"] += len(chunk.get("constitutional_clause_tags", []))
            stats["documents"].add(chunk.get("document_id", ""))
            stats["total_words"] += chunk.get("word_count", 0)
            stats["issue_mentions"] += len(chunk.get("issue_tags", []))

        # Calculate influence score
        results = []
        for author, stats in author_stats.items():
            num_docs = len(stats["documents"])

            # Influence = clauses mentioned × log(documents) × normalized words
            influence_score = (
                stats["clause_mentions"] *
                (1 + len(str(num_docs))) *
                (stats["total_words"] / 100 + 1)
            )

            results.append({
                "author": author,
                "influence_score": round(influence_score, 2),
                "clause_mentions": stats["clause_mentions"],
                "documents_contributed": num_docs,
                "total_words": stats["total_words"],
                "issue_mentions": stats["issue_mentions"],
                "avg_words_per_doc": round(stats["total_words"] / max(1, num_docs), 1),
            })

        # Sort by influence score
        results.sort(key=lambda x: -x["influence_score"])

        return results[:20]  # Top 20 influencers

    def analyze_semantic_similarity_clusters(self):
        """Cluster chunks by constitutional clause similarity."""
        print("\n🔀 Analyzing semantic similarity clusters...")

        clusters = defaultdict(list)

        for chunk in self.chunks:
            # Use primary clause as cluster key
            primary_clause = chunk.get("constitutional_clause_tags", [None])[0]
            if primary_clause:
                clusters[primary_clause].append({
                    "chunk_id": chunk.get("chunk_id"),
                    "title": chunk.get("title"),
                    "author": chunk.get("author"),
                })

        result = []
        for clause, chunks_list in clusters.items():
            result.append({
                "clause": clause,
                "size": len(chunks_list),
                "chunks": chunks_list,
            })

        # Sort by cluster size
        result.sort(key=lambda x: -x["size"])

        return {
            "clusters": result,
            "total_clusters": len(result),
            "largest_cluster": result[0]["size"] if result else 0,
        }

    def analyze_ratification_tracking(self):
        """Track state involvement in constitutional ratification."""
        print("\n🗳️  Analyzing ratification tracking...")

        # US states that were part of 1787 convention
        states = {
            "Virginia": "VA",
            "Pennsylvania": "PA",
            "New York": "NY",
            "Massachusetts": "MA",
            "Maryland": "MD",
            "Connecticut": "CT",
            "New Jersey": "NJ",
            "Delaware": "DE",
            "Georgia": "GA",
            "South Carolina": "SC",
            "North Carolina": "NC",
            "New Hampshire": "NH",
            "Rhode Island": "RI",
        }

        ratification_data = defaultdict(lambda: defaultdict(int))

        for chunk in self.chunks:
            text_lower = chunk.get("text", "").lower()

            mentioned_states = []
            for state, code in states.items():
                if state.lower() in text_lower or code in text_lower:
                    mentioned_states.append(state)

            for clause in chunk.get("constitutional_clause_tags", []):
                for state in mentioned_states:
                    ratification_data[clause][state] += 1

        # Format output
        result = []
        for clause, states_dict in ratification_data.items():
            result.append({
                "clause": clause,
                "states_mentioned": dict(states_dict),
                "states_count": len(states_dict),
                "top_states": sorted(states_dict.items(), key=lambda x: -x[1])[:5],
            })

        return {
            "ratification": result,
            "total_states": len(states),
        }

    def generate_all(self):
        """Generate all advanced analytics."""
        print("\n" + "="*60)
        print("ADVANCED ANALYTICS GENERATION")
        print("="*60)

        analytics = {
            "temporal_network": self.analyze_temporal_network(),
            "clause_debate_network": self.analyze_clause_debate_network(),
            "author_influence": self.analyze_author_influence(),
            "semantic_similarity": self.analyze_semantic_similarity_clusters(),
            "ratification_tracking": self.analyze_ratification_tracking(),
        }

        return analytics

    def save_analytics(self, analytics):
        """Save analytics to JSON files."""
        print("\n💾 Saving analytics...")
        ANALYTICS_OUTPUT.mkdir(parents=True, exist_ok=True)

        for key, data in analytics.items():
            path = ANALYTICS_OUTPUT / f"advanced_{key}.json"
            with open(path, "w") as f:
                json.dump(data, f, indent=2)
            size = path.stat().st_size / 1024
            print(f"  ✅ {path.name} ({size:.1f} KB)")

    def print_summary(self, analytics):
        """Print summary of analytics."""
        print("\n" + "="*60)
        print("ADVANCED ANALYTICS SUMMARY")
        print("="*60)

        print(f"\n📊 Temporal Network:")
        print(f"  Years covered: {len(analytics['temporal_network']['timeline'])}")
        print(f"  {analytics['temporal_network']['timeline'][0]} to {analytics['temporal_network']['timeline'][-1]}")

        print(f"\n🔗 Clause Debate Network:")
        print(f"  Clauses: {analytics['clause_debate_network']['total_nodes']}")
        print(f"  Co-occurrence relationships: {len(analytics['clause_debate_network']['links'])}")

        print(f"\n👑 Author Influence:")
        top_author = analytics['author_influence'][0]
        print(f"  Top influencer: {top_author['author']}")
        print(f"  Influence score: {top_author['influence_score']}")
        print(f"  Total authors: {len(analytics['author_influence'])}")

        print(f"\n🔀 Semantic Similarity:")
        print(f"  Clusters: {analytics['semantic_similarity']['total_clusters']}")
        print(f"  Largest cluster: {analytics['semantic_similarity']['largest_cluster']} chunks")

        print(f"\n🗳️  Ratification Tracking:")
        print(f"  Tracked states: {analytics['ratification_tracking']['total_states']}")

        print("\n✅ Advanced analytics ready for visualization")


def main():
    """Main entry point."""
    analytics_engine = AdvancedAnalytics()
    analytics = analytics_engine.generate_all()
    analytics_engine.save_analytics(analytics)
    analytics_engine.print_summary(analytics)
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())
