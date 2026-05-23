<script lang="ts">

    let {

        title = "",

        pivot = 0,

        resistanceLevels = [],

        supportLevels = []

    } = $props();

</script>

<div class="panel">

    <div class="panel-header">
        {title}
    </div>

    <div class="ladder">

        <!-- Resistance -->

        <div class="section">

            {#each [...resistanceLevels].reverse() as r}

                <div class="level resistance">

                    <span>
                        {r.label}
                    </span>

                    <span>
                        {r.value.toFixed(2)}
                    </span>

                </div>

            {/each}

        </div>

        <!-- Pivot -->

        <div class="pivot-box">

            <div class="pivot-label">
                PIVOT
            </div>

            <div class="pivot-value">
                {pivot.toFixed(2)}
            </div>

        </div>

        <!-- Support -->

        <div class="section">

            {#each supportLevels as s}

                <div class="level support">

                    <span>
                        {s.label}
                    </span>

                    <span>
                        {s.value.toFixed(2)}
                    </span>

                </div>

            {/each}

        </div>

    </div>

</div>

<style>

.panel {

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

    display: flex;

    flex-direction: column;

    gap: 18px;

    box-shadow:
        0 0 0 1px rgba(255,255,255,0.02),
        0 8px 30px rgba(0,0,0,0.25);
}

.panel-header {

    text-align: center;

    font-size: 15px;

    font-weight: 800;

    letter-spacing: 1px;

    color: white;
}

.ladder {

    display: flex;

    flex-direction: column;

    gap: 10px;
}

.section {

    display: flex;

    flex-direction: column;

    gap: 8px;
}

.level {

    display: flex;

    justify-content: space-between;

    align-items: center;

    padding: 10px 14px;

    border-radius: 10px;

    font-size: 14px;

    font-weight: 700;
}

.resistance {

    background:
        rgba(34,197,94,0.12);

    border:
        1px solid rgba(34,197,94,0.25);

    color: #4ade80;
}

.support {

    background:
        rgba(239,68,68,0.12);

    border:
        1px solid rgba(239,68,68,0.25);

    color: #f87171;
}

.pivot-box {

    background:
        rgba(59,130,246,0.12);

    border:
        1px solid rgba(59,130,246,0.3);

    border-radius: 12px;

    padding: 18px;

    display: flex;

    flex-direction: column;

    align-items: center;

    gap: 6px;
}

.pivot-label {

    font-size: 12px;

    letter-spacing: 2px;

    color: #93c5fd;
}

.pivot-value {

    font-size: 26px;

    font-weight: 800;

    color: white;
}

</style>