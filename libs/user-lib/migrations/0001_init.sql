-- Create users table
CREATE TABLE users (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    CONSTRAINT `user_email_unique` UNIQUE (email)
);

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