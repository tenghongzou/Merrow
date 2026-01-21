<script lang="ts">
  import { onMount } from "svelte";

  type Summary = {
    config: {
      mode: string;
      exchange: string;
      symbol: string;
      data: { source: string; candle_interval: string; exchange_category?: string | null };
      orders: { order_type: string; fee_rate: number; slippage_bps: number };
      strategy: { buy_cash_ratio: number; sell_pos_ratio: number; rebuy_cash_ratio: number };
      risk: { max_trade_ratio: number; min_cash_reserve_ratio: number; max_position_value_ratio: number };
      triggers: {
        time_enabled: boolean;
        time_minutes: number;
        price_enabled: boolean;
        trigger_mode_any: boolean;
        ma_window: number;
        buy_threshold: number;
        sell_threshold: number;
      };
    };
    pg_enabled: boolean;
    metrics: Record<string, number>;
    backtest?: {
      run_id: string;
      start_time: string;
      end_time: string;
      return_rate: number;
      max_drawdown: number;
      win_rate: number;
      trade_count: number;
      sharpe?: number | null;
    } | null;
    trades: Array<{ time: string; symbol: string; side: string; price: number; qty: number; fee: number }>;
    orders: Array<{
      time: string;
      symbol: string;
      side: string;
      order_type: string;
      price?: number | null;
      qty: number;
      status: string;
    }>;
    positions: Array<{ time: string; symbol: string; qty: number; avg_price: number }>;
    balances: Array<{ time: string; asset: string; free: number; locked: number }>;
  };

  const apiBase = import.meta.env.VITE_API_BASE ?? "http://127.0.0.1:8088";

  let summary: Summary | null = null;
  let error: string | null = null;
  let loading = false;
  let updatedAt = "";

  const metricValue = (key: string, fallback = 0) => summary?.metrics?.[key] ?? fallback;

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

<div class="wrap">
  <header class="header">
    <div>
      <h1 class="title">Merrow Control Deck</h1>
      <div class="subtitle">Local dashboard for strategy, metrics, and execution flow.</div>
    </div>
    <div class="pills">
      <span class="pill">API: {apiBase}</span>
      <span class="pill">Updated: {updatedAt || "—"}</span>
      <button class="refresh" on:click={load} disabled={loading}>
        {loading ? "Refreshing..." : "Refresh"}
      </button>
    </div>
  </header>

  {#if error}
    <div class="panel">
      <strong>Connection error:</strong> {error}
    </div>
  {:else}
    <section class="grid">
      <div class="card">
        <h3>Mode</h3>
        <div class="value">{summary?.config.mode ?? "—"}</div>
        <div class="muted">Exchange: {summary?.config.exchange ?? "—"}</div>
      </div>
      <div class="card">
        <h3>Symbol</h3>
        <div class="value">{summary?.config.symbol ?? "—"}</div>
        <div class="muted">Interval: {summary?.config.data.candle_interval ?? "—"}</div>
      </div>
      <div class="card">
        <h3>Latest Return</h3>
        <div class="value">{metricValue("merrow_last_return_rate").toFixed(4)}</div>
        <div class="muted">Sharpe: {metricValue("merrow_last_sharpe").toFixed(3)}</div>
      </div>
      <div class="card">
        <h3>Drawdown</h3>
        <div class="value">{metricValue("merrow_last_max_drawdown").toFixed(4)}</div>
        <div class="muted">Win rate: {metricValue("merrow_last_win_rate").toFixed(3)}</div>
      </div>
      <div class="card">
        <h3>Live Orders Sent</h3>
        <div class="value">{metricValue("merrow_live_orders_sent_total")}</div>
        <div class="muted">Retries: {metricValue("merrow_live_retry_total")}</div>
      </div>
      <div class="card">
        <h3>Runs</h3>
        <div class="value">{metricValue("merrow_backtest_runs_total")}</div>
        <div class="muted">Paper: {metricValue("merrow_paper_runs_total")}</div>
      </div>
    </section>

    <section class="section layout-two">
      <div class="panel">
        <h2>Strategy Snapshot</h2>
        <p class="muted">
          Trigger: {summary?.config.triggers.trigger_mode_any ? "Any" : "All"} ·
          Time: {summary?.config.triggers.time_enabled ? "On" : "Off"} ·
          Price: {summary?.config.triggers.price_enabled ? "On" : "Off"}
        </p>
        <div class="grid" style="margin-top: 12px;">
          <div class="card">
            <h3>Buy Ratio</h3>
            <div class="value">{summary?.config.strategy.buy_cash_ratio ?? 0}</div>
          </div>
          <div class="card">
            <h3>Sell Ratio</h3>
            <div class="value">{summary?.config.strategy.sell_pos_ratio ?? 0}</div>
          </div>
          <div class="card">
            <h3>Rebuy Ratio</h3>
            <div class="value">{summary?.config.strategy.rebuy_cash_ratio ?? 0}</div>
          </div>
        </div>
      </div>
      <div class="panel">
        <h2>Storage</h2>
        <p class="muted">PGSQL: {summary?.pg_enabled ? "Enabled" : "Disabled"}</p>
        {#if summary?.backtest}
          <div class="badge">Latest Backtest</div>
          <p>Run: {summary.backtest.run_id}</p>
          <p class="muted">
            {summary.backtest.start_time} → {summary.backtest.end_time}
          </p>
          <p class="muted">
            Return: {summary.backtest.return_rate.toFixed(4)} · Trades:
            {summary.backtest.trade_count}
          </p>
        {:else}
          <p class="muted">No backtest data available.</p>
        {/if}
      </div>
    </section>

    <section class="section">
      <h2>Recent Trades</h2>
      <div class="panel">
        <table class="table">
          <thead>
            <tr>
              <th>Time</th>
              <th>Symbol</th>
              <th>Side</th>
              <th>Price</th>
              <th>Qty</th>
              <th>Fee</th>
            </tr>
          </thead>
          <tbody>
            {#each summary?.trades ?? [] as trade}
              <tr>
                <td>{trade.time}</td>
                <td>{trade.symbol}</td>
                <td>{trade.side}</td>
                <td>{trade.price.toFixed(4)}</td>
                <td>{trade.qty.toFixed(6)}</td>
                <td>{trade.fee.toFixed(6)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>

    <section class="section layout-two">
      <div class="panel">
        <h2>Orders</h2>
        <table class="table">
          <thead>
            <tr>
              <th>Time</th>
              <th>Symbol</th>
              <th>Side</th>
              <th>Type</th>
              <th>Qty</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {#each summary?.orders ?? [] as order}
              <tr>
                <td>{order.time}</td>
                <td>{order.symbol}</td>
                <td>{order.side}</td>
                <td>{order.order_type}</td>
                <td>{order.qty.toFixed(6)}</td>
                <td>{order.status}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      <div class="panel">
        <h2>Positions</h2>
        <table class="table">
          <thead>
            <tr>
              <th>Symbol</th>
              <th>Qty</th>
              <th>Avg</th>
            </tr>
          </thead>
          <tbody>
            {#each summary?.positions ?? [] as pos}
              <tr>
                <td>{pos.symbol}</td>
                <td>{pos.qty.toFixed(6)}</td>
                <td>{pos.avg_price.toFixed(4)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>

    <section class="section">
      <h2>Balances</h2>
      <div class="panel">
        <table class="table">
          <thead>
            <tr>
              <th>Asset</th>
              <th>Free</th>
              <th>Locked</th>
            </tr>
          </thead>
          <tbody>
            {#each summary?.balances ?? [] as bal}
              <tr>
                <td>{bal.asset}</td>
                <td>{bal.free.toFixed(4)}</td>
                <td>{bal.locked.toFixed(4)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  {/if}
</div>
