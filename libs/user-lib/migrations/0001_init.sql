-- Create users table (Keycloak-integrated schema)
-- User profile data (name, email) is stored in Keycloak
-- Local DB only stores user ID and Keycloak reference
CREATE TABLE users (
    id CHAR(36) PRIMARY KEY,
    keycloak_id VARCHAR(255) NOT NULL,
    CONSTRAINT `user_keycloak_id_unique` UNIQUE (keycloak_id)
);

-- Create index for fast Keycloak ID lookups
CREATE INDEX idx_users_keycloak_id ON users(keycloak_id);

-- Create roles table
CREATE TABLE roles (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    CONSTRAINT `role_name_unique` UNIQUE (name)
);

-- Create join table for user-role relationship
CREATE TABLE user_roles (
    user_id CHAR(36) NOT NULL,
    role_id CHAR(36) NOT NULL,
    CONSTRAINT `user_roles_pk` PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);

-- Seed default roles
INSERT INTO roles (id, name) VALUES
    ('00000000-0000-0000-0000-000000000001', 'admin'),
    ('00000000-0000-0000-0000-000000000002', 'user');