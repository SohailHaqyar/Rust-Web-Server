-- Your SQL goes here
create table memes (
  id serial primary key,
  name varchar(255) not null,
  image_path varchar(255) not null
)