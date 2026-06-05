CREATE TABLE support_sessions (
    device_id INTEGER PRIMARY KEY REFERENCES devices (id) ON DELETE CASCADE,
    auth_token VARCHAR(1024) NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('support_sessions');
