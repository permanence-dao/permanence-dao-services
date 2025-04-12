CREATE TABLE IF NOT EXISTS pdao_vote
(
    id                      SERIAL PRIMARY KEY,
    network_id              INT NOT NULL,
    referendum_id           INT NOT NULL,
    index                   INT NOT NULL,
    block_hash              VARCHAR NOT NULL,
    block_number            BIGINT NOT NULL,
    extrinsic_index         INT NOT NULL,
    vote                    BOOLEAN,
    balance                 VARCHAR NOT NULL,
    conviction              INT NOT NULL,
    is_removed              BOOLEAN NOT NULL DEFAULT FALSE,
    subsquare_comment_cid   VARCHAR,
    subsquare_comment_index INT,
    has_coi                 BOOLEAN NOT NULL DEFAULT FALSE,
    is_forced               BOOLEAN NOT NULL DEFAULT FALSE,
    created_at              TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT pdao_vote_fk_network
        FOREIGN KEY (network_id)
            REFERENCES pdao_network (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT pdao_vote_fk_referendum
        FOREIGN KEY (referendum_id)
            REFERENCES pdao_referendum (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS pdao_vote_idx_referendum_id
    ON pdao_vote (referendum_id);

ALTER TABLE pdao_referendum
    ADD CONSTRAINT pdao_referendum_fk_vote
        FOREIGN KEY (last_vote_id)
        REFERENCES pdao_vote (id)
        ON DELETE RESTRICT
        ON UPDATE CASCADE;