<script lang="ts">

    let {

        summary = {}

    } = $props();

    function format(value: number) {

        return value.toFixed(2);
    }

</script>

<div class="matrix">

    <div class="matrix-header">

        <div>
            TF
        </div>

        <div>
            HIGH
        </div>

        <div>
            LOW
        </div>

        <div>
            CLOSE
        </div>

        <div>
            PIVOT
        </div>

    </div>

    {#each Object.entries(summary) as [key, value]}

        <div class="matrix-row">

            <div class="tf">
                {key}
            </div>

            <div class="high">
                {format(value.high)}
            </div>

            <div class="low">
                {format(value.low)}
            </div>

            <div class="close">
                {format(value.close)}
            </div>

            <div class="pivot">
                {format(value.pivot)}
            </div>

        </div>

    {/each}

</div>

<style>

.matrix {

    display: flex;

    flex-direction: column;

    background:
        linear-gradient(
            to bottom,
            #0f172a,
            #111827
        );

    border:
        1px solid #1e293b;

    border-radius: 14px;

    overflow: hidden;

    margin-bottom: 24px;

    box-shadow:
        0 8px 30px rgba(0,0,0,0.25);
}

.matrix-header {

    display: grid;

    grid-template-columns:
        1.2fr 1fr 1fr 1fr 1fr;

    background: #1e293b;

    padding: 14px 18px;

    font-size: 12px;

    font-weight: 800;

    letter-spacing: 1px;

    color: #94a3b8;
}

.matrix-row {

    display: grid;

    grid-template-columns:
        1.2fr 1fr 1fr 1fr 1fr;

    padding: 14px 18px;

    border-top:
        1px solid #1e293b;

    align-items: center;

    font-size: 14px;

    transition: 0.15s;
}

.matrix-row:hover {

    background:
        rgba(255,255,255,0.02);
}

.tf {

    font-weight: 700;

    color: white;
}

.high {

    color: #4ade80;

    font-weight: 700;
}

.low {

    color: #f87171;

    font-weight: 700;
}

.close {

    color: #e2e8f0;
}

.pivot {

    color: #60a5fa;

    font-weight: 700;
}

</style>