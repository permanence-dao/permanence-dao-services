CREATE TABLE IF NOT EXISTS pdao_pending_member_vote
(
    id            SERIAL PRIMARY KEY,
    cid           VARCHAR(64) NOT NULL,
    network_id    INT NOT NULL,
    referendum_id INT NOT NULL,
    index         INT NOT NULL,
    address       VARCHAR(128) NOT NULL,
    vote          BOOLEAN,
    feedback      TEXT NOT NULL,
    created_at    TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT pda_pending_member_vote_u_member_vote UNIQUE (referendum_id, address),
    CONSTRAINT pdao_pending_member_vote_fk_network
        FOREIGN KEY (network_id)
            REFERENCES pdao_network (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT pdao_pending_member_vote_fk_referendum
        FOREIGN KEY (referendum_id)
            REFERENCES pdao_referendum (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS pdao_pending_member_vote_idx_referendum_id
    ON pdao_pending_member_vote (referendum_id);

CREATE INDEX IF NOT EXISTS pdao_pending_member_vote_idx_referendum_id_address
    ON pdao_pending_member_vote (referendum_id, address);