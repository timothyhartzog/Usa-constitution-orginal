/**
 * Advanced Chart Renderer
 * Visualizations for temporal networks, debates, influence, and more
 */

class AdvancedChartRenderer {
    constructor(echartsLib, d3Lib) {
        this.echarts = echartsLib;
        this.d3 = d3Lib;
    }

    renderTemporalNetworkChart(chartId, data) {
        if (!data || !data.evolution) return;

        const chart = this.echarts.init(document.getElementById(chartId));
        const timeline = data.timeline;
        const evolution = data.evolution;

        // Prepare top clauses across all years
        const allClauses = new Set();
        evolution.forEach(year => {
            Object.keys(year.clauses || {}).forEach(c => allClauses.add(c));
        });

        const clauses = Array.from(allClauses).slice(0, 10); // Top 10
        const series = clauses.map(clause => ({
            name: clause,
            type: 'line',
            data: evolution.map(year => year.clauses[clause] || 0),
            smooth: true,
        }));

        const option = {
            title: { text: 'Constitutional Ideas Over Time' },
            tooltip: { trigger: 'axis' },
            legend: {
                data: clauses,
                top: 30,
            },
            xAxis: {
                type: 'category',
                data: timeline,
            },
            yAxis: {
                type: 'value',
                name: 'Mentions',
            },
            series: series,
            grid: { top: 100, left: 60, right: 20, bottom: 50 },
        };

        chart.setOption(option);
    }

    renderClauseDebateNetworkChart(chartId, data) {
        if (!data || !data.nodes) return;

        const chart = this.echarts.init(document.getElementById(chartId));

        // Scale node sizes by value
        const maxValue = Math.max(...data.nodes.map(n => n.value));
        const nodes = data.nodes.map(n => ({
            ...n,
            symbolSize: 10 + (n.value / maxValue) * 30,
        }));

        const option = {
            title: { text: 'Clause Debate Network (Which Clauses Were Debated Together)' },
            tooltip: {},
            legend: [{ data: ['Clauses'] }],
            series: [{
                name: 'Clause Relationships',
                type: 'graph',
                layout: 'force',
                data: nodes,
                links: data.links,
                roam: true,
                label: {
                    position: 'right',
                    formatter: '{b}',
                },
                emphasis: {
                    focus: 'adjacency',
                    lineStyle: {
                        width: 2,
                    },
                },
                force: {
                    repulsion: 100,
                    edgeLength: 100,
                    friction: 0.8,
                },
            }],
        };

        chart.setOption(option);
    }

    renderAuthorInfluenceChart(chartId, data) {
        if (!data || data.length === 0) return;

        const chart = this.echarts.init(document.getElementById(chartId));

        const authors = data.map(d => d.author);
        const scores = data.map(d => d.influence_score);

        const option = {
            title: { text: 'Author Influence Ranking' },
            tooltip: {
                formatter: (params) => {
                    const item = params[0];
                    const dataIndex = item.dataIndex;
                    const author = data[dataIndex];
                    return `
                        <b>${author.author}</b><br/>
                        Influence: ${item.value.toFixed(1)}<br/>
                        Clauses: ${author.clause_mentions}<br/>
                        Documents: ${author.documents_contributed}<br/>
                        Words: ${author.total_words}
                    `;
                },
            },
            xAxis: {
                type: 'category',
                data: authors,
                axisLabel: { interval: 0, rotate: 45 },
            },
            yAxis: {
                type: 'value',
                name: 'Influence Score',
            },
            series: [{
                data: scores,
                type: 'bar',
                itemStyle: {
                    color: new this.echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        { offset: 0, color: '#83bff6' },
                        { offset: 0.5, color: '#188df0' },
                        { offset: 1, color: '#188df0' },
                    ]),
                },
            }],
            grid: { top: 50, left: 60, right: 20, bottom: 100 },
        };

        chart.setOption(option);
    }

    renderSemanticSimilarityChart(chartId, data) {
        if (!data || !data.clusters) return;

        const chart = this.echarts.init(document.getElementById(chartId));

        // Get top clusters
        const topClusters = data.clusters.slice(0, 10);
        const clauses = topClusters.map(c => c.clause);
        const sizes = topClusters.map(c => c.size);

        const option = {
            title: { text: 'Semantic Clustering by Constitutional Clause' },
            tooltip: {
                formatter: (params) => {
                    if (params[0]) {
                        const index = params[0].dataIndex;
                        const cluster = topClusters[index];
                        return `<b>${cluster.clause}</b><br/>Chunk count: ${cluster.size}`;
                    }
                    return '';
                },
            },
            xAxis: {
                type: 'category',
                data: clauses,
                axisLabel: { interval: 0, rotate: 45 },
            },
            yAxis: {
                type: 'value',
                name: 'Chunks in Cluster',
            },
            series: [{
                data: sizes,
                type: 'bar',
                itemStyle: {
                    color: '#f25c75',
                },
            }],
            grid: { top: 50, left: 60, right: 20, bottom: 100 },
        };

        chart.setOption(option);
    }

    renderRatificationChart(chartId, data) {
        if (!data || !data.ratification) return;

        const chart = this.echarts.init(document.getElementById(chartId));

        // Count how many clauses each state influenced
        const stateClauses = {};
        data.ratification.forEach(item => {
            Object.keys(item.states_mentioned).forEach(state => {
                stateClauses[state] = (stateClauses[state] || 0) + 1;
            });
        });

        const states = Object.keys(stateClauses).sort((a, b) => stateClauses[b] - stateClauses[a]);
        const counts = states.map(s => stateClauses[s]);

        const option = {
            title: { text: 'State Involvement in Constitutional Clauses' },
            tooltip: {
                formatter: (params) => {
                    if (params[0]) {
                        return `${params[0].name}<br/>Clauses mentioned: ${params[0].value}`;
                    }
                    return '';
                },
            },
            xAxis: {
                type: 'category',
                data: states,
                axisLabel: { interval: 0, rotate: 45 },
            },
            yAxis: {
                type: 'value',
                name: 'Clause Count',
            },
            series: [{
                data: counts,
                type: 'bar',
                itemStyle: {
                    color: '#91cc75',
                },
            }],
            grid: { top: 50, left: 60, right: 20, bottom: 100 },
        };

        chart.setOption(option);
    }

    renderTemporalHeatmap(chartId, data) {
        if (!data || !data.evolution) return;

        const chart = this.echarts.init(document.getElementById(chartId));

        // Prepare data for heatmap: years × clauses
        const allClauses = new Set();
        data.evolution.forEach(year => {
            Object.keys(year.clauses || {}).forEach(c => allClauses.add(c));
        });

        const clauses = Array.from(allClauses).slice(0, 15);
        const heatmapData = [];

        data.evolution.forEach((year, yearIndex) => {
            clauses.forEach((clause, clauseIndex) => {
                heatmapData.push([
                    yearIndex,
                    clauseIndex,
                    year.clauses[clause] || 0,
                ]);
            });
        });

        const option = {
            title: { text: 'Constitutional Debate Heatmap (1786-1791)' },
            tooltip: { position: 'top' },
            xAxis: {
                type: 'category',
                data: data.timeline,
            },
            yAxis: {
                type: 'category',
                data: clauses,
            },
            visualMap: {
                min: 0,
                max: 5,
                text: ['High', 'Low'],
                realtime: true,
                calculable: true,
                inRange: {
                    color: ['blue', 'red'],
                },
            },
            series: [{
                name: 'Mentions',
                type: 'heatmap',
                data: heatmapData,
                emphasis: {
                    itemStyle: {
                        borderColor: '#333',
                        borderWidth: 1,
                    },
                },
            }],
            grid: { top: 50, left: 100, right: 20, bottom: 50 },
        };

        chart.setOption(option);
    }
}
