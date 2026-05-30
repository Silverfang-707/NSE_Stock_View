<script lang="ts">
  import { onMount } from 'svelte';

  // ============================================================
  // CONFIG
  // ============================================================
  const API = 'http://localhost:3000';

  const PLAN_MULTIPLIERS = [0.0, 0.382, 0.6535, 1.0, 2.0, 3.618, 6.236, 10.382];
  const TIMEFRAMES       = ['daily', 'weekly', 'monthly', 'quarterly', 'half_yearly', 'yearly'] as const;

  type Timeframe = typeof TIMEFRAMES[number];

  const TF_LABELS: Record<Timeframe, string> = {
    daily:      'Daily',
    weekly:     'Weekly',
    monthly:    'Monthly',
    quarterly:  'Quarterly',
    half_yearly:'Half Year',
    yearly:     'Yearly',
  };

  // ============================================================
  // TYPES
  // ============================================================
  interface Level {
    timeframe:    string;
    trade_date:   string;
    open_price:   number;
    high_price:   number;
    low_price:    number;
    close_price:  number;
    bdp:          number;
    wdp:          number;
    range_value:  number;
    buffer_value: number;
    jgd:          number;
    jwd:          number;
    pattern:      string;
    
    // Legacy support fields
    prev_jgd:     number | null;
    prev_jwd:     number | null;
    prev_bdp:     number | null;
    prev_wdp:     number | null;
  }

  interface AdminUser {
    id:       number;
    username: string;
    role:     string;
    is_root:  boolean;
  }

  interface Grid {
    greenCols:     number[];
    greenAbove:    number[];
    greenBelow:    number[];
    redCols:       number[];
    redAbove:      number[];
    redBelow:      number[];
    jgdGreenCols:  number[];
    jgdGreenAbove: number[];
    jgdGreenBelow: number[];
    jgdRedCols:    number[];
    jgdRedAbove:   number[];
    jgdRedBelow:   number[];
    legacyContext: string[] | null;
    newBdp:        string;
    newWdp:        string;
    rangeVal:      string;
  }

  // ============================================================
  // AUTH STATE
  // ============================================================
  let token        = $state('');
  let userRole     = $state('');
  let isRoot       = $state(false);
  let view         = $state<'login' | 'dashboard' | 'admin'>('login');

  let loginUser    = $state('');
  let loginPass    = $state('');
  let loginError   = $state('');
  let loginLoading = $state(false);

  // ============================================================
  // DASHBOARD STATE
  // ============================================================
  let symbols     = $state<string[]>([]);
  let series_list = $state<string[]>([]);
  let selectedSym = $state('');
  let selectedSer = $state('');
  let levels      = $state<Level[]>([]);
  let loading     = $state(false);
  let error       = $state('');

  // ============================================================
  // ADMIN STATE
  // ============================================================
  let adminUsers    = $state<AdminUser[]>([]);
  let adminLoading  = $state(false);
  let newUsername   = $state('');
  let newPassword   = $state('');
  let newRole       = $state('viewer');
  let adminCreating = $state(false);
  let adminError    = $state('');
  let adminSuccess  = $state('');

  // ============================================================
  // DERIVED
  // ============================================================
  const levelMap = $derived.by(() => {
    const map: Partial<Record<Timeframe, Level>> = {};
    for (const tf of TIMEFRAMES) {
      const rows = levels
        .filter(l => l.timeframe === tf)
        .sort((a, b) => b.trade_date.localeCompare(a.trade_date));
      if (rows.length) map[tf] = rows[0];
    }
    return map;
  });

  const summaryRows = $derived.by(() => {
    const result: { high: Record<string, string>; low: Record<string, string> } = { high: {}, low: {} };
    for (const tf of TIMEFRAMES) {
      const l = levelMap[tf];
      result.high[tf] = l ? fmt(l.high_price) : '—';
      result.low[tf]  = l ? fmt(l.low_price)  : '—';
    }
    return result;
  });

  // ============================================================
  // HELPERS
  // ============================================================
  function fmt(v: number | null | undefined): string {
    if (v == null || isNaN(v)) return '—';
    return Number(v).toFixed(2);
  }

  function round2(v: number): number {
    return Math.round(v * 100) / 100;
  }

  function buildGrid(level: Level): Grid | null {
    if (!level) return null;
    const { bdp, wdp, range_value: r, buffer_value: buf, jgd, jwd, pattern } = level;

    const greenCols  = PLAN_MULTIPLIERS.map(m => round2(bdp + r * m));
    const redCols    = PLAN_MULTIPLIERS.map(m => round2(wdp - r * m));
    const greenAbove = greenCols.map(c => round2(c + buf));
    const greenBelow = greenCols.map(c => round2(c - buf));
    const redAbove   = redCols.map(c => round2(c + buf));
    const redBelow   = redCols.map(c => round2(c - buf));

    const jgdGreenCols  = [0.0, 0.382, 0.6535].map(m => round2(jwd + r * m));
    const jgdRedCols    = [0.0, 0.382, 0.6535].map(m => round2(jwd - r * m));
    const jgdGreenAbove = jgdGreenCols.map(c => round2(c + buf));
    const jgdGreenBelow = jgdGreenCols.map(c => round2(c - buf));
    const jgdRedAbove   = jgdRedCols.map(c => round2(c + buf));
    const jgdRedBelow   = jgdRedCols.map(c => round2(c - buf));

    // =====================================
    // LEGACY CONTEXT ROUTING (Based on Pattern)
    // =====================================
    let legacyContext: string[] | null = null;

    if (pattern === "3+1") {
      legacyContext = [level.jgd, level.prev_jgd, level.prev_jwd]
        .filter((v): v is number => v != null)
        .sort((a, b) => a - b)
        .map(fmt);
    } 
    else if (pattern === "2+2") {
      legacyContext = [level.prev_jgd, level.prev_jwd, level.jgd, level.jwd]
        .filter((v): v is number => v != null)
        .map(fmt);
    } 
    else if (pattern === "2+1") {
      legacyContext = [level.prev_jgd, level.jgd, level.jwd]
        .filter((v): v is number => v != null)
        .map(fmt);
    }

    return {
      greenCols, greenAbove, greenBelow,
      redCols,   redAbove,   redBelow,
      jgdGreenCols, jgdGreenAbove, jgdGreenBelow,
      jgdRedCols,   jgdRedAbove,   jgdRedBelow,
      legacyContext,
      newBdp:   fmt(bdp),
      newWdp:   fmt(wdp),
      rangeVal: fmt(r),
    };
  }

  // ============================================================
  // AUTH HELPERS
  // ============================================================
  function authHeaders(): Record<string, string> {
    return { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' };
  }

  async function doLogin() {
    loginError   = '';
    loginLoading = true;
    try {
      const res  = await fetch(`${API}/auth/login`, {
        method:  'POST',
        headers: { 'Content-Type': 'application/json' },
        body:    JSON.stringify({ username: loginUser, password: loginPass }),
      });
      const data = await res.json();
      
      if (!res.ok || !data.success || !data.token) {
        loginError = data.error || 'Invalid credentials';
        return;
      }
      
      token    = data.token;
      userRole = data.role     || '';
      isRoot   = data.is_root  || false;
      localStorage.setItem('token',   token);
      localStorage.setItem('role',    userRole);
      localStorage.setItem('is_root', String(isRoot));
      view = 'dashboard';
      await loadSymbols();
    } catch {
      loginError = 'Connection failed';
    } finally {
      loginLoading = false;
    }
  }

  function doLogout() {
    token       = '';
    userRole    = '';
    isRoot      = false;
    levels      = [];
    symbols     = [];
    series_list = [];
    localStorage.removeItem('token');
    localStorage.removeItem('role');
    localStorage.removeItem('is_root');
    view = 'login';
  }

  // ============================================================
  // ADMIN API
  // ============================================================
  async function loadAdminUsers() {
    adminLoading = true;
    try {
      const res  = await fetch(`${API}/admin/users`, { headers: authHeaders() });
      if (!res.ok) throw new Error('Failed to load');
      adminUsers = await res.json() as AdminUser[];
    } catch {
      adminUsers = [];
    } finally {
      adminLoading = false;
    }
  }

  async function createUser() {
    adminError    = '';
    adminSuccess  = '';
    adminCreating = true;
    try {
      const res  = await fetch(`${API}/admin/create-user`, {
        method:  'POST',
        headers: authHeaders(),
        body:    JSON.stringify({ username: newUsername, password: newPassword, role: newRole }),
      });
      const data = await res.json();
      if (data.success) {
        adminSuccess = 'User created successfully';
        newUsername  = '';
        newPassword  = '';
        newRole      = 'viewer';
        await loadAdminUsers();
      } else {
        adminError = data.error || 'Failed to create user';
      }
    } catch {
      adminError = 'Failed to create user';
    } finally {
      adminCreating = false;
    }
  }

  async function deleteUser(id: number) {
    if (!confirm('Delete this user?')) return;
    try {
      const res  = await fetch(`${API}/admin/users/${id}`, {
        method:  'DELETE',
        headers: authHeaders(),
      });
      const data = await res.json();
      if (data.success) {
        await loadAdminUsers();
      } else {
        alert(data.error || 'Failed to delete user');
      }
    } catch {
      alert('Failed to delete user');
    }
  }

  function goAdmin() {
    adminError   = '';
    adminSuccess = '';
    view = 'admin';
    loadAdminUsers();
  }

  function goDashboard() {
    view = 'dashboard';
  }

  // ============================================================
  // DASHBOARD API
  // ============================================================
  async function loadSymbols() {
    try {
      const res = await fetch(`${API}/symbols`, { headers: authHeaders() });
      if (!res.ok) throw new Error();
      symbols   = await res.json() as string[];
      if (symbols.length) selectedSym = symbols[0];
    } catch {
      error = 'Failed to load symbols';
    }
  }

  async function loadSeries() {
    if (!selectedSym) return;
    try {
      const res   = await fetch(`${API}/series/${selectedSym}`, { headers: authHeaders() });
      if (!res.ok) throw new Error();
      series_list = await res.json() as string[];
      selectedSer = series_list.length ? series_list[0] : '';
    } catch {
      series_list = [];
    }
  }

  async function loadLevels() {
    if (!selectedSym) return;
    loading = true;
    error   = '';
    try {
      const res = await fetch(`${API}/levels/${selectedSym}`, { headers: authHeaders() });
      if (!res.ok) throw new Error();
      const all = await res.json() as Level[];
      
      // Removed the `.filter(l => l.series === selectedSer)` logic since 
      // the Axum backend isn't returning 'series' in the response yet.
      levels = all; 
    } catch {
      error  = 'Failed to load levels';
      levels = [];
    } finally {
      loading = false;
    }
  }

  // ============================================================
  // EFFECTS
  // ============================================================
  $effect(() => {
    if (selectedSym) {
      loadSeries();
      loadLevels();
    }
  });

  onMount(async () => {
    const t  = localStorage.getItem('token');
    const r  = localStorage.getItem('role');
    const ir = localStorage.getItem('is_root');
    if (t) {
      token    = t;
      userRole = r    || '';
      isRoot   = ir === 'true';
      view     = 'dashboard';
      await loadSymbols();
    }
  });
</script>

<style>
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

  :root {
    --bg:         #e2e2e2;
    --white:      #ffffff;
    --border:     #d0d0d0;
    --text:       #222222;
    --text-muted: #666666;
    --green-bg:   #1a7a3c;
    --red-bg:     #a0103a;
    --hint-red:   #f0293d;
    --hint-blue:  #3636ff;
    --header-bg:  #e8e8e8;
    --topbar:     #2b2b2b;
    --cell-pad:   4px 10px;
    --radius:     0px; 
  }

  .page {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    font-size: 13px;
    color: var(--text);
    background: var(--bg);
    min-height: 100vh;
    padding: 0 0 60px;
  }

  /* ── LOGIN ── */
  .login-wrap {
    min-height: 100vh;
    background: var(--bg);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  }

  .login-box {
    background: var(--white);
    border: 1px solid var(--border);
    padding: 32px 36px 28px;
    width: 320px;
  }

  .login-title {
    font-size: 17px;
    font-weight: 700;
    color: var(--text);
    margin-bottom: 20px;
    letter-spacing: 0.02em;
    border-bottom: 2px solid var(--topbar);
    padding-bottom: 10px;
  }

  .login-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 12px;
  }

  .login-field label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .login-field input {
    padding: 7px 10px;
    font-size: 13px;
    border: 1px solid var(--border);
    background: #fafafa;
    color: var(--text);
    outline: none;
    font-family: inherit;
  }

  .login-field input:focus {
    border-color: var(--topbar);
    background: var(--white);
  }

  .login-btn {
    width: 100%;
    padding: 8px;
    background: var(--topbar);
    color: white;
    border: none;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    margin-top: 8px;
    letter-spacing: 0.04em;
    font-family: inherit;
  }

  .login-btn:hover:not(:disabled) { background: #444; }
  .login-btn:disabled { opacity: 0.6; cursor: default; }

  .login-error {
    color: var(--hint-red);
    font-size: 12px;
    margin-top: 10px;
  }

  /* ── TOP BAR ── */
  .topbar {
    background: var(--topbar);
    padding: 8px 16px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .topbar-left  { display: flex; align-items: center; gap: 8px; flex: 1; }
  .topbar-right { display: flex; align-items: center; gap: 6px; }

  .topbar select {
    padding: 6px 10px;
    font-size: 13px;
    border: none;
    background: white;
    min-width: 150px;
    cursor: pointer;
    font-family: inherit;
  }

  .topbar button {
    padding: 6px 12px;
    background: #555;
    color: white;
    border: none;
    font-size: 13px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: inherit;
  }

  .topbar button:hover   { background: #444; }
  .admin-btn             { background: #444 !important; }
  .admin-btn:hover       { background: #555 !important; }
  .admin-btn.active      { background: #1a5c9a !important; }
  .logout-btn            { background: #7a1a1a !important; }
  .logout-btn:hover      { background: #6b1111 !important; }

  .user-badge {
    font-size: 11px;
    color: #aaa;
    padding: 0 8px;
    border-left: 1px solid #444;
    line-height: 1.3;
  }

  .user-badge strong { color: #ddd; font-size: 12px; display: block; }

  /* ── SUMMARY TABLE ── */
  .summary-wrap { padding: 14px 20px 0; display: flex; justify-content: center; }

  .summary-table {
    border-collapse: collapse;
    width: 100%;
    max-width: 1000px;
    background: var(--bg);
    border: 1px solid var(--border);
  }

  .summary-table th,
  .summary-table td {
    border: 1px solid var(--border);
    padding: 10px 14px;
    text-align: center;
    white-space: nowrap;
    color: var(--text);
  }

  .summary-table th {
    background: var(--header-bg);
    font-weight: 500;
    font-size: 13px;
  }

  .summary-table td:first-child {
    font-weight: 600;
    background: var(--header-bg);
    text-align: center;
  }

  /* ── PLANS ── */
  .plans { padding: 24px 20px; display: flex; flex-direction: column; gap: 40px; align-items: center;}

  .plan-block {
    width: 100%;
    max-width: 1200px;
    background: var(--bg);
    padding: 16px 20px 20px;
  }

  .plan-title {
    text-align: center;
    font-size: 18px;
    font-weight: 400;
    margin-bottom: 12px;
    color: var(--text);
  }

  .legacy-headers {
    display: flex;
    justify-content: center;
    gap: 150px;
    margin-bottom: 16px;
    font-size: 13px;
    color: var(--text);
  }

  .plan-body {
    display: flex;
    gap: 80px;
    justify-content: center;
    align-items: flex-start;
  }

  .jgd-side { display: flex; flex-direction: column; align-items: center; min-width: 240px; }

  .side-label {
    font-size: 12px;
    font-weight: 400;
    color: var(--text);
    margin-bottom: 5px;
    align-self: flex-end;
    padding-right: 4px;
  }

  .grid-3col { display: flex; flex-direction: column; gap: 4px; align-items: center; }
  .row-3     { display: flex; gap: 4px; }

  .cell {
    width: 62px;
    text-align: center;
    padding: 4px 6px;
    font-size: 13px;
    border-radius: var(--radius);
    font-variant-numeric: tabular-nums;
  }

  .cell.plain { color: var(--text); background: transparent; }
  .cell.green { background: var(--green-bg); color: white; font-weight: 600; }
  .cell.red   { background: var(--red-bg);   color: white; font-weight: 600; }

  .new-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 8px 0;
    font-size: 13px;
    width: 100%;
    justify-content: center;
  }

  .new-row .lval  { min-width: 44px; text-align: right; color: var(--text); }
  .new-row .label { min-width: 70px; text-align: center; color: var(--text); font-weight: 500; }
  .new-row .rval  { min-width: 44px; text-align: left; font-weight: 500; }

  .context-hints { 
    display: flex; 
    gap: 14px; 
    margin: 10px 0; 
    justify-content: center; 
    flex-wrap: wrap; 
    max-width: 180px; 
    line-height: 1.6;
  }
  .context-hints span { font-size: 13px; color: var(--hint-red); font-weight: 500; }
  .context-hints.monthly-hint span { color: var(--hint-blue); }

  .bdp-side  { display: flex; flex-direction: column; align-items: flex-start; flex: 1; max-width: 800px; }
  .grid-8col { display: flex; flex-direction: column; gap: 4px; width: 100%; }
  .row-8     { display: flex; gap: 4px; justify-content: center; }

  .cell-8 {
    flex: 1;
    min-width: 58px;
    text-align: center;
    padding: 4px 6px;
    font-size: 13px;
    border-radius: var(--radius);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .cell-8.plain { color: var(--text); }
  .cell-8.green { background: var(--green-bg); color: white; font-weight: 600; }
  .cell-8.red   { background: var(--red-bg);   color: white; font-weight: 600; }

  .zone-sep { height: 60px; display: flex; align-items: center; justify-content: center; width: 100%; }

  /* ── ADMIN ── */
  .admin-wrap { padding: 16px 20px; display: flex; flex-direction: column; gap: 20px; }
  .admin-card { background: white; border: 1px solid var(--border); padding: 16px 20px; }
  .admin-card h2 { font-size: 13px; font-weight: 700; margin-bottom: 14px; color: var(--text-muted); border-bottom: 1px solid var(--border); padding-bottom: 8px; text-transform: uppercase; letter-spacing: 0.06em; }
  .form-row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .form-row input, .form-row select { padding: 6px 10px; font-size: 13px; border: 1px solid var(--border); background: #fafafa; color: var(--text); font-family: inherit; outline: none; }
  .form-row input:focus, .form-row select:focus { border-color: var(--topbar); background: white; }
  .form-row input { min-width: 140px; }
  .form-btn { padding: 6px 14px; background: var(--topbar); color: white; border: none; font-size: 13px; font-weight: 600; cursor: pointer; font-family: inherit; }
  .form-btn:hover:not(:disabled) { background: #444; }
  .form-btn:disabled { opacity: 0.6; cursor: default; }
  .admin-msg { font-size: 12px; margin-top: 10px; }
  .admin-msg.err { color: var(--hint-red); }
  .admin-msg.ok  { color: var(--green-bg); }
  .admin-table { width: 100%; border-collapse: collapse; }
  .admin-table th, .admin-table td { border: 1px solid var(--border); padding: 6px 12px; text-align: left; font-size: 13px; }
  .admin-table th { background: var(--header-bg); font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-muted); }
  .badge { display: inline-block; padding: 2px 8px; font-size: 11px; font-weight: 700; border-radius: 2px; }
  .badge-role { background: #dbeafe; color: #1d4ed8; }
  .badge-root { background: #dcfce7; color: #166534; }
  .badge-lock { background: #f3f4f6; color: #6b7280; }
  .del-btn { padding: 3px 10px; background: #b91c1c; color: white; border: none; font-size: 12px; cursor: pointer; font-family: inherit; }
  .del-btn:hover { background: #991b1b; }

  /* ── STATUS ── */
  .status    { padding: 40px; text-align: center; color: var(--text-muted); font-size: 14px; }
  .error-msg { color: var(--hint-red); padding: 12px 20px; font-size: 13px; }

  .spinner {
    display: inline-block;
    width: 16px; height: 16px;
    border: 2px solid #ddd;
    border-top-color: #555;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    vertical-align: middle;
    margin-right: 6px;
  }

  @keyframes spin { to { transform: rotate(360deg); } }
</style>

{#if view === 'login'}
<div class="login-wrap">
  <div class="login-box">
    <div class="login-title">Market Analysis Platform</div>

    <div class="login-field">
      <label for="lu">Username</label>
      <input
        id="lu"
        bind:value={loginUser}
        placeholder="Enter username"
        autocomplete="username"
        onkeydown={(e) => e.key === 'Enter' && doLogin()}
      />
    </div>

    <div class="login-field">
      <label for="lp">Password</label>
      <input
        id="lp"
        type="password"
        bind:value={loginPass}
        placeholder="Enter password"
        autocomplete="current-password"
        onkeydown={(e) => e.key === 'Enter' && doLogin()}
      />
    </div>

    <button class="login-btn" onclick={doLogin} disabled={loginLoading}>
      {#if loginLoading}
        <span class="spinner"></span> Signing in…
      {:else}
        Sign In
      {/if}
    </button>

    {#if loginError}
      <div class="login-error">{loginError}</div>
    {/if}
  </div>
</div>

{:else}
<div class="page">

  <div class="topbar">
    <div class="topbar-left">
      {#if view === 'dashboard'}
        <select bind:value={selectedSym}>
          {#each symbols as sym}
            <option value={sym}>{sym}</option>
          {/each}
        </select>

        <select bind:value={selectedSer}>
          {#each series_list as ser}
            <option value={ser}>{ser}</option>
          {/each}
        </select>
      {:else}
        <span style="color:#aaa; font-size:13px;">Admin Panel</span>
      {/if}
    </div>

    <div class="topbar-right">
      <div class="user-badge">
        <strong>{userRole}{isRoot ? ' ★' : ''}</strong>
        {userRole === 'admin' ? 'Administrator' : userRole}
      </div>

      {#if view === 'admin'}
        <button class="admin-btn" onclick={goDashboard}>← Dashboard</button>
      {:else if userRole === 'admin' || isRoot}
        <button class="admin-btn" onclick={goAdmin}>Admin</button>
      {/if}

      <button class="logout-btn" onclick={doLogout}>Logout</button>
    </div>
  </div>

  {#if view === 'dashboard'}

    {#if error}
      <div class="error-msg">{error}</div>
    {/if}

    {#if loading}
      <div class="status"><span class="spinner"></span> Loading…</div>
    {:else if levels.length === 0}
      <div class="status">No data. Select a symbol from the dropdown to load automatically.</div>
    {:else}

      <div class="summary-wrap">
        <table class="summary-table">
          <thead>
            <tr>
              <th></th>
              {#each TIMEFRAMES as tf}<th>{TF_LABELS[tf]}</th>{/each}
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>High</td>
              {#each TIMEFRAMES as tf}<td>{summaryRows.high[tf] ?? '—'}</td>{/each}
            </tr>
            <tr>
              <td>Low</td>
              {#each TIMEFRAMES as tf}<td>{summaryRows.low[tf] ?? '—'}</td>{/each}
            </tr>
          </tbody>
        </table>
      </div>

      <div class="plans">
        {#each TIMEFRAMES as tf}
          {@const level = levelMap[tf]}
          {@const g     = level ? buildGrid(level) : null}
          {#if level && g}
          <div class="plan-block">
            
            <div class="plan-title">{TF_LABELS[tf]} Plan</div>
            
            <div class="legacy-headers">
               <span>JGD</span>
               <span>BDP</span>
            </div>

            <div class="plan-body">

              <div class="jgd-side">
                <div class="side-label">WDP</div>
                <div class="grid-3col">
                  <div class="row-3">
                    {#each g.jgdGreenAbove as v}<div class="cell plain">{fmt(v)}</div>{/each}
                  </div>
                  <div class="row-3">
                    {#each g.jgdGreenCols as v}<div class="cell green">{fmt(v)}</div>{/each}
                  </div>
                  <div class="row-3">
                    {#each g.jgdGreenBelow as v}<div class="cell plain">{fmt(v)}</div>{/each}
                  </div>
                </div>

                <div class="new-row">
                  <span class="lval">{fmt(level.jgd)}</span>
                  <span class="label">NEW BDP</span>
                  <span class="rval">{g.newBdp}</span>
                </div>
                <div class="new-row">
                  <span class="lval">{g.rangeVal}</span>
                  <span class="label">NEW WDP</span>
                  <span class="rval">{g.newWdp}</span>
                </div>

                <div class="grid-3col">
                  <div class="row-3">
                    {#each g.jgdRedAbove as v}<div class="cell plain">{fmt(v)}</div>{/each}
                  </div>
                  <div class="row-3">
                    {#each g.jgdRedCols as v}<div class="cell red">{fmt(v)}</div>{/each}
                  </div>
                  <div class="row-3">
                    {#each g.jgdRedBelow as v}<div class="cell plain">{fmt(v)}</div>{/each}
                  </div>
                </div>
                <div class="side-label" style="align-self:center; margin-top:10px;">WDP</div>
              </div>

              <div class="bdp-side">
                <div class="grid-8col">
                  <div class="row-8">
                    {#each g.greenAbove as v}<div class="cell-8 plain">{fmt(v)}</div>{/each}
                  </div>
                  <div class="row-8">
                    {#each g.greenCols as v}<div class="cell-8 green">{fmt(v)}</div>{/each}
                  </div>
                  <div class="row-8">
                    {#each g.greenBelow as v}<div class="cell-8 plain">{fmt(v)}</div>{/each}
                  </div>

                  <div class="zone-sep">
                    {#if g.legacyContext}
                      <div class="context-hints {tf === 'monthly' ? 'monthly-hint' : ''}">
                        {#each g.legacyContext as v}<span>{v}</span>{/each}
                      </div>
                    {/if}
                  </div>

                  <div class="row-8">
                    {#each g.redAbove as v}<div class="cell-8 plain">{fmt(v)}</div>{/each}
                  </div>
                  <div class="row-8">
                    {#each g.redCols as v}<div class="cell-8 red">{fmt(v)}</div>{/each}
                  </div>
                  <div class="row-8">
                    {#each g.redBelow as v}<div class="cell-8 plain">{fmt(v)}</div>{/each}
                  </div>
                </div>
              </div>

            </div>
          </div>
          {/if}
        {/each}
      </div>

    {/if}

  {:else if view === 'admin'}
  <div class="admin-wrap">
    <div class="admin-card">
      <h2>Create User</h2>
      <div class="form-row">
        <input bind:value={newUsername} placeholder="Username" />
        <input type="password" bind:value={newPassword} placeholder="Password" />
        <select bind:value={newRole}>
          <option value="viewer">Viewer</option>
          <option value="analyst">Analyst</option>
          <option value="admin">Admin</option>
        </select>
        <button class="form-btn" onclick={createUser} disabled={adminCreating}>
          {adminCreating ? 'Creating…' : 'Create User'}
        </button>
      </div>
      {#if adminError}   <div class="admin-msg err">{adminError}</div>   {/if}
      {#if adminSuccess} <div class="admin-msg ok">{adminSuccess}</div> {/if}
    </div>

    <div class="admin-card">
      <h2>Users</h2>
      {#if adminLoading}
        <div class="status"><span class="spinner"></span> Loading users…</div>
      {:else}
        <table class="admin-table">
          <thead>
            <tr>
              <th>Username</th>
              <th>Role</th>
              <th>Root</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each adminUsers as user}
            <tr>
              <td>{user.username}</td>
              <td><span class="badge badge-role">{user.role}</span></td>
              <td>
                {#if user.is_root}
                  <span class="badge badge-root">ROOT</span>
                {:else}—{/if}
              </td>
              <td>
                {#if user.is_root}
                  <span class="badge badge-lock">Protected</span>
                {:else}
                  <button class="del-btn" onclick={() => deleteUser(user.id)}>Delete</button>
                {/if}
              </td>
            </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>

  </div>
  {/if}

</div>
{/if}