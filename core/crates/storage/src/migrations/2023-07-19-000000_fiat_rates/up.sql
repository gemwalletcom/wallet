CREATE TYPE currency AS ENUM (
    'MXN', 'CHF', 'CNY', 'THB', 'HUF', 'AUD', 'IDR', 'RUB', 'ZAR', 'EUR', 'NZD', 'SAR', 'SGD', 'BMD', 'KWD', 'HKD',
    'JPY', 'GBP', 'DKK', 'KRW', 'PHP', 'CLP', 'TWD', 'PKR', 'BRL', 'CAD', 'BHD', 'MMK', 'VEF', 'VND', 'CZK', 'TRY',
    'INR', 'ARS', 'BDT', 'NOK', 'USD', 'LKR', 'ILS', 'PLN', 'NGN', 'UAH', 'XDR', 'MYR', 'AED', 'SEK'
);

CREATE TABLE fiat_rates (
    id currency NOT NULL PRIMARY KEY,
    name VARCHAR NOT NULL,
    rate float NOT NULL DEFAULT 0,
    created_at timestamp NOT NULL default current_timestamp,
    updated_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('fiat_rates');
