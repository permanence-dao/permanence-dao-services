CREATE TABLE IF NOT EXISTS pdao_member
(
    id                          SERIAL PRIMARY KEY,
    name                        VARCHAR(128) NOT NULL,
    telegram_username           VARCHAR(128) NOT NULL,
    polkadot_address            VARCHAR(128) NOT NULL,
    polkadot_payment_address    VARCHAR(128) NOT NULL,
    kusama_address              VARCHAR(128) NOT NULL,
    kusama_payment_address      VARCHAR(128) NOT NULL,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT pda_member_u_telegram_username UNIQUE (telegram_username),
    CONSTRAINT pda_member_u_polkadot_address UNIQUE (polkadot_address),
    CONSTRAINT pda_member_u_kusama_address UNIQUE (kusama_address)
);