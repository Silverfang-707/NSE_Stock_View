<script lang="ts">

    import { onMount } from "svelte";
    import api from "$lib/api";

    let username = "";

    let password = "";

    let loading = false;

    let error = "";

    onMount(() => {

    const token =

        localStorage.getItem(
            "token"
        );

    if (token) {

        window.location.href = "/";
    }
});
    async function login() {

        error = "";

        loading = true;

        try {

            const response =
                await api.post(

                    "/auth/login",

                    {

                        username,

                        password
                    }
                );

            localStorage.setItem(

                "token",

                response.data.token
            );
            localStorage.setItem(
                "role",
                response.data.role
            );

            localStorage.setItem(
                "is_root",
                response.data.is_root
            );

            window.location.href = "/";

        } catch (err: any) {

            error =
                err?.response?.data?.message ??

                "Login failed";
        }

        loading = false;
    }

</script>

<div class="page">

    <div class="login-card">

        <h1>
            Admin Login
        </h1>

        <p>
            NSE Market Terminal
        </p>

        <div class="field">

            <label>
                Username
            </label>

            <input

                bind:value={username}

                type="text"

                placeholder="Enter username"
            />

        </div>

        <div class="field">

            <label>
                Password
            </label>

            <input

                bind:value={password}

                type="password"

                placeholder="Enter password"
            />

        </div>

        {#if error}

            <div class="error">

                {error}

            </div>

        {/if}

        <button

            onclick={login}

            disabled={loading}
        >

            {#if loading}

                Logging in...

            {:else}

                Login

            {/if}

        </button>

    </div>

</div>

<style>

.page {

    min-height: 100vh;

    display: flex;

    justify-content: center;

    align-items: center;

    background:
        linear-gradient(
            to bottom,
            #020617,
            #0f172a
        );

    color: white;
}

.login-card {

    width: 360px;

    background:
        linear-gradient(
            to bottom,
            #0f172a,
            #111827
        );

    border:
        1px solid #1e293b;

    border-radius: 18px;

    padding: 32px;

    display: flex;

    flex-direction: column;

    gap: 18px;
}

h1 {

    margin: 0;

    font-size: 32px;

    font-weight: 800;
}

p {

    margin: 0;

    color: #94a3b8;
}

.field {

    display: flex;

    flex-direction: column;

    gap: 8px;
}

label {

    font-size: 13px;

    color: #94a3b8;

    font-weight: 700;
}

input {

    background: #020617;

    border:
        1px solid #334155;

    color: white;

    padding: 14px;

    border-radius: 10px;

    outline: none;
}

input:focus {

    border-color: #2563eb;
}

button {

    background: #2563eb;

    border: none;

    color: white;

    padding: 14px;

    border-radius: 10px;

    cursor: pointer;

    font-weight: 700;

    margin-top: 8px;
}

button:hover {

    background: #1d4ed8;
}

button:disabled {

    opacity: 0.6;

    cursor: not-allowed;
}

.error {

    background:
        rgba(239,68,68,0.1);

    border:
        1px solid rgba(239,68,68,0.3);

    color: #f87171;

    padding: 12px;

    border-radius: 10px;
}

</style>