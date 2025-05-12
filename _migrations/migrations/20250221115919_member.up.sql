CREATE TABLE IF NOT EXISTS pdao_membership_type
(
    code        VARCHAR(16) PRIMARY KEY,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);

INSERT INTO pdao_membership_type(code) VALUES('core') ON CONFLICT(code) DO NOTHING;
INSERT INTO pdao_membership_type(code) VALUES('community') ON CONFLICT(code) DO NOTHING;

CREATE TABLE IF NOT EXISTS pdao_member
(
    id                          SERIAL PRIMARY KEY,
    name                        VARCHAR(128) NOT NULL,
    telegram_username           VARCHAR(128) NOT NULL,
    polkadot_address            VARCHAR(128) NOT NULL,
    polkadot_payment_address    VARCHAR(128) NOT NULL,
    kusama_address              VARCHAR(128) NOT NULL,
    kusama_payment_address      VARCHAR(128) NOT NULL,
    is_on_leave                 BOOLEAN NOT NULL DEFAULT FALSE,
    is_removed                  BOOLEAN NOT NULL DEFAULT FALSE,
    membership_type_code        VARCHAR(16) NOT NULL DEFAULT 'community',
    membership_date             TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW(),
    removal_date                TIMESTAMP WITHOUT TIME ZONE,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT pda_member_u_telegram_username UNIQUE (telegram_username),
    CONSTRAINT pda_member_u_polkadot_address UNIQUE (polkadot_address),
    CONSTRAINT pda_member_u_kusama_address UNIQUE (kusama_address),
    CONSTRAINT pdao_member_fk_membership_type
        FOREIGN KEY (membership_type_code)
            REFERENCES pdao_membership_type (code)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);