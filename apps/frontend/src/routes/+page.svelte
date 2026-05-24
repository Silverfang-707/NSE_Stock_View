<script lang="ts">

    import type {

        MarketLevel

    } from "$lib/types/market-level";
    import { onMount } from "svelte";

    import api from "$lib/api";

    import FilterBar
        from "$lib/components/FilterBar.svelte";

    import SummaryTable
        from "$lib/components/SummaryTable.svelte";

    import AnalysisPanel
        from "$lib/components/AnalysisPanel.svelte";

    import MiniChart
        from "$lib/components/MiniChart.svelte";

    import { goto } from "$app/navigation";
    // =====================================
    // STATE
    // =====================================

    let symbols =
        $state<string[]>([]);

    let series =
        $state<string[]>([]);

    let role =
        $state("");

    let selectedSymbol =
        $state("");

    let selectedSeries =
        $state("");

    let analysis =
        $state<any>(null);

    let candles =
        $state<any[]>([]);

    let levels =
        $state<MarketLevel[]>([]);

    let loading =
        $state(false);

    let ingesting =
        $state(false);

    let ingestProgress =
        $state(0);

    let yearsToIngest =
        $state(1);

    let ingestMessage =
        $state("");

    let groupedLevels =

        $derived.by(() => {

            const grouped:
                Record<string, MarketLevel> = {};

            for (const level of levels) {

                if (
                    !grouped[
                        level.timeframe
                    ]
                ) {

                    grouped[
                        level.timeframe
                    ] = level;
                }
            }

            return grouped;
        });
    // =====================================
    // LOAD SYMBOLS
    // =====================================

    async function loadSymbols() {

        try {

            const response =
                await api.get(
                    `/symbols`
                );

            const newSymbols =
                response.data;

            symbols = newSymbols;

            if (

                !selectedSymbol &&

                newSymbols.length > 0

            ) {

                selectedSymbol =
                    newSymbols[0];

                await loadSeries();
            }

        } catch (error) {

            console.error(
                "Failed to load symbols",
                error
            );
        }
    }

    // =====================================
    // LOAD SERIES
    // =====================================

    async function loadSeries() {

        if (!selectedSymbol) {

            series = [];

            return;
        }

        try {

            const response =
                await api.get(
                    `/series/${selectedSymbol}`
                );

            const newSeries =
                response.data;

            series = newSeries;

            if (

                !newSeries.includes(
                    selectedSeries
                )

            ) {

                if (
                    newSeries.length > 0
                ) {

                    selectedSeries =
                        newSeries[0];
                }
            }

        } catch (error) {

            console.error(
                "Failed to load series",
                error
            );
        }
    }

    // =====================================
    // LOAD ANALYSIS
    // =====================================

    async function loadAnalysis() {

        if (!selectedSymbol) {

            return;
        }

        try {

            const response =
                await api.get(
                    `/analysis/${selectedSymbol}`
                );

            analysis =
                response.data;

        } catch (error) {

            console.error(
                "Failed to load analysis",
                error
            );
        }
    }

    // =====================================
    // LOAD CANDLES
    // =====================================

    async function loadCandles() {

        if (!selectedSymbol) {

            return;
        }

        try {

            const response =
                await api.get(
                    `/candles/${selectedSymbol}`
                );

            candles =

                response.data

                    .map(
                        (candle: any) => ({

                            time:

                                (
                                    candle.trade_date ??

                                    candle.tradeDate
                                )

                                ?.split("T")[0],

                            open:

                                candle.open_price ??

                                candle.openPrice,

                            high:

                                candle.high_price ??

                                candle.highPrice,

                            low:

                                candle.low_price ??

                                candle.lowPrice,

                            close:

                                candle.close_price ??

                                candle.closePrice
                        })
                    )

                    .filter(
                        (c: any) =>

                            c.time &&

                            c.open != null &&

                            c.high != null &&

                            c.low != null &&

                            c.close != null
                    )

                    .slice(-30);

        } catch (error) {

            console.error(
                "Failed to load candles",
                error
            );
        }
    }

    // =====================================
    // LOAD LEVELS
    // =====================================

    async function loadLevels() {

        if (!selectedSymbol) {

            return;
        }

        try {

            const response =

                await api.get(

                    `/levels/${selectedSymbol}`
                );

            levels =
                response.data;

            console.log(
                "📊 Levels:",
                levels
            );

        } catch (error) {

            console.error(

                "Failed to load levels",

                error
            );
        }
    }

    // =====================================
    // SEARCH
    // =====================================

    async function search() {

        loading = true;

        try {

            await Promise.all([

                loadAnalysis(),

                loadCandles(),

                loadLevels()

            ]);

        } finally {

            loading = false;
        }
    }

    // =====================================
    // CHANGE HANDLERS
    // =====================================

    async function changeSymbol() {

        await loadSeries();

        await search();
    }

    async function changeSeries() {

        await search();
    }

    // =====================================
    // INGEST DATA
    // =====================================

    async function ingestData() {

        try {

            ingesting = true;

            ingestProgress = 0;

            ingestMessage =
                "Starting ingestion...";

            const interval =
                setInterval(() => {

                    if (
                        ingestProgress < 90
                    ) {

                        ingestProgress += 5;
                    }

                }, 300);

            const currentYear =
                new Date().getFullYear();

            const fromYear =
                currentYear -
                yearsToIngest;

            const response =
                await api.get(

                    `/admin/backfill?from=${fromYear}-01-01&to=${currentYear}-12-31`

                );

            clearInterval(interval);

            ingestProgress = 100;

            ingestMessage =
                `Imported ${response.data.rows_inserted} rows`;

            setTimeout(() => {

                ingestProgress = 0;

            }, 2000);

        } catch (error) {

            console.error(error);

            ingestMessage =
                "Ingestion failed";

        } finally {

            ingesting = false;
        }
    }

    // =====================================
    // INITIAL LOAD
    // =====================================

    onMount(async () => {

    const token =

        localStorage.getItem(
            "token"
        );

        if (!token) {

            goto("/login");

            return;
        }

        role =
            localStorage.getItem(
                "role"
            ) ?? "";

        await loadSymbols();

        if (selectedSymbol) {

            await search();
        }
    });

</script>

<div class="page">

    <div class="header">

        <div>

            <h1>
                Airaa Stock Handler
            </h1>

            <p>
                NSE Market Terminal
            </p>

            <div class="nav-actions">

                <button
                    class="nav-btn"

                    onclick={() => goto("/")}
                >

                    Dashboard

                </button>

                {#if role === "admin"}

                    <button
                        class="nav-btn"

                        onclick={() => goto("/admin")}
                    >

                        Admin

                    </button>

                {/if}

                <button

                    class="logout-btn"

                    onclick={() => {

                        localStorage.removeItem(
                            "token"
                        );

                        localStorage.removeItem(
                            "role"
                        );

                        localStorage.removeItem(
                            "is_root"
                        );

                        goto("/login");
                    }}
                >

                    Logout

                </button>

            </div>

        </div>

    </div>

    <FilterBar

        {symbols}

        {series}

        bind:selectedSymbol

        bind:selectedSeries

        onSymbolChange={changeSymbol}

        onSeriesChange={changeSeries}
    />

    <div class="admin-bar">

        <div class="admin-left">

            <div class="input-group">

                <label for="yearsToIngest">
                    Years To Ingest
                </label>

                <select
                    id="yearsToIngest"
                    bind:value={yearsToIngest}
                >

                    <option value={1}>
                        1 Year
                    </option>

                    <option value={2}>
                        2 Years
                    </option>

                    <option value={3}>
                        3 Years
                    </option>

                    <option value={5}>
                        5 Years
                    </option>

                    <option value={10}>
                        10 Years
                    </option>

                </select>

            </div>

            <button

                class="ingest-btn"

                disabled={ingesting}

                onclick={() => ingestData()}
            >

                {#if ingesting}

                    Ingesting...

                {:else}

                    Ingest Market Data

                {/if}

            </button>

        </div>

        <div class="admin-right">

            {#if ingestMessage}

                <div class="message">
                    {ingestMessage}
                </div>

            {/if}

        </div>

    </div>

    {#if ingesting || ingestProgress > 0}

        <div class="progress-wrapper">

            <div
                class="progress-bar"
                style="
                    width:
                    {ingestProgress}%;
                "
            ></div>

        </div>

    {/if}

    {#if loading}

        <div class="loading">
            Loading market analysis...
        </div>

    {/if}

    {#if analysis}

        <div class="top-grid">

            <SummaryTable

                summary={{

                    Daily:
                        analysis.daily,

                    Weekly:
                        analysis.weekly,

                    Monthly:
                        analysis.monthly,

                    Quarterly:
                        analysis.quarterly,

                    HalfYear:
                        analysis.half_yearly,

                    Yearly:
                        analysis.yearly

                }}
            />

            <MiniChart
                {candles}
            />

        </div>

        {#if Object.keys(groupedLevels).length > 0}

    <div class="levels-panel">

        <h2>
            Calculated Levels
        </h2>

        <div class="levels-grid">

            {#each Object.entries(groupedLevels) as [timeframe, level]}

                    <div class="level-card">

                        <div class="level-top">

                            <h3>
                                {timeframe}
                            </h3>

                            <span>
                                {level.trade_date}
                            </span>

                        </div>

                        <div class="metrics">

                            <div>

                                <span>
                                    JGD
                                </span>

                                <strong>
                                    {level.jgd?.toFixed(2)}
                                </strong>

                            </div>

                            <div>

                                <span class="metric-label">
                                    JWD
                                </span>

                                <strong>
                                    {level.jwd?.toFixed(2)}
                                </strong>

                            </div>

                            <div>

                                <span class="metric-label">
                                    BDP
                                </span>

                                <strong>
                                    {level.bdp?.toFixed(2)}
                                </strong>

                            </div>

                            <div>

                                <span class="metric-label">
                                    WDP
                                </span>

                                <strong>
                                    {level.wdp?.toFixed(2)}
                                </strong>

                            </div>

                            <div>

                                <span class="metric-label">
                                    Range
                                </span>

                                <strong>
                                    {level.range_value?.toFixed(2)}
                                </strong>

                            </div>

                            <div>

                                <span class="metric-label">
                                    Buffer
                                </span>

                                <strong>
                                    {level.buffer_value?.toFixed(2)}
                                </strong>

                            </div>

                        </div>

                        <div class="pattern-tag">

                            Pattern:
                            {level.pattern || "INITIAL"}

                        </div>

                    </div>

                {/each}

            </div>

        </div>

    {/if}

        <div class="analysis-grid">

            <AnalysisPanel

                title="Daily Plan"

                pivot={
                    analysis.daily.pivot
                }

                resistanceLevels={
                    analysis.daily
                        .resistance_levels
                }

                supportLevels={
                    analysis.daily
                        .support_levels
                }
            />

            <AnalysisPanel

                title="Weekly Plan"

                pivot={
                    analysis.weekly.pivot
                }

                resistanceLevels={
                    analysis.weekly
                        .resistance_levels
                }

                supportLevels={
                    analysis.weekly
                        .support_levels
                }
            />

            <AnalysisPanel

                title="Monthly Plan"

                pivot={
                    analysis.monthly.pivot
                }

                resistanceLevels={
                    analysis.monthly
                        .resistance_levels
                }

                supportLevels={
                    analysis.monthly
                        .support_levels
                }
            />

            <AnalysisPanel

                title="Quarterly Plan"

                pivot={
                    analysis.quarterly.pivot
                }

                resistanceLevels={
                    analysis.quarterly
                        .resistance_levels
                }

                supportLevels={
                    analysis.quarterly
                        .support_levels
                }
            />

            <AnalysisPanel

                title="Half Yearly Plan"

                pivot={
                    analysis.half_yearly.pivot
                }

                resistanceLevels={
                    analysis.half_yearly
                        .resistance_levels
                }

                supportLevels={
                    analysis.half_yearly
                        .support_levels
                }
            />

            <AnalysisPanel

                title="Yearly Plan"

                pivot={
                    analysis.yearly.pivot
                }

                resistanceLevels={
                    analysis.yearly
                        .resistance_levels
                }

                supportLevels={
                    analysis.yearly
                        .support_levels
                }
            />

        </div>

    {/if}

</div>

<style>

:global(body) {

    margin: 0;

    background:
        linear-gradient(
            to bottom,
            #020617,
            #0f172a
        );

    color: white;

    font-family:
        Inter,
        sans-serif;
}

.page {

    padding: 24px;

    min-height: 100vh;
}

.header {

    margin-bottom: 24px;
}

h1 {

    margin: 0;

    font-size: 34px;

    font-weight: 800;
}

p {

    margin-top: 8px;

    color: #94a3b8;
}

.admin-bar {

    display: flex;

    justify-content: space-between;

    align-items: center;

    margin-bottom: 18px;

    background:
        linear-gradient(
            to bottom,
            #0f172a,
            #111827
        );

    border:
        1px solid #1e293b;

    border-radius: 14px;

    padding: 18px;
}

.admin-left {

    display: flex;

    gap: 18px;

    align-items: flex-end;
}

.input-group {

    display: flex;

    flex-direction: column;

    gap: 8px;
}

label {

    font-size: 12px;

    color: #94a3b8;

    font-weight: 700;

    letter-spacing: 1px;
}

select {

    background: #020617;

    border: 1px solid #334155;

    color: white;

    padding: 12px;

    border-radius: 10px;

    min-width: 180px;
}

.ingest-btn {

    background: #2563eb;

    border: none;

    color: white;

    padding: 12px 22px;

    border-radius: 10px;

    cursor: pointer;

    font-weight: 700;

    transition: 0.15s;
}

.ingest-btn:hover {

    background: #1d4ed8;
}

.ingest-btn:disabled {

    opacity: 0.6;

    cursor: not-allowed;
}

.progress-wrapper {

    width: 100%;

    height: 10px;

    background: #111827;

    border-radius: 999px;

    overflow: hidden;

    margin-bottom: 24px;

    border:
        1px solid #1e293b;
}

.progress-bar {

    height: 100%;

    background:
        linear-gradient(
            to right,
            #2563eb,
            #22c55e
        );

    transition:
        width 0.3s ease;
}

.message {

    color: #22c55e;

    font-weight: 700;
}

.loading {

    background:
        rgba(255,255,255,0.04);

    border:
        1px solid rgba(255,255,255,0.06);

    padding: 18px;

    border-radius: 12px;

    margin-bottom: 20px;
}

.top-grid {

    display: grid;

    grid-template-columns:
        1fr 1.3fr;

    gap: 20px;

    margin-bottom: 24px;
}

.analysis-grid {

    display: grid;

    grid-template-columns:
        repeat(
            auto-fit,
            minmax(320px, 1fr)
        );

    gap: 20px;
}

.levels-panel {

    margin-bottom: 24px;

    background:
        linear-gradient(
            to bottom,
            #0f172a,
            #111827
        );

    border:
        1px solid #1e293b;

    border-radius: 14px;

    padding: 20px;
}

.levels-panel h2 {

    margin: 0;

    margin-bottom: 18px;

    font-size: 22px;

    font-weight: 800;
}

.levels-grid {

    display: grid;

    grid-template-columns:
        repeat(
            auto-fit,
            minmax(180px, 1fr)
        );

    gap: 16px;
}

.level-card {

    background: #020617;

    border:
        1px solid #334155;

    border-radius: 12px;

    padding: 18px;

    display: flex;

    flex-direction: column;

    gap: 10px;

    transition: 0.15s;
}

.level-card:hover {

    border-color: #2563eb;

    transform:
        translateY(-2px);
}

.level-card span {

    color: #94a3b8;

    font-size: 12px;

    font-weight: 700;

    letter-spacing: 1px;

    text-transform: uppercase;
}

.level-card strong {

    color: white;

    font-size: 24px;

    font-weight: 800;
}

.pattern strong {

    color: #22c55e;
}
.nav-actions {

    display: flex;

    gap: 12px;

    align-items: center;
}

.nav-btn {

    background: #1e293b;

    border:
        1px solid #334155;

    color: white;

    padding: 10px 18px;

    border-radius: 10px;

    cursor: pointer;

    font-weight: 700;

    transition: 0.15s;
}

.nav-btn:hover {

    background: #334155;
}

.logout-btn {

    background: #dc2626;

    border: none;

    color: white;

    padding: 10px 18px;

    border-radius: 10px;

    cursor: pointer;

    font-weight: 700;
}

.logout-btn:hover {

    background: #b91c1c;
}

.level-top {

    display: flex;

    justify-content: space-between;

    align-items: center;

    margin-bottom: 18px;
}

.level-top h3 {

    margin: 0;

    color: white;

    text-transform: capitalize;

    font-size: 18px;
}

.level-top span {

    color: #94a3b8;

    font-size: 12px;
}

.metrics {

    display: grid;

    grid-template-columns:
        repeat(2, 1fr);

    gap: 14px;
}

.metrics div {

    display: flex;

    flex-direction: column;

    gap: 6px;
}

.metrics label {

    color: #94a3b8;

    font-size: 11px;

    letter-spacing: 1px;
}

.metrics strong {

    color: white;

    font-size: 18px;
}

.pattern-tag {

    margin-top: 18px;

    padding-top: 12px;

    border-top:
        1px solid #1e293b;

    color: #22c55e;

    font-weight: 700;
}
</style>