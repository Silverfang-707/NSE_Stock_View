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

    // =========================
    // LOAD SYMBOLS
    // =========================

    async function loadSymbols() {

        try {

            const response =
                await api.get(
                    `/symbols/${selectedSeries}`
                );

            symbols =
                response.data;

            // pick first symbol automatically
            if (
                symbols.length > 0
            ) {

                selectedSymbol =
                    symbols[0];
            }

        } catch (error) {

            console.error(
                "Failed to load symbols",
                error
            );
        }
    }

    // =========================
    // LOAD ANALYSIS
    // =========================

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

    // =========================
    // LOAD CANDLES
    // =========================

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

    // =========================
    // SEARCH ACTION
    // =========================

    async function search() {

        await loadAnalysis();

        await loadCandles();
    }

    // =========================
    // SERIES CHANGE
    // =========================

    async function refreshSeries() {

        await loadSymbols();

        await loadAnalysis();

        await loadCandles();
    }

    // =========================
    // INITIAL LOAD
    // =========================

    onMount(async () => {

        await refreshSeries();

    });

</script>

<div class="page">

    <!-- HEADER -->

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

    <!-- FILTER BAR -->

    <FilterBar

        {symbols}

        bind:selectedSymbol

        bind:selectedSeries

        onSearch={search}
    />

    <!-- REFRESH -->

    <div class="toolbar">

        <button
            class="refresh-btn"
            onclick={() => refreshSeries()}
        >
            Refresh Series
        </button>

    </div>

    <!-- LOADING -->

    {#if loading}

        <div class="loading">
            Loading market analysis...
        </div>

    {/if}

    <!-- CONTENT -->

    {#if analysis}

        <!-- SUMMARY -->

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

        <!-- CHART -->

        <MiniChart
            {candles}
        />

        <!-- ANALYSIS GRID -->

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

    display: flex;

    justify-content: space-between;

    align-items: center;

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

.toolbar {

    margin-bottom: 20px;
}

.refresh-btn {

    background: #1e293b;

    border: 1px solid #334155;

    color: white;

    padding: 10px 18px;

    border-radius: 10px;

    cursor: pointer;

    font-weight: 700;

    transition: 0.15s;
}

.refresh-btn:hover {

    background: #334155;
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