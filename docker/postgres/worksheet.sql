CREATE TABLE openid_connect_states (
  sid varchar(26) not null primary key,
  state varchar(26) not null
)