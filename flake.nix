# adapted from https://github.com/loophp/rust-shell/blob/main/flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOs/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" ];
      perSystem = { config, pkgs, system, ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };
          prepareEnv = { version, profile }:
            let
              rust = pkgs.rust-bin.${version}.latest.${profile}.override {
                extensions = [ "rust-src" ];
              };
              diesel = pkgs.diesel-cli.override {
                sqliteSupport = false;
                mysqlSupport = false;
              };
            in {
              name = "rust-" + version + "-" + profile;
              path = "${rust}/lib/rustlib/src/rust/library";

              drvs = [
                pkgs.just
                pkgs.openssl
                pkgs.pkg-config
                pkgs.rust-analyzer
                pkgs.postgresql
                pkgs.mold
                rust
                diesel
                pkgs.cargo-nextest
                pkgs.clang
              ];
            };
          matrix = {
            stable-default = {
              version = "stable";
              profile = "default";
            };
            stable-minimal = {
              version = "stable";
              profile = "minimal";
            };
            unstable-default = {
              version = "unstable";
              profile = "default";
            };
            unstable-minimal = {
              version = "unstable";
              profile = "minimal";
            };
          };
        in {
          formatter = pkgs.nixfmt;
          devShells = builtins.mapAttrs (name: value:
            let
              version = value.version;
              profile = value.profile;
              rustInfo = prepareEnv { inherit version profile; };
            in pkgs.mkShell {
              name = rustInfo.name;
              RUST_SRC_PATH = rustInfo.path;
              buildInputs = rustInfo.drvs;
            }) matrix // {
              default = let
                version = matrix.stable-default.version;
                profile = matrix.stable-default.profile;
                rustInfo = prepareEnv { inherit version profile; };
              in pkgs.mkShell {
                name = rustInfo.name;
                RUST_SRC_PATH = rustInfo.path;
                buildInputs = rustInfo.drvs;
                shellHook = ''
                  ./scripts/init_db.sh
                  ./scripts/init_redis.sh
                '';
              };
            };

        };
    };
}
