# Advanced Analytics Guide

This guide covers the advanced analytics features in the Constitutional Research System dashboard, including filtering, temporal analysis, author comparison, and report generation.

## Overview

The analytics dashboard provides sophisticated analysis tools for exploring the constitutional corpus:

- **Analytics Filtering**: Drill down into specific collections, authors, and time periods
- **Temporal Analysis**: Track how constitutional concepts evolved through debates
- **Author Comparison**: Compare perspectives and debate priorities between authors
- **Report Generation**: Export analytics in JSON, HTML, or printable formats

---

## 1. Analytics Filtering

### Purpose
Filter all analytics to focus on specific subsets of the corpus, enabling targeted research.

### How to Use

1. **Access Filters**: The filter section appears at the top of the dashboard
2. **Select Criteria**:
   - **Collections**: Choose one or more document collections (Constitution, Madison's Notes, etc.)
   - **Authors**: Select specific authors to analyze
   - **Date Range**: Set start and end dates to focus on specific time periods
3. **Apply**: Click "Apply Filters" to recalculate all visualizations
4. **Reset**: Click "Reset" to return to full corpus analysis

### Filter Examples

**Example 1: Federalist vs Anti-Federalist Comparison**
- Select two collections: "federalist_papers" and "antifederalist_writings"
- Compare how each side discussed the same constitutional clauses

**Example 2: James Madison's Impact**
- Select author: "James Madison"
- Analyze the specific clauses Madison focused on in his writings

**Example 3: Summer 1787 Convention Debates**
- Set date range: June 1787 - September 1787
- Focus on Madison's Notes from the Constitutional Convention

### Filtered Analytics

When filters are active, all visualizations update to show:
- Top clauses within filtered subset
- Most discussed issues
- Author contributions (if not filtering by author)
- Collection distribution
- All matrices and network visualizations

---

## 2. Temporal Analysis

### Purpose
Understand how constitutional concepts evolved across time and different documents.

### Visualization: Clause Mentions Over Time

**What It Shows**:
- Line chart tracking constitutional clause mentions across different documents
- X-axis: Document/time periods
- Y-axis: Mention count
- Multiple lines represent top 8 constitutional clauses

**Reading the Chart**:
- Rising trend = Increasing focus on that clause
- Flat line = Consistent discussion
- New line appearing = Clause became relevant later in debates

### Example Insights

**The Separation of Powers Clause**:
- Heavily discussed in Constitution (1787)
- Reappears in Federalist Papers with focus on presidential powers
- Anti-Federalists emphasize concerns about executive overreach

**Commerce Clause**:
- Debates in Convention (Madison's Notes)
- Central to Federalist ratification arguments
- Anti-Federalists cite it as evidence of federal overreach

### Using Temporal Analysis

1. Look at major clauses to see debate progression
2. Compare same clause across documents
3. Identify when issues became prominent
4. Trace how understanding evolved

---

## 3. Author Comparative Analysis

### Purpose
Compare how different authors/delegates prioritized constitutional issues.

### How to Use

1. **Select First Author**: Choose first author from dropdown
2. **Select Second Author**: Choose second author from dropdown
3. **Click Compare**: System generates comparison chart

### Comparison Metrics

**Agreement Score** (0-100%):
- 0% = No overlap in discussed clauses/issues
- 100% = Identical debate focus
- Shows ideological alignment on constitutional topics

**Side-by-Side Bar Chart**:
- Shows top 8 clauses for each author
- Red bars = Second author
- Blue bars = First author
- Overlapping bars = Shared focus areas

### Comparison Examples

**Hamilton vs Madison**:
- Both focus on separation of powers
- Hamilton emphasizes executive power
- Madison focuses on legislative supremacy
- Shows their alliance and differences

**Federalist vs Anti-Federalist Leaders**:
- Alexander Hamilton vs George Mason
- James Wilson vs Patrick Henry
- Reveals core ideological splits

**Conservative vs Progressive Founders**:
- George Washington vs George Mason
- James Madison vs Elbridge Gerry
- Shows generational and philosophical differences

### Key Findings

The comparison reveals:
1. **Shared Concerns**: Which clauses both authors discussed
2. **Different Priorities**: Where they focused differently
3. **Ideological Alignment**: Agreement score shows compatibility
4. **Debate Strategy**: What arguments each side emphasized

---

## 4. Report Generation

### Purpose
Export analytics data for external use, publication, or archival.

### Export Formats

#### JSON Report
- **Use Case**: Programmatic access, data analysis tools, integration
- **Contains**: All core metrics, top items, distributions
- **File**: `constitutional-analytics-YYYY-MM-DD.json`
- **Size**: 10-50KB typical

**JSON Structure**:
```json
{
  "title": "Constitutional Research Corpus Analytics Report",
  "generated": "2024-01-15T10:30:00",
  "overview": {
    "total_chunks": 3000,
    "total_documents": 7,
    "total_words": 1500000,
    ...
  },
  "top_clauses": [...],
  "top_issues": [...],
  "collections": [...],
  "authors": [...],
  "documents": [...],
  "word_frequency": [...]
}
```

#### HTML Report
- **Use Case**: Human-readable reports, publications, sharing with non-technical audiences
- **Contains**: Executive summary, tables, statistics with formatting
- **File**: `constitutional-analytics-YYYY-MM-DD.html`
- **Features**: 
  - Professional styling
  - Print-friendly layout
  - Responsive design
  - Can be opened in any browser

**HTML Sections**:
1. Executive Summary (stat cards)
2. Top Constitutional Clauses (table)
3. Top Thematic Issues (table)
4. Collections Breakdown (table)
5. Top Authors (table)

#### Print Report
- **Use Case**: Create PDF via browser print dialog
- **Steps**:
  1. Click "Print Report"
  2. Browser print dialog opens
  3. Select "Save as PDF" or print to paper
  4. Customizable via print settings

### Report Contents

All reports include:

**Corpus Statistics**:
- Total chunks: Number of text segments
- Documents: Individual documents analyzed
- Total words: All text combined
- Collections: Breakdown by source
- Constitutional clauses: Unique clauses referenced
- Thematic issues: Unique themes discussed

**Top Items**:
- 20 most discussed constitutional clauses
- 15 most discussed thematic issues
- Top 10 authors by contribution
- Collection statistics

**Advanced Metrics**:
- Clause distribution (average, median)
- Issue distribution (average, median)
- Document composition (chunks and words)
- Author contribution levels

### Report Workflow

1. **Analyze**: Interact with dashboard, apply filters, generate insights
2. **Export**: Click desired export format
3. **Use**: Share, publish, or integrate with other tools

### Example Use Cases

**Academic Research**:
```
1. Filter by specific debate topics
2. Generate HTML report
3. Print to PDF for publication appendix
4. Export JSON for citation statistics
```

**Collaborative Research**:
```
1. Team member generates full report
2. Shares HTML via email
3. Colleagues review findings
4. Discuss specific clauses from report
```

**Data Integration**:
```
1. Export JSON report
2. Import into analysis tool (Python, R)
3. Combine with other constitutional sources
4. Create enhanced comparative analysis
```

---

## Workflow Examples

### Example 1: Tracing a Constitutional Controversy

**Goal**: Understand the Commerce Clause debate

**Steps**:
1. **Temporal Analysis**: View Commerce Clause in temporal chart
   - See where it's discussed most
   - Identify which documents focus on it

2. **Apply Filters**: Select "federalist_papers"
   - See how Federalists justified Commerce Clause
   - Compare mentions to other clauses

3. **Author Comparison**: Hamilton vs Mason
   - Hamilton emphasizes federal commerce power
   - Mason worries about overreach
   - Agreement score shows disagreement (20-30%)

4. **Export Report**: Generate HTML report
   - Share findings with research group
   - Include Commerce Clause statistics

### Example 2: Founder Influence Analysis

**Goal**: Determine who had most impact on final Constitution

**Steps**:
1. **Filter by Madison's Notes**
   - See which authors are most quoted/discussed

2. **Author Comparison**: Madison vs Other Delegates
   - Show Madison's unique contributions
   - Identify his allies (high agreement) vs opponents

3. **Temporal Analysis**:
   - Track when Madison's ideas gained traction
   - Show evolution of his positions

4. **Generate Reports**:
   - JSON for quantitative analysis
   - HTML for publication

### Example 3: Ideological Coalitions

**Goal**: Map Federalist vs Anti-Federalist fault lines

**Steps**:
1. **Create Comparison Matrix**:
   - Key Federalist: Alexander Hamilton
   - Key Anti-Federalist: George Mason
   - Compare agreement score (likely <30%)

2. **Analyze Shared Concerns**:
   - Separation of Powers (both discuss)
   - Commerce Clause (sharp disagreement)
   - Presidential Power (opposite views)

3. **Temporal Analysis**:
   - When did each side mobilize on specific issues?
   - How did arguments evolve?

4. **Report**:
   - Document coalition structure
   - Export statistics
   - Publish findings

---

## Best Practices

### 1. Progressive Filtering
- Start with full corpus
- Apply filters incrementally
- Observe how visualizations change
- Identify patterns in subsets

### 2. Temporal Insight
- Use for understanding debate progression
- Compare same clause across documents
- Note when issues became salient
- Track evolution of positions

### 3. Author Analysis
- Compare leaders with followers
- Identify ideological blocs
- Find surprising alliances
- Document disagreements

### 4. Report Usage
- Export at key analytical stages
- Use for checkpoints in research
- Share with collaborators
- Maintain archive of findings

### 5. Data Interpretation
- Remember filter applies to all charts
- Check corpus size when interpreting
- Consider author representation
- Validate findings with source texts

---

## Troubleshooting

### Report Download Not Working
- Check browser allows downloads
- Ensure JavaScript is enabled
- Try different export format
- Check file size in browser console

### Filters Not Applying
- Ensure selections are made
- Click "Apply Filters" explicitly
- Check for JavaScript errors (console)
- Reload page and reapply

### Comparison Shows No Data
- Verify both authors exist in corpus
- Check if authors have different collections
- Try comparing with different author pair
- Ensure sufficient data in filtered subset

### Temporal Chart Empty
- Verify corpus has date information
- Check if filter excludes all dated documents
- Reset filters to include all data
- Try different collection

---

## Technical Details

### API Endpoints

```
POST /api/analytics/filtered
  Payload: {"collections": [], "authors": [], "start_date": "", "end_date": ""}
  Returns: Filtered corpus overview and analyses

GET /api/analytics/temporal
  Returns: Timeline of clause mentions by document

POST /api/analytics/author-comparison
  Payload: {"author1": "", "author2": "", "clause": ""}
  Returns: Comparison metrics and statistics

GET /api/analytics/report
  Returns: Executive summary report JSON
```

### Filtering Algorithm

1. Load full corpus chunks
2. Filter by collection (if specified)
3. Filter by author (if specified)
4. Filter by date range (if specified)
5. Run analyses on filtered subset
6. Return results to dashboard

### Performance Considerations

- Filtering is real-time, but may take 1-2 seconds
- Large filters (many authors) take longer
- Comparison uses top 8 items for clarity
- Reports generate instantly from cached data

---

## API for Custom Tools

If integrating with external tools, use these endpoints:

```javascript
// Get filtered analytics
fetch('/api/analytics/filtered', {
  method: 'POST',
  headers: {'Content-Type': 'application/json'},
  body: JSON.stringify({
    collections: ['constitution'],
    authors: ['James Madison'],
    start_date: '1787-06-01',
    end_date: '1787-09-30'
  })
}).then(r => r.json())

// Get temporal data
fetch('/api/analytics/temporal')
  .then(r => r.json())

// Compare authors
fetch('/api/analytics/author-comparison', {
  method: 'POST',
  headers: {'Content-Type': 'application/json'},
  body: JSON.stringify({
    author1: 'Alexander Hamilton',
    author2: 'George Mason'
  })
}).then(r => r.json())

// Get report
fetch('/api/analytics/report')
  .then(r => r.json())
```

---

## See Also

- [Analytics Dashboard Overview](README.md)
- [API Reference](API.md)
- [Corpus Structure](METADATA_SCHEMA.md)
- [Source Documentation](SOURCES.md)
