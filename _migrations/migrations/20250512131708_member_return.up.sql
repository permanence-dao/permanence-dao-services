CREATE TABLE IF NOT EXISTS pdao_member_return
(
    id          SERIAL PRIMARY KEY,
    member_id   INT NOT NULL,
    created_at  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT pdao_member_return_fk_member
        FOREIGN KEY (member_id)
            REFERENCES pdao_member (id)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS pdao_member_return_idx_member_id
    ON pdao_member_return (member_id);
