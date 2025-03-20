CREATE TABLE IF NOT EXISTS pdao_settings
(
    key                         VARCHAR(256) PRIMARY KEY,
    value                       VARCHAR NOT NULL,
    created_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now()
);