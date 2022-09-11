create table todo
(
    owner      text not null,
    content    text not null,
    created_at timestamp with time zone default now() not null
);
