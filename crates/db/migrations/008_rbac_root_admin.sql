-- =====================================
-- RBAC + ROOT ADMIN SUPPORT
-- =====================================

ALTER TABLE users

ADD COLUMN IF NOT EXISTS role
TEXT NOT NULL
DEFAULT 'viewer';

ALTER TABLE users

ADD COLUMN IF NOT EXISTS is_admin
BOOLEAN NOT NULL
DEFAULT FALSE;

ALTER TABLE users

ADD COLUMN IF NOT EXISTS is_root
BOOLEAN NOT NULL
DEFAULT FALSE;

-- =====================================
-- MAKE EXISTING ADMIN ACCOUNT ROOT
-- =====================================

UPDATE users

SET

    role = 'admin',

    is_admin = TRUE,

    is_root = TRUE

WHERE username = 'admin';

-- =====================================
-- INDEXES
-- =====================================

CREATE INDEX IF NOT EXISTS idx_users_role
ON users(role);

CREATE INDEX IF NOT EXISTS idx_users_root
ON users(is_root);