/**
 * Chart Renderer
 *
 * Renders visualizations using ECharts and D3.js.
 */

class ChartRenderer {
    constructor() {
        this.charts = {};
        this.chartTheme = {
            primaryColor: '#1a5f7a',
            primaryDark: '#0d3d4f',
            primaryLight: '#2d8aa3',
            accentColor: '#c41e3a',
            accentLight: '#ff6b6b',
            bgColor: '#f5f7fa',
            textColor: '#333',
            textLight: '#666',
            borderColor: '#ddd'
        };
    }

    disposeChart(elementId) {
        if (this.charts[elementId]) {
            this.charts[elementId].dispose();
            delete this.charts[elementId];
        }
    }

    renderTopClausesChart(data, elementId) {
        this.disposeChart(elementId);
        const container = document.getElementById(elementId);
        if (!container) return;

        const chart = echarts.init(container);
        this.charts[elementId] = chart;

        const clauseData = data.slice(0, 15);
        const option = {
            tooltip: { trigger: 'axis' },
            grid: { left: 100, right: 20, top: 20, bottom: 20 },
            xAxis: {
                type: 'category',
                data: clauseData.map(c => c.clause),
                axisLabel: { rotate: 45 }
            },
            yAxis: { type: 'value' },
            series: [{
                data: clauseData.map(c => c.count),
                type: 'bar',
                itemStyle: { color: this.chartTheme.primaryColor }
            }]
        };

        chart.setOption(option);
    }

    renderTopIssuesChart(data, elementId) {
        this.disposeChart(elementId);
        const container = document.getElementById(elementId);
        if (!container) return;

        const chart = echarts.init(container);
        this.charts[elementId] = chart;

        const issueData = data.slice(0, 12);
        const option = {
            tooltip: { trigger: 'axis' },
            grid: { left: 100, right: 20, top: 20, bottom: 20 },
            xAxis: {
                type: 'category',
                data: issueData.map(i => i.issue.replace(/_/g, ' ')),
                axisLabel: { rotate: 45 }
            },
            yAxis: { type: 'value' },
            series: [{
                data: issueData.map(i => i.count),
                type: 'bar',
                itemStyle: { color: this.chartTheme.accentColor }
            }]
        };

        chart.setOption(option);
    }

    renderCollectionsPieChart(data, elementId) {
        this.disposeChart(elementId);
        const container = document.getElementById(elementId);
        if (!container) return;

        const chart = echarts.init(container);
        this.charts[elementId] = chart;

        const option = {
            tooltip: { trigger: 'item' },
            legend: { orient: 'vertical', left: 'left' },
            series: [{
                name: 'Chunks',
                type: 'pie',
                radius: '50%',
                data: data.map(c => ({
                    name: c.collection.replace(/_/g, ' '),
                    value: c.chunks
                })),
                emphasis: { itemStyle: { borderColor: this.chartTheme.primaryColor, borderWidth: 2 } }
            }]
        };

        chart.setOption(option);
    }

    renderAuthorsPieChart(data, elementId) {
        this.disposeChart(elementId);
        const container = document.getElementById(elementId);
        if (!container) return;

        const chart = echarts.init(container);
        this.charts[elementId] = chart;

        const topAuthors = data.slice(0, 10);
        const option = {
            tooltip: { trigger: 'item' },
            legend: { orient: 'vertical', left: 'left' },
            series: [{
                name: 'Chunks',
                type: 'pie',
                radius: '50%',
                data: topAuthors.map(a => ({
                    name: a.author,
                    value: a.chunks
                })),
                emphasis: { itemStyle: { borderColor: this.chartTheme.primaryColor, borderWidth: 2 } }
            }]
        };

        chart.setOption(option);
    }

    renderWordFrequencyChart(data, elementId) {
        this.disposeChart(elementId);
        const container = document.getElementById(elementId);
        if (!container) return;

        const chart = echarts.init(container);
        this.charts[elementId] = chart;

        const wordData = data.slice(0, 30);
        const option = {
            tooltip: { trigger: 'axis' },
            grid: { left: 100, right: 20, top: 20, bottom: 20 },
            xAxis: {
                type: 'category',
                data: wordData.map(w => w.word),
                axisLabel: { rotate: 45, interval: 0, fontSize: 10 }
            },
            yAxis: { type: 'value' },
            series: [{
                data: wordData.map(w => w.count),
                type: 'bar',
                itemStyle: { color: this.chartTheme.primaryLight }
            }]
        };

        chart.setOption(option);
    }

    renderChunkSizesChart(data, elementId) {
        this.disposeChart(elementId);
        const container = document.getElementById(elementId);
        if (!container) return;

        const chart = echarts.init(container);
        this.charts[elementId] = chart;

        const option = {
            tooltip: { trigger: 'item' },
            legend: { orient: 'vertical', left: 'left' },
            series: [{
                name: 'Chunks',
                type: 'pie',
                radius: '50%',
                data: Object.entries(data).map(([category, count]) => ({
                    name: category,
                    value: count
                })),
                emphasis: { itemStyle: { borderColor: this.chartTheme.primaryColor, borderWidth: 2 } }
            }]
        };

        chart.setOption(option);
    }

    renderClauseIssueHeatmap(data, elementId) {
        this.disposeChart(elementId);
        const container = document.getElementById(elementId);
        if (!container) return;

        const chart = echarts.init(container);
        this.charts[elementId] = chart;

        const clauses = data.clauses || [];
        const issues = data.issues || [];
        const matrix = data.matrix || [];

        const heatmapData = [];
        matrix.forEach((row, clauseIdx) => {
            issues.forEach((issue, issueIdx) => {
                const count = row.issues[issue] || 0;
                if (count > 0) {
                    heatmapData.push([clauseIdx, issueIdx, count]);
                }
            });
        });

        const option = {
            tooltip: { trigger: 'item' },
            grid: { left: 150, right: 20, top: 20, bottom: 100 },
            xAxis: {
                type: 'category',
                data: issues.map(i => i.replace(/_/g, ' ')),
                axisLabel: { rotate: 45 }
            },
            yAxis: {
                type: 'category',
                data: clauses
            },
            visualMap: {
                min: 0,
                max: Math.max(...heatmapData.map(d => d[2]), 1),
                inRange: { color: ['#f5f7fa', this.chartTheme.accentColor] }
            },
            series: [{
                data: heatmapData,
                type: 'heatmap',
                label: { show: false }
            }]
        };

        chart.setOption(option);
    }

    renderAuthorClauseHeatmap(data, elementId) {
        this.disposeChart(elementId);
        const container = document.getElementById(elementId);
        if (!container) return;

        const chart = echarts.init(container);
        this.charts[elementId] = chart;

        const authors = data.authors || [];
        const clauses = data.clauses || [];
        const matrix = data.matrix || [];

        const heatmapData = [];
        matrix.forEach((row, authorIdx) => {
            clauses.forEach((clause, clauseIdx) => {
                const count = row.clauses[clause] || 0;
                if (count > 0) {
                    heatmapData.push([clauseIdx, authorIdx, count]);
                }
            });
        });

        const option = {
            tooltip: { trigger: 'item' },
            grid: { left: 150, right: 20, top: 20, bottom: 100 },
            xAxis: {
                type: 'category',
                data: clauses,
                axisLabel: { rotate: 45, fontSize: 10 }
            },
            yAxis: {
                type: 'category',
                data: authors
            },
            visualMap: {
                min: 0,
                max: Math.max(...heatmapData.map(d => d[2]), 1),
                inRange: { color: ['#f5f7fa', this.chartTheme.primaryColor] }
            },
            series: [{
                data: heatmapData,
                type: 'heatmap',
                label: { show: false }
            }]
        };

        chart.setOption(option);
    }

    renderDocumentScatterChart(data, elementId) {
        this.disposeChart(elementId);
        const container = document.getElementById(elementId);
        if (!container) return;

        const chart = echarts.init(container);
        this.charts[elementId] = chart;

        const scatterData = data.map(doc => [
            doc.total_words,
            doc.unique_clauses,
            doc.chunks,
            doc.document_id
        ]);

        const option = {
            tooltip: {
                trigger: 'item',
                formatter: (params) => {
                    if (params.componentSubType === 'scatter') {
                        return `${params.value[3]}<br/>Words: ${params.value[0]}<br/>Clauses: ${params.value[1]}<br/>Chunks: ${params.value[2]}`;
                    }
                }
            },
            grid: { left: 80, right: 20, top: 20, bottom: 80 },
            xAxis: { type: 'value', name: 'Total Words' },
            yAxis: { type: 'value', name: 'Unique Clauses' },
            series: [{
                data: scatterData,
                type: 'scatter',
                symbolSize: (val) => Math.sqrt(val[2]) * 8,
                itemStyle: { color: this.chartTheme.primaryColor, opacity: 0.6 }
            }]
        };

        chart.setOption(option);
    }

    renderDocumentsBarChart(data, elementId) {
        this.disposeChart(elementId);
        const container = document.getElementById(elementId);
        if (!container) return;

        const chart = echarts.init(container);
        this.charts[elementId] = chart;

        const topDocs = data.slice(0, 12);
        const option = {
            tooltip: { trigger: 'axis' },
            grid: { left: 150, right: 20, top: 20, bottom: 20 },
            xAxis: {
                type: 'category',
                data: topDocs.map(d => d.document_id.substring(0, 20)),
                axisLabel: { rotate: 45 }
            },
            yAxis: { type: 'value' },
            series: [{
                data: topDocs.map(d => d.chunks),
                type: 'bar',
                itemStyle: { color: this.chartTheme.primaryLight }
            }]
        };

        chart.setOption(option);
    }

    renderDocumentNetworkGraph(data, elementId) {
        const container = document.getElementById(elementId);
        if (!container || data.length === 0) return;

        d3.select(`#${elementId}`).selectAll('*').remove();

        const width = container.clientWidth;
        const height = 500;

        const docs = new Map();
        data.forEach(rel => {
            if (!docs.has(rel.document_1)) {
                docs.set(rel.document_1, { id: rel.document_1 });
            }
            if (!docs.has(rel.document_2)) {
                docs.set(rel.document_2, { id: rel.document_2 });
            }
        });

        const nodes = Array.from(docs.values());
        const links = data.slice(0, 30).map(rel => ({
            source: rel.document_1,
            target: rel.document_2,
            strength: rel.similarity_score
        }));

        const simulation = d3.forceSimulation(nodes)
            .force('link', d3.forceLink(links).id(d => d.id).distance(100).strength(0.5))
            .force('charge', d3.forceManyBody().strength(-300))
            .force('center', d3.forceCenter(width / 2, height / 2));

        const svg = d3.select(`#${elementId}`)
            .append('svg')
            .attr('width', width)
            .attr('height', height);

        const link = svg.selectAll('line')
            .data(links)
            .join('line')
            .attr('stroke', this.chartTheme.borderColor)
            .attr('stroke-width', d => Math.sqrt(d.strength));

        const node = svg.selectAll('circle')
            .data(nodes)
            .join('circle')
            .attr('r', 8)
            .attr('fill', this.chartTheme.primaryColor)
            .call(d3.drag()
                .on('start', dragstarted)
                .on('drag', dragged)
                .on('end', dragended));

        const label = svg.selectAll('text')
            .data(nodes)
            .join('text')
            .text(d => d.id.substring(0, 15))
            .attr('font-size', 10)
            .attr('text-anchor', 'middle');

        simulation.on('tick', () => {
            link
                .attr('x1', d => d.source.x)
                .attr('y1', d => d.source.y)
                .attr('x2', d => d.target.x)
                .attr('y2', d => d.target.y);

            node
                .attr('cx', d => d.x)
                .attr('cy', d => d.y);

            label
                .attr('x', d => d.x)
                .attr('y', d => d.y - 12);
        });

        const dragstarted = (event, d) => {
            if (!event.active) simulation.alphaTarget(0.3).restart();
            d.fx = d.x;
            d.fy = d.y;
        };

        const dragged = (event, d) => {
            d.fx = event.x;
            d.fy = event.y;
        };

        const dragended = (event, d) => {
            if (!event.active) simulation.alphaTarget(0);
            d.fx = null;
            d.fy = null;
        };
    }

    renderWordCloud(data, elementId) {
        const container = document.getElementById(elementId);
        if (!container || data.length === 0) return;

        d3.select(`#${elementId}`).selectAll('*').remove();

        const width = container.clientWidth;
        const height = 500;
        const words = data.slice(0, 50);

        const maxCount = Math.max(...words.map(w => w.count));
        const fontSizeScale = d3.scaleLinear()
            .domain([words[words.length - 1].count, maxCount])
            .range([12, 48]);

        const svg = d3.select(`#${elementId}`)
            .append('svg')
            .attr('width', width)
            .attr('height', height);

        const group = svg.append('g')
            .attr('transform', `translate(${width / 2},${height / 2})`);

        const colors = [
            this.chartTheme.primaryColor,
            this.chartTheme.accentColor,
            this.chartTheme.primaryLight,
            this.chartTheme.accentLight
        ];

        let angle = 0;
        group.selectAll('text')
            .data(words)
            .join('text')
            .attr('x', () => (Math.random() - 0.5) * width * 0.8)
            .attr('y', () => (Math.random() - 0.5) * height * 0.8)
            .attr('font-size', d => fontSizeScale(d.count))
            .attr('font-weight', 'bold')
            .attr('fill', (d, i) => colors[i % colors.length])
            .attr('text-anchor', 'middle')
            .attr('opacity', 0.8)
            .text(d => d.word);
    }
}
