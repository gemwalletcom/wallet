CREATE TYPE platform AS ENUM ('ios', 'android');
CREATE TYPE platform_store AS ENUM ('appStore', 'googlePlay', 'fdroid', 'huawei', 'solanaStore', 'samsungStore', 'apkUniversal', 'emerald', 'local');
CREATE TYPE device_locale AS ENUM (
    'ar', 'bn', 'cs', 'da', 'de', 'en', 'es', 'fa', 'fil', 'fr', 'ha', 'he', 'hi', 'id', 'it', 'ja',
    'ko', 'ms', 'nl', 'pl', 'pt-BR', 'ro', 'ru', 'sw', 'th', 'tr', 'uk', 'ur', 'vi', 'zh-Hans', 'zh-Hant'
);

CREATE TABLE devices (
    id SERIAL PRIMARY KEY,
    device_id VARCHAR(64) NOT NULL,
    is_push_enabled boolean NOT NULL,
    platform platform NOT NULL,
    platform_store platform_store NOT NULL,
    token VARCHAR(256) NOT NULL,
    locale device_locale NOT NULL,
    version VARCHAR(8) NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    currency VARCHAR(8) NOT NULL REFERENCES fiat_rates (id) ON DELETE CASCADE,
    subscriptions_version INTEGER NOT NULL DEFAULT 0,
    is_price_alerts_enabled boolean NOT NULL DEFAULT false,
    os VARCHAR(64) NOT NULL,
    model VARCHAR(128) NOT NULL,
    UNIQUE(device_id)
);

CREATE INDEX devices_token_idx ON devices (token);

SELECT diesel_manage_updated_at('devices');
