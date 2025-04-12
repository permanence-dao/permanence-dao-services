CREATE TABLE IF NOT EXISTS pdao_member_vote
(
    id            SERIAL PRIMARY KEY,
    vote_id       INT NOT NULL,
    cid           VARCHAR(64) NOT NULL,
    network_id    INT NOT NULL,
    referendum_id INT NOT NULL,
    index         INT NOT NULL,
    address       VARCHAR(128) NOT NULL,
    feedback      TEXT NOT NULL,
    created_at    TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT pda_member_vote_u_member_vote UNIQUE (vote_id, address),
    CONSTRAINT pdao_member_vote_fk_vote
        FOREIGN KEY (vote_id)
            REFERENCES pdao_vote (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE,
    CONSTRAINT pdao_member_vote_fk_referendum
        FOREIGN KEY (referendum_id)
            REFERENCES pdao_referendum (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS pdao_member_vote_idx_vote_id
    ON pdao_member_vote (vote_id);
CREATE INDEX IF NOT EXISTS pdao_member_vote_idx_address
    ON pdao_member_vote (address);