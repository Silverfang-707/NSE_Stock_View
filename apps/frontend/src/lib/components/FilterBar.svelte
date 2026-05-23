<script lang="ts">

    let {

        symbols = [],

        series = [],

        selectedSymbol = $bindable(""),

        selectedSeries = $bindable(""),

        onSearch = (() => {}) as () => void,

        onSymbolChange =
            (() => {}) as () => void,

        onSeriesChange =
            (() => {}) as () => void,

    } = $props();

</script>

<div class="filter-bar">

    <!-- SYMBOL -->

    <div class="field">

        <label>
            Symbol
        </label>

        <select

            bind:value={selectedSymbol}

            onchange={() => onSymbolChange()}

        >

            {#each symbols as symbol}

                <option value={symbol}>
                    {symbol}
                </option>

            {/each}

        </select>

    </div>

    <!-- SERIES -->

    <div class="field">

        <label>
            Series
        </label>

        <select

            bind:value={selectedSeries}

            onchange={() => onSeriesChange()}

        >

            {#each series as s}

                <option value={s}>
                    {s}
                </option>

            {/each}

        </select>

    </div>

    <!-- SEARCH -->

    <button
        onclick={() => onSearch()}
    >

        Analyze

    </button>

</div>

<style>

.filter-bar {

    display: flex;

    gap: 18px;

    align-items: flex-end;

    margin-bottom: 24px;

    background:
        linear-gradient(
            to bottom,
            #0f172a,
            #111827
        );

    border:
        1px solid #1e293b;

    padding: 18px;

    border-radius: 14px;
}

.field {

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

    min-width: 220px;

    font-size: 14px;
}

button {

    background: #2563eb;

    border: none;

    color: white;

    padding: 12px 22px;

    border-radius: 10px;

    cursor: pointer;

    font-weight: 700;

    transition: 0.15s;
}

button:hover {

    background: #1d4ed8;
}

</style>