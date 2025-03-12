-- Postgres SQL

-- ---------------------------
-- Begin - ETH_EVENT -
-- ---------------------------

create table ETH_EVENT (
    ID bigserial primary key,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSONB
);

CREATE INDEX ETH_EVENT_INDEX_EVENT_TYPE ON ETH_EVENT( (DATA->>'event_type') );

-- End - ETH_EVENT -
