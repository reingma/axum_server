# axum_server
Newsletter delivery toy api. 
Based on the book https://www.zero2prod.com. With the following changes:
  - Using axum instead of actix as the http server.
  - Using diesel instead of sqlx for the postgres connection
  - Using the async version of diesel
  - Using tera for html rendering
  - Using tower middleware layer and tracing
  - Added a nix flake for starting up whole enviroment using direnv
Besides some natural code structuring changes 
Credentials for first login:
username: admin
password: adminpassword
