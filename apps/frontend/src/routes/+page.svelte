<script lang="ts">

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

    // =====================================
    // STATE
    // =====================================

    let symbols =
        $state<string[]>([]);

    let selectedSymbol =
        $state("");

    let selectedSeries =
        $state("EQ");

    let analysis =
        $state<any>(null);

    let candles =
        $state<any[]>([]);

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

    // =====================================
    // LOAD SYMBOLS
    // =====================================

    async function loadSymbols() {

        try {

            const response =
                await api.get(
                    `/symbols/${selectedSeries}`
                );

            const newSymbols =
                response.data;

            symbols = newSymbols;

            // preserve symbol if valid
            if (

                !newSymbols.includes(
                    selectedSymbol
                )

            ) {

                if (
                    newSymbols.length > 0
                ) {

                    selectedSymbol =
                        newSymbols[0];
                }
            }

        } catch (error) {

            console.error(
                "Failed to load symbols",
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

            loading = true;

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

        } finally {

            loading = false;
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
                response.data.slice(-30);

        } catch (error) {

            console.error(
                "Failed to load candles",
                error
            );
        }
    }

    // =====================================
    // SEARCH / ANALYZE
    // =====================================

    async function search() {

        await loadAnalysis();

        await loadCandles();
    }

    // =====================================
    // SERIES CHANGE
    // =====================================

    async function changeSeries() {

        await loadSymbols();
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

            // fake smooth progress
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

        await loadSymbols();

        await loadAnalysis();

        await loadCandles();

    });

</script>

<div class="page">

    <!-- ===================================== -->
    <!-- HEADER -->
    <!-- ===================================== -->

    <div class="header">

        <div>

            <h1>
                NSE Market Terminal
            </h1>

            <p>
                Rust + TimescaleDB Analysis Engine
            </p>

        </div>

    </div>

    <!-- ===================================== -->
    <!-- FILTER BAR -->
    <!-- ===================================== -->

    <FilterBar

        {symbols}

        bind:selectedSymbol

        bind:selectedSeries

        onSearch={search}

        onSeriesChange={changeSeries}
    />

    <!-- ===================================== -->
    <!-- ADMIN BAR -->
    <!-- ===================================== -->

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

    <!-- ===================================== -->
    <!-- PROGRESS -->
    <!-- ===================================== -->

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

    <!-- ===================================== -->
    <!-- LOADING -->
    <!-- ===================================== -->

    {#if loading}

        <div class="loading">
            Loading market analysis...
        </div>

    {/if}

    <!-- ===================================== -->
    <!-- CONTENT -->
    <!-- ===================================== -->

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
        1.2fr 1fr;

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

</style>