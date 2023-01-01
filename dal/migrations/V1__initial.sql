CREATE TABLE users (
    id VARCHAR(32) NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE oauth2_authorization_start (
    id VARCHAR(32) NOT NULL,
    user_id VARCHAR(32) NOT NULL,
    timestamp BIGINT NOT NULL,
    caller TEXT NOT NULL,
    scopes TEXT NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE oauth2_tokens (
    user_id VARCHAR(32) NOT NULL,
    token VARCHAR(2048) NOT NULL,
    token_type ENUM('Access', 'Refresh') NOT NULL,
    expiry BIGINT NOT NULL,
    PRIMARY KEY (user_id, token_type),
    FOREIGN KEY (user_id) REFERENCES users(id)
);