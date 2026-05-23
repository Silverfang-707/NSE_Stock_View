<script lang="ts">

    import { onMount } from "svelte";

    import * as echarts from "echarts";

    let {

        candles = []

    } = $props();

    let chartElement:
        HTMLDivElement;

    let chart: any;

    // =====================================
    // UPDATE CHART
    // =====================================

    function updateChart() {

        if (
            !chart ||
            candles.length === 0
        ) {

            return;
        }

        const option = {

            backgroundColor:
                "transparent",

            tooltip: {

                trigger: "axis"
            },

            grid: {

                left: 10,

                right: 10,

                top: 20,

                bottom: 20
            },

            xAxis: {

                type: "category",

                data:
                    candles.map(
                        (c: any) =>
                            c.time
                    ),

                axisLine: {

                    lineStyle: {

                        color: "#475569"
                    }
                },

                axisLabel: {

                    color: "#94a3b8"
                }
            },

            yAxis: {

                scale: true,

                axisLine: {

                    lineStyle: {

                        color: "#475569"
                    }
                },

                splitLine: {

                    lineStyle: {

                        color:
                            "rgba(255,255,255,0.04)"
                    }
                },

                axisLabel: {

                    color: "#94a3b8"
                }
            },

            series: [

                {

                    type: "candlestick",

                    data:
                        candles.map(
                            (c: any) => [

                                c.open,

                                c.close,

                                c.low,

                                c.high
                            ]
                        ),

                    itemStyle: {

                        color: "#22c55e",

                        color0: "#ef4444",

                        borderColor:
                            "#22c55e",

                        borderColor0:
                            "#ef4444"
                    }
                }
            ]
        };

        chart.setOption(option);
    }

    // =====================================
    // INIT CHART
    // =====================================

    onMount(() => {

        chart =
            echarts.init(
                chartElement
            );

        updateChart();

        const resizeHandler =
            () => chart.resize();

        window.addEventListener(
            "resize",
            resizeHandler
        );

        return () => {

            window.removeEventListener(
                "resize",
                resizeHandler
            );

            chart.dispose();
        };
    });

    // =====================================
    // REACTIVE UPDATE
    // =====================================

    $effect(() => {

        updateChart();
    });

</script>

<div
    bind:this={chartElement}
    class="chart"
/>

<style>

.chart {

    width: 100%;

    height: 320px;

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
}

</style>