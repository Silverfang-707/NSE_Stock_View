<script lang="ts">
    import { onMount } from "svelte";
    import * as echarts from "echarts";
    import api from "$lib/api";

    let chartContainer: HTMLDivElement;
    let chart: echarts.ECharts;

    // ── State ──────────────────────────────────────
    let symbols: string[] = $state([]);
    let search = $state("");
    let selectedSymbol = $state("");
    let loading = $state(false);
    let noData = $state(false);

    // Filters
    let timeframe = $state("Daily");
    let seriesType = $state("EQ");

    // Date range — default last 1 year
    const today = new Date();
    const oneYearAgo = new Date(today);
    oneYearAgo.setFullYear(today.getFullYear() - 1);

    let dateFrom = $state(oneYearAgo.toISOString().slice(0, 10));
    let dateTo   = $state(today.toISOString().slice(0, 10));

    // Dropdowns open state
    let tfOpen  = $state(false);
    let stOpen  = $state(false);

    const TIMEFRAMES = ["Daily", "Weekly", "Monthly", "Quarterly", "Half Year", "Yearly"];
    // FIX 1b: Valid NSE series types
    const SERIES_TYPES = ["EQ", "BE", "GB", "N5", "ND", "DR", "SM"];

    // ── Derived ────────────────────────────────────
    const filteredSymbols = $derived(
        symbols
            .filter(s => s.toLowerCase().includes(search.toLowerCase()))
            .slice(0, 200)
    );

    // ── Candle aggregation helpers ─────────────────
    function getGroupKey(dateStr: string, tf: string): string {
        const d = new Date(dateStr);
        const y = d.getFullYear();
        const m = d.getMonth(); 
        const day = d.getDate();

        if (tf === "Daily") return dateStr;

        if (tf === "Weekly") {
            const tmp = new Date(d);
            tmp.setHours(0, 0, 0, 0);
            tmp.setDate(tmp.getDate() - ((tmp.getDay() + 6) % 7));
            return tmp.toISOString().slice(0, 10);
        }

        if (tf === "Monthly") return `${y}-${String(m + 1).padStart(2, "0")}`;

        if (tf === "Quarterly") {
            const q = Math.floor(m / 3) + 1;
            return `${y}-Q${q}`;
        }

        if (tf === "Half Year") {
            const h = m < 6 ? "H1" : "H2";
            return `${y}-${h}`;
        }

        if (tf === "Yearly") return `${y}`;

        return dateStr;
    }

    function aggregateCandles(candles: any[], tf: string) {
        if (tf === "Daily") return candles;

        const groups: Map<string, any[]> = new Map();

        for (const c of candles) {
            const key = getGroupKey(c.trade_date, tf);
            if (!groups.has(key)) groups.set(key, []);
            groups.get(key)!.push(c);
        }

        const result: any[] = [];
        for (const [key, cs] of groups) {
            result.push({
                trade_date: key,
                open_price:  cs[0].open_price,
                close_price: cs[cs.length - 1].close_price,
                high_price:  Math.max(...cs.map((c: any) => c.high_price)),
                low_price:   Math.min(...cs.map((c: any) => c.low_price)),
                volume:      cs.reduce((s: number, c: any) => s + c.volume, 0),
            });
        }

        return result;
    }

    // ── Data Fetching ──────────────────────────────
    // FIX 3: Load symbols based on selected series type
    async function loadSymbols() {
        try {
            const response = await api.get(`/symbols/${seriesType}`);
            symbols = response.data;
        } catch (err) {
            console.error("Failed to load symbols:", err);
            symbols = [];
        }
    }

    async function loadChart(symbol: string) {
        if (!symbol || !chart) return;

        loading = true;
        noData = false;

        try {
            const response = await api.get(`/candles/${symbol}`);
            let candles: any[] = response.data;

            candles = candles.filter(c =>
                c.trade_date >= dateFrom && c.trade_date <= dateTo
            );

            candles = aggregateCandles(candles, timeframe);

            if (!candles.length) {
                chart.clear();
                noData = true;
                loading = false;
                return;
            }

            const dates  = candles.map(c => c.trade_date);
            const values = candles.map(c => [
                c.open_price,
                c.close_price,
                c.low_price,
                c.high_price,
            ]);
            const volumes = candles.map(c => c.volume);

            chart.setOption({
                backgroundColor: "transparent",
                animation: true,

                title: {
                    text: symbol,
                    subtext: `${timeframe}  ·  ${seriesType}  ·  ${dateFrom} → ${dateTo}`,
                    left: "center",
                    textStyle: { color: "#f8fafc", fontSize: 28, fontWeight: "800", letterSpacing: 1 },
                    subtextStyle: { color: "#94a3b8", fontSize: 13, fontWeight: "500" },
                },

                tooltip: {
                    trigger: "axis",
                    axisPointer: { type: "cross", lineStyle: { color: 'rgba(255,255,255,0.2)' } },
                    backgroundColor: "rgba(15, 23, 42, 0.9)",
                    borderColor: "#334155",
                    backdropFilter: "blur(10px)",
                    borderRadius: 12,
                    padding: 16,
                    textStyle: { color: "#f8fafc" },
                    formatter(params: any) {
                        const c = params.find((p: any) => p.seriesType === "candlestick");
                        if (!c) return "";
                        const [o, cl, l, h] = c.data;
                        const isUp = cl >= o;
                        const dot = `<span style="display:inline-block;width:10px;height:10px;border-radius:50%;background:${isUp ? "#10b981" : "#ef4444"};margin-right:8px;box-shadow: 0 0 8px ${isUp ? '#10b981' : '#ef4444'};"></span>`;
                        return `
                            <div style="font-size:14px;line-height:1.8;font-family:'JetBrains Mono', monospace;">
                                <b style="color:#cbd5e1;font-size:12px;">${c.axisValue}</b><br/>
                                <div style="margin-top:8px; border-top: 1px solid #334155; padding-top: 8px;">
                                    ${dot}<br/>
                                    <span style="color:#94a3b8">Open&nbsp;&nbsp;&nbsp;</span><b style="float:right;">${o.toFixed(2)}</b><br/>
                                    <span style="color:#94a3b8">Close&nbsp;&nbsp;</span><b style="float:right;">${cl.toFixed(2)}</b><br/>
                                    <span style="color:#94a3b8">Lowest&nbsp;</span><b style="float:right;">${l.toFixed(2)}</b><br/>
                                    <span style="color:#94a3b8">Highest</span><b style="float:right;">${h.toFixed(2)}</b>
                                </div>
                            </div>`;
                    }
                },

                axisPointer: { link: [{ xAxisIndex: "all" }] },

                grid: [
                    { left: "4%", right: "4%", top: "15%", bottom: "30%"  },
                    { left: "4%", right: "4%", top: "72%", bottom: "8%"  },
                ],

                xAxis: [
                    {
                        type: "category", data: dates, gridIndex: 0,
                        axisLine: { lineStyle: { color: "#334155" } },
                        axisLabel: { color: "#94a3b8", fontSize: 12, margin: 12 },
                        axisTick: { show: false },
                        splitLine: { show: false },
                    },
                    {
                        type: "category", data: dates, gridIndex: 1,
                        axisLine: { lineStyle: { color: "#334155" } },
                        axisLabel: { show: false },
                        axisTick: { show: false },
                    },
                ],

                yAxis: [
                    {
                        scale: true, gridIndex: 0,
                        axisLine: { show: false },
                        axisTick: { show: false },
                        splitLine: { lineStyle: { color: "rgba(255,255,255,0.05)", type: "dashed" } },
                        axisLabel: { color: "#94a3b8", fontSize: 12 },
                    },
                    {
                        scale: true, gridIndex: 1,
                        axisLine: { show: false },
                        axisTick: { show: false },
                        splitLine: { show: false },
                        axisLabel: { show: false },
                    },
                ],

                dataZoom: [
                    { type: "inside", xAxisIndex: [0, 1], start: candles.length > 60 ? 70 : 0, end: 100 },
                    { 
                        type: "slider", xAxisIndex: [0, 1], bottom: "2%", height: 24,
                        borderColor: "transparent", backgroundColor: "rgba(30, 41, 59, 0.5)", fillerColor: "rgba(56, 189, 248, 0.15)",
                        textStyle: { color: "#94a3b8" }, handleStyle: { color: "#38bdf8", shadowBlur: 4, shadowColor: "rgba(56, 189, 248, 0.5)" },
                        dataBackground: { lineStyle: { color: "#475569" }, areaStyle: { color: "#334155" } },
                        selectedDataBackground: { lineStyle: { color: "#38bdf8" }, areaStyle: { color: "#38bdf8" } }
                    },
                ],

                series: [
                    {
                        type: "candlestick",
                        xAxisIndex: 0, yAxisIndex: 0,
                        data: values,
                        itemStyle: {
                            color: "#10b981",
                            color0: "#ef4444",
                            borderColor: "#10b981",
                            borderColor0: "#ef4444",
                        },
                    },
                    {
                        type: "bar",
                        xAxisIndex: 1, yAxisIndex: 1,
                        data: volumes,
                        itemStyle: {
                            borderRadius: [4, 4, 0, 0],
                            color(params: any) {
                                const [o, c] = values[params.dataIndex];
                                return c >= o ? "rgba(16, 185, 129, 0.4)" : "rgba(239, 68, 68, 0.4)";
                            }
                        },
                    },
                ],
            }, true);

        } catch (err) {
            console.error(err);
        }

        loading = false;
    }

    async function selectSymbol(symbol: string) {
        selectedSymbol = symbol;
        tfOpen = false;
        stOpen = false;
        await loadChart(symbol);
    }

    // FIX 3: Re-fetch symbols when series type changes
    async function applyFilters() {
        tfOpen = false;
        stOpen = false;
        
        await loadSymbols();
        
        if (symbols.length > 0) {
            // Select the first symbol of the newly loaded series type
            selectedSymbol = symbols[0];
            await loadChart(selectedSymbol);
        } else {
            // Handle edge case if the series is totally empty
            chart.clear();
            noData = true;
            selectedSymbol = "";
        }
    }

    // FIX 2: Correct ingestion route
    async function ingestLatest() {
        loading = true;
        try {
            const response = await api.get("/admin/ingest");
            console.log(response.data);
            alert(`✅ Imported ${response.data.rows} rows`);

            // Refresh everything after import
            await loadSymbols();
            if (selectedSymbol) {
                await loadChart(selectedSymbol);
            }
        } catch (err) {
            console.error(err);
            alert("❌ Ingestion failed");
        }
        loading = false; 
    }

    // FIX 3: Initialize with new loadSymbols flow
    onMount(async () => {
        chart = echarts.init(chartContainer);

        await loadSymbols();

        if (symbols.length > 0) {
            selectedSymbol = symbols[0];
            await loadChart(selectedSymbol);
        }

        window.addEventListener("resize", () => chart.resize());
    });
</script>

<!-- Close dropdowns on outside click -->
<svelte:window onclick={(e) => {
    const t = e.target as HTMLElement;
    if (!t.closest(".dd-wrap")) { tfOpen = false; stOpen = false; }
}} />

<div class="layout">
    <!-- ── Sidebar ── -->
    <aside class="sidebar">
        <div class="logo">
            <div class="logo-icon-wrap">
                <span class="logo-icon">📈</span>
            </div>
            <span>NSE Market</span>
        </div>

        <div class="search-wrap">
            <span class="search-icon">⌕</span>
            <input
                bind:value={search}
                placeholder="Search symbol..."
            />
        </div>

        <div class="symbol-count">
            <span>Market Symbols</span>
            <span class="badge">{filteredSymbols.length} / {symbols.length}</span>
        </div>

        <div class="symbol-list">
            {#each filteredSymbols as symbol (symbol)}
                <button
                    class:selected={symbol === selectedSymbol}
                    onclick={() => selectSymbol(symbol)}
                >
                    {symbol}
                </button>
            {:else}
                {#if symbols.length === 0}
                    <div class="empty-state">
                        <div class="mini-spinner"></div>
                        Fetching symbols...
                    </div>
                {:else}
                    <div class="empty-state">No results for "{search}"</div>
                {/if}
            {/each}
        </div>
    </aside>

    <!-- ── Main ── -->
    <main class="main">
        <!-- Filter Bar -->
        <div class="filter-bar">
            <!-- Grouping Action & Dates on the left -->
            <div class="filters-left">
                <button class="action-btn" onclick={ingestLatest}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                        <polyline points="7 10 12 15 17 10"></polyline>
                        <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                    Import Latest
                </button>

                <div class="divider"></div>

                <label class="date-label">
                    <span>From</span>
                    <input type="date" bind:value={dateFrom} class="date-input" />
                </label>
                <label class="date-label">
                    <span>To</span>
                    <input type="date" bind:value={dateTo} class="date-input" />
                </label>
            </div>

            <!-- Grouping Dropdowns & Apply on the right -->
            <div class="filters-right">
                <div class="dd-wrap">
                    <button class="dd-btn" onclick={() => { tfOpen = !tfOpen; stOpen = false; }}>
                        {timeframe}
                        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" class:rotate={tfOpen}>
                            <path d="M2 4l4 4 4-4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                        </svg>
                    </button>
                    {#if tfOpen}
                        <div class="dd-menu glass-panel">
                            {#each TIMEFRAMES as tf}
                                <button
                                    class:active={tf === timeframe}
                                    onclick={() => { timeframe = tf; tfOpen = false; applyFilters(); }}
                                >{tf}</button>
                            {/each}
                        </div>
                    {/if}
                </div>

                <div class="dd-wrap">
                    <button class="dd-btn" onclick={() => { stOpen = !stOpen; tfOpen = false; }}>
                        {seriesType}
                        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" class:rotate={stOpen}>
                            <path d="M2 4l4 4 4-4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                        </svg>
                    </button>
                    {#if stOpen}
                        <div class="dd-menu glass-panel">
                            {#each SERIES_TYPES as st}
                                <button
                                    class:active={st === seriesType}
                                    onclick={() => { seriesType = st; stOpen = false; applyFilters(); }}
                                >{st}</button>
                            {/each}
                        </div>
                    {/if}
                </div>

                <button class="apply-btn" onclick={applyFilters} aria-label="Apply Filters">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="20 6 9 17 4 12"></polyline>
                    </svg>
                </button>
            </div>
        </div>

        <!-- Chart area -->
        <div class="chart-wrap">
            {#if loading}
                <div class="overlay glass-panel">
                    <div class="modern-spinner"></div>
                    <span>Rendering Data...</span>
                </div>
            {/if}

            {#if noData && !loading}
                <div class="overlay no-data glass-panel">
                    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#475569" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" style="margin-bottom: 12px;">
                        <circle cx="12" cy="12" r="10"></circle>
                        <line x1="12" y1="8" x2="12" y2="12"></line>
                        <line x1="12" y1="16" x2="12.01" y2="16"></line>
                    </svg>
                    No data found for <strong style="color: #38bdf8; margin-left: 6px;">{selectedSymbol}</strong>
                </div>
            {/if}

            <div bind:this={chartContainer} class="chart"></div>
        </div>
    </main>
</div>

<style>
    /* ── Global Theme & Vars ── */
    :global(body) {
        margin: 0;
        overflow: hidden;
        background: #06090f;
        background-image: radial-gradient(circle at top right, rgba(15, 23, 42, 1), transparent 40%);
        font-family: 'Inter', system-ui, -apple-system, sans-serif;
        color: #f8fafc;
    }

    .symbol-list button, .date-input, .symbol-count .badge {
        font-family: 'JetBrains Mono', 'Fira Code', monospace;
    }

    ::-webkit-scrollbar { width: 6px; height: 6px; }
    ::-webkit-scrollbar-track { background: transparent; }
    ::-webkit-scrollbar-thumb { background: #334155; border-radius: 10px; }
    ::-webkit-scrollbar-thumb:hover { background: #475569; }

    .layout {
        display: flex;
        width: 100vw;
        height: 100vh;
    }

    /* ── Sidebar ── */
    .sidebar {
        width: 280px;
        min-width: 260px;
        background: rgba(15, 23, 42, 0.65);
        backdrop-filter: blur(20px);
        border-right: 1px solid rgba(255, 255, 255, 0.05);
        display: flex;
        flex-direction: column;
        box-shadow: 4px 0 24px rgba(0,0,0,0.2);
        z-index: 10;
    }

    .logo {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 24px 20px;
        font-size: 18px;
        font-weight: 800;
        color: #f8fafc;
        letter-spacing: -0.5px;
    }

    .logo-icon-wrap {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 36px;
        height: 36px;
        background: linear-gradient(135deg, #38bdf8, #3b82f6);
        border-radius: 10px;
        box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
    }
    .logo-icon { font-size: 18px; }

    .search-wrap {
        position: relative;
        padding: 0 20px 16px;
    }

    .search-icon {
        position: absolute;
        left: 32px;
        top: 45%;
        transform: translateY(-50%);
        color: #64748b;
        font-size: 16px;
        pointer-events: none;
    }

    .search-wrap input {
        width: 100%;
        padding: 10px 14px 10px 38px;
        background: rgba(30, 41, 59, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        color: #f8fafc;
        font-size: 13px;
        transition: all 0.2s ease;
        box-sizing: border-box;
    }

    .search-wrap input:focus {
        border-color: #38bdf8;
        background: rgba(30, 41, 59, 0.8);
        box-shadow: 0 0 0 3px rgba(56, 189, 248, 0.15);
        outline: none;
    }

    .symbol-count {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0 20px 12px;
        font-size: 11px;
        color: #64748b;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .symbol-count .badge {
        background: rgba(30, 41, 59, 0.8);
        padding: 2px 8px;
        border-radius: 20px;
        font-size: 10px;
        color: #94a3b8;
    }

    .symbol-list {
        flex: 1;
        overflow-y: auto;
        padding: 0 12px 20px;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .symbol-list button {
        width: 100%;
        padding: 10px 16px;
        background: transparent;
        border: 1px solid transparent;
        border-radius: 10px;
        color: #cbd5e1;
        text-align: left;
        cursor: pointer;
        font-size: 13px;
        font-weight: 500;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .symbol-list button:hover {
        background: rgba(30, 41, 59, 0.4);
        color: #f8fafc;
        transform: translateX(2px);
    }

    .symbol-list button.selected {
        background: linear-gradient(90deg, rgba(56, 189, 248, 0.1), transparent);
        border: 1px solid rgba(56, 189, 248, 0.2);
        color: #38bdf8;
        font-weight: 700;
        box-shadow: inset 2px 0 0 #38bdf8;
    }

    .empty-state {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        padding: 30px 20px;
        color: #64748b;
        font-size: 13px;
        text-align: center;
    }

    /* ── Main Area ── */
    .main {
        flex: 1;
        display: flex;
        flex-direction: column;
        position: relative;
    }

    /* ── Filter Bar ── */
    .filter-bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 16px 24px;
        background: rgba(15, 23, 42, 0.4);
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        backdrop-filter: blur(10px);
    }

    .filters-left, .filters-right {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    /* Beautiful Admin Import Button */
    .action-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 7px 14px;
        background: rgba(16, 185, 129, 0.1); 
        border: 1px solid rgba(16, 185, 129, 0.3);
        border-radius: 20px;
        color: #34d399;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: 0 0 10px rgba(16, 185, 129, 0.05);
    }

    .action-btn:hover {
        background: rgba(16, 185, 129, 0.2);
        border-color: rgba(16, 185, 129, 0.5);
        color: #10b981;
        transform: translateY(-1px);
        box-shadow: 0 4px 12px rgba(16, 185, 129, 0.15);
    }

    .divider {
        height: 20px;
        width: 1px;
        background: rgba(255, 255, 255, 0.1);
        margin: 0 8px;
    }

    .date-label {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 12px;
        color: #94a3b8;
        font-weight: 500;
    }

    .date-input {
        background: rgba(30, 41, 59, 0.6);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        color: #e2e8f0;
        font-size: 12px;
        padding: 8px 12px;
        outline: none;
        cursor: pointer;
        transition: all 0.2s;
        color-scheme: dark;
    }
    .date-input:hover { background: rgba(30, 41, 59, 0.9); }
    .date-input:focus { border-color: #38bdf8; }

    /* ── Dropdowns ── */
    .dd-wrap { position: relative; }

    .dd-btn {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 16px;
        background: rgba(30, 41, 59, 0.6);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 20px;
        color: #e2e8f0;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
    }
    
    .dd-btn svg { transition: transform 0.2s; }
    .dd-btn svg.rotate { transform: rotate(180deg); }

    .dd-btn:hover {
        background: rgba(30, 41, 59, 0.9);
        border-color: rgba(255, 255, 255, 0.2);
    }

    .glass-panel {
        background: rgba(15, 23, 42, 0.85);
        backdrop-filter: blur(16px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        box-shadow: 0 10px 30px rgba(0,0,0,0.5);
    }

    .dd-menu {
        position: absolute;
        top: calc(100% + 8px);
        right: 0;
        border-radius: 12px;
        min-width: 150px;
        z-index: 200;
        overflow: hidden;
        padding: 6px;
    }

    .dd-menu button {
        display: block;
        width: 100%;
        padding: 10px 14px;
        background: transparent;
        border: none;
        border-radius: 6px;
        color: #94a3b8;
        font-size: 13px;
        font-weight: 500;
        text-align: left;
        cursor: pointer;
        transition: all 0.15s;
    }

    .dd-menu button:hover {
        background: rgba(56, 189, 248, 0.1);
        color: #f8fafc;
    }

    .dd-menu button.active {
        background: #38bdf8;
        color: #0f172a;
        font-weight: 700;
    }

    /* ── Apply button ── */
    .apply-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 36px;
        height: 36px;
        background: linear-gradient(135deg, #38bdf8, #2563eb);
        border: none;
        border-radius: 10px;
        color: #ffffff;
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
    }

    .apply-btn:hover {
        transform: translateY(-1px);
        box-shadow: 0 6px 16px rgba(59, 130, 246, 0.4);
    }
    
    .apply-btn:active { transform: translateY(1px); }

    /* ── Chart ── */
    .chart-wrap {
        flex: 1;
        position: relative;
        padding: 16px;
    }

    .chart { width: 100%; height: 100%; }

    /* ── Overlays ── */
    .overlay {
        position: absolute;
        inset: 24px;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 16px;
        border-radius: 16px;
        z-index: 50;
        color: #94a3b8;
        font-size: 15px;
        font-weight: 500;
    }

    .no-data {
        flex-direction: row;
        font-size: 16px;
        background: rgba(15, 23, 42, 0.7);
    }

    /* Smooth Modern Spinner */
    .modern-spinner {
        width: 40px;
        height: 40px;
        border: 3px solid rgba(56, 189, 248, 0.2);
        border-top-color: #38bdf8;
        border-radius: 50%;
        animation: spin 0.8s cubic-bezier(0.6, 0.2, 0.4, 0.8) infinite;
    }
    
    .mini-spinner {
        width: 16px;
        height: 16px;
        border: 2px solid rgba(56, 189, 248, 0.2);
        border-top-color: #38bdf8;
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin { to { transform: rotate(360deg); } }
</style>