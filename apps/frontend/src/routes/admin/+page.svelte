<script lang="ts">

    import { onMount } from "svelte";

    import { goto } from "$app/navigation";

    import api from "$lib/api";

    // =====================================
    // STATE
    // =====================================

    let users =
        $state<any[]>([]);

    let loading =
        $state(false);

    let creating =
        $state(false);

    let username =
        $state("");

    let password =
        $state("");

    let role =
        $state("viewer");

    let error =
        $state("");

    let success =
        $state("");

    // =====================================
    // LOAD USERS
    // =====================================

    async function loadUsers() {

        try {

            loading = true;

            const response =
                await api.get(
                    "/admin/users"
                );

            users =
                response.data;

        } catch (err) {

            console.error(err);

        } finally {

            loading = false;
        }
    }

    // =====================================
    // CREATE USER
    // =====================================

    async function createUser() {

        error = "";

        success = "";

        try {

            creating = true;

            const response =
                await api.post(

                    "/admin/create-user",

                    {

                        username,

                        password,

                        role
                    }
                );

            if (
                response.data.success
            ) {

                success =
                    "User created";

                username = "";

                password = "";

                role = "viewer";

                await loadUsers();

            } else {

                error =
                    response.data.error;
            }

        } catch (err) {

            console.error(err);

            error =
                "Failed to create user";

        } finally {

            creating = false;
        }
    }

    // =====================================
    // DELETE USER
    // =====================================

    async function deleteUser(
        id: number
    ) {

        const confirmed =
            confirm(
                "Delete this user?"
            );

        if (!confirmed) {

            return;
        }

        try {

            const response =
                await api.delete(
                    `/admin/users/${id}`
                );

            if (
                response.data.success
            ) {

                await loadUsers();

            } else {

                alert(
                    response.data.error
                );
            }

        } catch (err) {

            console.error(err);

            alert(
                "Failed to delete user"
            );
        }
    }

    // =====================================
    // AUTH CHECK
    // =====================================

    onMount(async () => {

        const token =
            localStorage.getItem(
                "token"
            );

        if (!token) {

            goto("/login");

            return;
        }

        await loadUsers();
    });

</script>

<div class="page">

    <div class="header">

        <div>

            <h1>
                Admin Dashboard
            </h1>

            <p>
                User & Platform Management
            </p>

            <div class="nav-actions">

                <button
                    class="nav-btn"

                    onclick={() => goto("/")}
                >

                    Dashboard

                </button>

                {#if role === "admin"}

                    <button
                        class="nav-btn"

                        onclick={() => goto("/admin")}
                    >

                        Admin

                    </button>

                {/if}

                <button

                    class="logout-btn"

                    onclick={() => {

                        localStorage.removeItem(
                            "token"
                        );

                        localStorage.removeItem(
                            "role"
                        );

                        localStorage.removeItem(
                            "is_root"
                        );

                        goto("/login");
                    }}
                >

                    Logout

                </button>

            </div>

        </div>

    </div>

    <!-- CREATE USER -->

    <div class="card">

        <h2>
            Create User
        </h2>

        <div class="form-grid">

            <input

                bind:value={username}

                placeholder="Username"
            />

            <input

                bind:value={password}

                type="password"

                placeholder="Password"
            />

            <select bind:value={role}>

                <option value="viewer">
                    Viewer
                </option>

                <option value="analyst">
                    Analyst
                </option>

                <option value="admin">
                    Admin
                </option>

            </select>

            <button

                onclick={createUser}

                disabled={creating}
            >

                {#if creating}

                    Creating...

                {:else}

                    Create User

                {/if}

            </button>

        </div>

        {#if error}

            <div class="error">
                {error}
            </div>

        {/if}

        {#if success}

            <div class="success">
                {success}
            </div>

        {/if}

    </div>

    <!-- USERS -->

    <div class="card">

        <h2>
            Users
        </h2>

        {#if loading}

            <div class="loading">

                Loading users...

            </div>

        {:else}

            <table>

                <thead>

                    <tr>

                        <th>
                            Username
                        </th>

                        <th>
                            Role
                        </th>

                        <th>
                            Root
                        </th>

                        <th>
                            Actions
                        </th>

                    </tr>

                </thead>

                <tbody>

                    {#each users as user}

                        <tr>

                            <td>

                                {user.username}

                            </td>

                            <td>

                                <span class="role">

                                    {user.role}

                                </span>

                            </td>

                            <td>

                                {#if user.is_root}

                                    <span class="root">

                                        ROOT

                                    </span>

                                {:else}

                                    —

                                {/if}

                            </td>

                            <td>

                                {#if !user.is_root}

                                    <button

                                        class="delete-btn"

                                        onclick={() =>
                                            deleteUser(
                                                user.id
                                            )
                                        }
                                    >

                                        Delete

                                    </button>

                                {:else}

                                    <span class="locked">

                                        Protected

                                    </span>

                                {/if}

                            </td>

                        </tr>

                    {/each}

                </tbody>

            </table>

        {/if}

    </div>

</div>

<style>

.page {

    padding: 24px;

    min-height: 100vh;

    background:
        linear-gradient(
            to bottom,
            #020617,
            #0f172a
        );

    color: white;
}

.header {

    margin-bottom: 24px;
}

h1 {

    margin: 0;

    font-size: 36px;

    font-weight: 800;
}

p {

    color: #94a3b8;
}

.card {

    background:
        linear-gradient(
            to bottom,
            #0f172a,
            #111827
        );

    border:
        1px solid #1e293b;

    border-radius: 16px;

    padding: 24px;

    margin-bottom: 24px;
}

h2 {

    margin-top: 0;

    margin-bottom: 20px;
}

.form-grid {

    display: grid;

    grid-template-columns:
        repeat(
            4,
            1fr
        );

    gap: 16px;
}

input,
select {

    background: #020617;

    border:
        1px solid #334155;

    color: white;

    padding: 14px;

    border-radius: 10px;
}

button {

    background: #2563eb;

    border: none;

    color: white;

    padding: 14px;

    border-radius: 10px;

    cursor: pointer;

    font-weight: 700;
}

button:hover {

    background: #1d4ed8;
}

button:disabled {

    opacity: 0.6;
}

table {

    width: 100%;

    border-collapse: collapse;
}

th,
td {

    padding: 16px;

    text-align: left;

    border-bottom:
        1px solid #1e293b;
}

th {

    color: #94a3b8;

    font-size: 13px;

    text-transform: uppercase;
}

.role {

    background:
        rgba(37,99,235,0.2);

    color: #60a5fa;

    padding: 6px 10px;

    border-radius: 999px;

    font-size: 12px;

    font-weight: 700;
}

.root {

    background:
        rgba(34,197,94,0.2);

    color: #4ade80;

    padding: 6px 10px;

    border-radius: 999px;

    font-size: 12px;

    font-weight: 700;
}

.locked {

    color: #94a3b8;
}

.delete-btn {

    background: #dc2626;
}

.delete-btn:hover {

    background: #b91c1c;
}

.error {

    margin-top: 16px;

    color: #f87171;
}

.success {

    margin-top: 16px;

    color: #4ade80;
}

.loading {

    color: #94a3b8;
}

.nav-actions {

    display: flex;

    gap: 12px;

    align-items: center;
}

.nav-btn {

    background: #1e293b;

    border:
        1px solid #334155;

    color: white;

    padding: 10px 18px;

    border-radius: 10px;

    cursor: pointer;

    font-weight: 700;

    transition: 0.15s;
}

.nav-btn:hover {

    background: #334155;
}

.logout-btn {

    background: #dc2626;

    border: none;

    color: white;

    padding: 10px 18px;

    border-radius: 10px;

    cursor: pointer;

    font-weight: 700;
}

.logout-btn:hover {

    background: #b91c1c;
}
</style>