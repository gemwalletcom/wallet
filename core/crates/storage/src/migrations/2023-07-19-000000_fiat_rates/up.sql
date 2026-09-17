CREATE TYPE currency AS ENUM (
    'MXN', 'CHF', 'CNY', 'THB', 'HUF', 'AUD', 'IDR', 'RUB', 'ZAR', 'EUR', 'NZD', 'SAR', 'SGD', 'BMD', 'KWD', 'HKD',
    'JPY', 'GBP', 'DKK', 'KRW', 'PHP', 'CLP', 'TWD', 'PKR', 'BRL', 'CAD', 'BHD', 'MMK', 'VEF', 'VND', 'CZK', 'TRY',
    'INR', 'ARS', 'BDT', 'NOK', 'USD', 'LKR', 'ILS', 'PLN', 'NGN', 'UAH', 'XDR', 'MYR', 'AED', 'SEK',
    'BYN', 'KZT', 'UZS', 'EGP', 'KES', 'COP', 'MAD', 'GHS', 'PEN'
);

CREATE TYPE fiat_rate_provider AS ENUM ('coingecko', 'coinmarketcap');

CREATE TABLE fiat_rates (
    id currency NOT NULL PRIMARY KEY,
    name VARCHAR NOT NULL,
    rate float NOT NULL DEFAULT 0,
    provider fiat_rate_provider NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at timestamp NOT NULL default current_timestamp,
    updated_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('fiat_rates');
