CREATE TABLE IF NOT EXISTS pdao_referendum
(
    id                          SERIAL PRIMARY KEY,
    network_id                  INT NOT NULL,
    track_id                    INT NOT NULL,
    index                       INT NOT NULL,
    status                      VARCHAR(128) NOT NULL,
    title                       TEXT,
    content                     TEXT,
    content_type                VARCHAR(128) NOT NULL,
    telegram_chat_id            BIGINT NOT NULL,
    telegram_topic_id           INT NOT NULL,
    telegram_intro_message_id   INT NOT NULL,
    opensquare_cid              VARCHAR(64) NOT NULL,
    opensquare_post_uid         VARCHAR(64) NOT NULL,
    last_vote_id                INT,
    message_archive             TEXT,
    is_terminated               BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT pdao_referendum_fk_network
        FOREIGN KEY (network_id)
            REFERENCES pdao_network (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT pdao_referendum_u_index UNIQUE (network_id, index)
);

CREATE INDEX IF NOT EXISTS pdao_referendum_idx_index
    ON pdao_referendum (index);
CREATE INDEX IF NOT EXISTS pdao_referendum_idx_network_id_index
    ON pdao_referendum (network_id, index);