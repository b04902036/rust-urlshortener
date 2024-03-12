DROP TABLE IF EXISTS url;
CREATE TABLE url (
    id                  BIGINT NOT NULL AUTO_INCREMENT,
    origin              VARCHAR(2048) NOT NULL,
    short               VARCHAR(16) NOT NULL,
    expire_at_secs      BIGINT NOT NULL,
    created_at_secs     BIGINT NOT NULL,
    deleted_at_secs     BIGINT DEFAULT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uniq_short(short),
    KEY (expire_at_secs)
);

