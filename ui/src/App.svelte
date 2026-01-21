<script>
  import { onMount } from "svelte";

  const apiBase = import.meta.env.VITE_API_BASE ?? "http://127.0.0.1:8088";

  let summary = null;
  let error = null;
  let loading = false;
  let updatedAt = "";

  const metricValue = (key, fallback = 0) => summary?.metrics?.[key] ?? fallback;
  const onOff = (value) => (value ? "On" : "Off");

  $: stats = [
    {
      label: "Latest Return",
      value: metricValue("merrow_last_return_rate").toFixed(4),
      meta: `Sharpe ${metricValue("merrow_last_sharpe").toFixed(3)}`
    },
    {
      label: "Drawdown",
      value: metricValue("merrow_last_max_drawdown").toFixed(4),
      meta: `Win ${metricValue("merrow_last_win_rate").toFixed(3)}`
    },
    {
      label: "Live Orders",
      value: metricValue("merrow_live_orders_sent_total").toString(),
      meta: `Retries ${metricValue("merrow_live_retry_total")}`
    },
    {
      label: "Backtests",
      value: metricValue("merrow_backtest_runs_total").toString(),
      meta: `Paper ${metricValue("merrow_paper_runs_total")}`
    },
    {
      label: "Trades",
      value: metricValue("merrow_trades_total").toString(),
      meta: `Signals ${metricValue("merrow_signals_total")}`
    },
    {
      label: "Errors",
      value: metricValue("merrow_errors_total").toString(),
      meta: `Last ${metricValue("merrow_last_error_code")}`
    }
  ];

  async function load() {
    loading = true;
    error = null;
    try {
      const res = await fetch(`${apiBase}/api/summary`);
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
      }
      summary = await res.json();
      updatedAt = new Date().toLocaleTimeString();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to fetch data";
    } finally {
      loading = false;
    }
  }

  onMount(load);
</script>

<div class="relative min-h-screen overflow-hidden">
  <div class="pointer-events-none absolute inset-0 bg-grid opacity-30"></div>
  <div class="pointer-events-none absolute -top-40 left-1/2 h-80 w-80 -translate-x-1/2 rounded-full bg-accent/20 blur-3xl"></div>
  <div class="pointer-events-none absolute -bottom-32 right-0 h-96 w-96 rounded-full bg-accent2/20 blur-3xl"></div>

  <div class="mx-auto max-w-7xl px-6 py-10 lg:px-12">
    <header class="grid gap-6 lg:grid-cols-[1.2fr_1fr] lg:items-center animate-rise">
      <div>
        <p class="panel-title">Merrow System</p>
        <h1 class="mt-2 text-3xl font-semibold md:text-4xl">Control Room</h1>
        <p class="mt-3 max-w-xl text-sm text-muted md:text-base">
          Monitor strategy health, triggers, and execution flow. Refresh to sync with the live UI API.
        </p>
        <div class="mt-4 flex flex-wrap gap-2 text-xs text-muted">
          <span class="rounded-full border border-line bg-panel px-3 py-1">API: {apiBase}</span>
          <span class="rounded-full border border-line bg-panel px-3 py-1">Updated: {updatedAt || "—"}</span>
        </div>
      </div>
      <div class="panel">
        <div class="flex items-center justify-between">
          <p class="panel-title">Runtime</p>
          <button
            class="rounded-xl border border-line px-4 py-2 text-xs font-medium transition hover:-translate-y-0.5 hover:border-accent disabled:cursor-not-allowed disabled:opacity-60"
            on:click={load}
            disabled={loading}
          >
            {loading ? "Refreshing..." : "Refresh"}
          </button>
        </div>
        <div class="mt-4 grid grid-cols-2 gap-3 text-sm">
          <div>
            <div class="text-muted">Mode</div>
            <div class="text-lg font-semibold">{summary?.config.mode ?? "—"}</div>
          </div>
          <div>
            <div class="text-muted">Exchange</div>
            <div class="text-lg font-semibold">{summary?.config.exchange ?? "—"}</div>
          </div>
          <div>
            <div class="text-muted">Symbol</div>
            <div class="text-lg font-semibold">{summary?.config.symbol ?? "—"}</div>
          </div>
          <div>
            <div class="text-muted">Interval</div>
            <div class="text-lg font-semibold">{summary?.config.data.candle_interval ?? "—"}</div>
          </div>
        </div>
      </div>
    </header>

    {#if error}
      <div class="panel mt-6">
        <strong>Connection error:</strong> {error}
      </div>
    {:else}
      <section class="mt-10">
        <div class="flex items-center justify-between">
          <h2 class="text-xl">Key Metrics</h2>
          <span class="text-xs text-muted">Rolling metrics from last run</span>
        </div>
        <div class="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {#each stats as item}
            <div class="stat-card animate-rise">
              <div class="stat-label">{item.label}</div>
              <div class="stat-value">{item.value}</div>
              <div class="stat-meta">{item.meta}</div>
            </div>
          {/each}
        </div>
      </section>

      <section class="mt-10 grid gap-4 lg:grid-cols-[2fr_1fr]">
        <div class="panel animate-rise">
          <div class="flex items-center justify-between">
            <h2 class="text-xl">Strategy</h2>
            <span class="text-xs text-muted">Triggers & ratios</span>
          </div>
          <div class="mt-3 flex flex-wrap gap-2 text-xs text-muted">
            <span class="rounded-full border border-line bg-panel/60 px-3 py-1">
              Trigger: {summary?.config.triggers.trigger_mode_any ? "Any" : "All"}
            </span>
            <span class="rounded-full border border-line bg-panel/60 px-3 py-1">
              Time {onOff(summary?.config.triggers.time_enabled)}
            </span>
            <span class="rounded-full border border-line bg-panel/60 px-3 py-1">
              Price {onOff(summary?.config.triggers.price_enabled)}
            </span>
          </div>
          <div class="mt-5 grid gap-3 sm:grid-cols-3">
            <div class="stat-card">
              <div class="stat-label">Buy Ratio</div>
              <div class="stat-value">{summary?.config.strategy.buy_cash_ratio ?? 0}</div>
              <div class="stat-meta">Cash allocation</div>
            </div>
            <div class="stat-card">
              <div class="stat-label">Sell Ratio</div>
              <div class="stat-value">{summary?.config.strategy.sell_pos_ratio ?? 0}</div>
              <div class="stat-meta">Position trim</div>
            </div>
            <div class="stat-card">
              <div class="stat-label">Rebuy Ratio</div>
              <div class="stat-value">{summary?.config.strategy.rebuy_cash_ratio ?? 0}</div>
              <div class="stat-meta">Sell proceeds</div>
            </div>
          </div>
        </div>
        <div class="panel animate-rise">
          <h2 class="text-xl">Storage</h2>
          <p class="mt-2 text-sm text-muted">PGSQL: {summary?.pg_enabled ? "Enabled" : "Disabled"}</p>
          {#if summary?.backtest}
            <div class="mt-4 rounded-xl border border-line bg-panel/60 px-4 py-3 text-sm">
              <div class="text-xs uppercase tracking-[0.25em] text-accent2">Latest Backtest</div>
              <div class="mt-2 font-semibold">Run {summary.backtest.run_id}</div>
              <div class="mt-1 text-xs text-muted">
                {summary.backtest.start_time} → {summary.backtest.end_time}
              </div>
              <div class="mt-2 text-xs text-muted">
                Return {summary.backtest.return_rate.toFixed(4)} · Trades {summary.backtest.trade_count}
              </div>
            </div>
          {:else}
            <p class="mt-4 text-sm text-muted">No backtest data available.</p>
          {/if}
        </div>
      </section>

      <section class="mt-10">
        <h2 class="text-xl">Recent Trades</h2>
        <div class="panel mt-3 animate-rise">
          <div class="table-wrap">
            <table class="table-base">
              <thead>
                <tr>
                  <th class="table-head">Time</th>
                  <th class="table-head">Symbol</th>
                  <th class="table-head">Side</th>
                  <th class="table-head">Price</th>
                  <th class="table-head">Qty</th>
                  <th class="table-head">Fee</th>
                </tr>
              </thead>
              <tbody>
                {#if (summary?.trades ?? []).length === 0}
                  <tr>
                    <td class="table-cell text-muted" colspan="6">No trades yet.</td>
                  </tr>
                {:else}
                  {#each summary?.trades ?? [] as trade}
                    <tr>
                      <td class="table-cell">{trade.time}</td>
                      <td class="table-cell">{trade.symbol}</td>
                      <td class="table-cell">{trade.side}</td>
                      <td class="table-cell">{trade.price.toFixed(4)}</td>
                      <td class="table-cell">{trade.qty.toFixed(6)}</td>
                      <td class="table-cell">{trade.fee.toFixed(6)}</td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <section class="mt-10 grid gap-4 lg:grid-cols-2">
        <div class="panel animate-rise">
          <h2 class="text-xl">Orders</h2>
          <div class="table-wrap mt-3">
            <table class="table-base">
              <thead>
                <tr>
                  <th class="table-head">Time</th>
                  <th class="table-head">Symbol</th>
                  <th class="table-head">Side</th>
                  <th class="table-head">Type</th>
                  <th class="table-head">Qty</th>
                  <th class="table-head">Status</th>
                </tr>
              </thead>
              <tbody>
                {#if (summary?.orders ?? []).length === 0}
                  <tr>
                    <td class="table-cell text-muted" colspan="6">No orders yet.</td>
                  </tr>
                {:else}
                  {#each summary?.orders ?? [] as order}
                    <tr>
                      <td class="table-cell">{order.time}</td>
                      <td class="table-cell">{order.symbol}</td>
                      <td class="table-cell">{order.side}</td>
                      <td class="table-cell">{order.order_type}</td>
                      <td class="table-cell">{order.qty.toFixed(6)}</td>
                      <td class="table-cell">{order.status}</td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>
        </div>
        <div class="panel animate-rise">
          <h2 class="text-xl">Positions</h2>
          <div class="table-wrap mt-3">
            <table class="table-base">
              <thead>
                <tr>
                  <th class="table-head">Symbol</th>
                  <th class="table-head">Qty</th>
                  <th class="table-head">Avg</th>
                </tr>
              </thead>
              <tbody>
                {#if (summary?.positions ?? []).length === 0}
                  <tr>
                    <td class="table-cell text-muted" colspan="3">No positions yet.</td>
                  </tr>
                {:else}
                  {#each summary?.positions ?? [] as pos}
                    <tr>
                      <td class="table-cell">{pos.symbol}</td>
                      <td class="table-cell">{pos.qty.toFixed(6)}</td>
                      <td class="table-cell">{pos.avg_price.toFixed(4)}</td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <section class="mt-10">
        <h2 class="text-xl">Balances</h2>
        <div class="panel mt-3 animate-rise">
          <div class="table-wrap">
            <table class="table-base">
              <thead>
                <tr>
                  <th class="table-head">Asset</th>
                  <th class="table-head">Free</th>
                  <th class="table-head">Locked</th>
                </tr>
              </thead>
              <tbody>
                {#if (summary?.balances ?? []).length === 0}
                  <tr>
                    <td class="table-cell text-muted" colspan="3">No balances yet.</td>
                  </tr>
                {:else}
                  {#each summary?.balances ?? [] as bal}
                    <tr>
                      <td class="table-cell">{bal.asset}</td>
                      <td class="table-cell">{bal.free.toFixed(4)}</td>
                      <td class="table-cell">{bal.locked.toFixed(4)}</td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>
        </div>
      </section>
    {/if}
  </div>
</div>
