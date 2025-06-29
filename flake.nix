{
  description = "MCP server for accessing Rust crate documentation via docs.rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = {
          default = self.packages.${system}.docsrs-mcp;

          docsrs-mcp = pkgs.rustPlatform.buildRustPackage {
            pname = "docsrs-mcp";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            buildInputs = with pkgs; [
              zstd
            ];

            # The binary should be named docsrs-mcp
            cargoBuildFlags = [ "--bin" "docsrs-mcp" ];

            meta = with pkgs.lib; {
              description = "MCP server for accessing Rust crate documentation via docs.rs";
              homepage = "https://github.com/maan2003/docsrs-mcp";
              license = with licenses; [ mit asl20 ];
              maintainers = [ ];
              mainProgram = "docsrs-mcp";
            };
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.docsrs-mcp ];

          packages = with pkgs; [
            rust-analyzer
            rustfmt
            clippy
            cargo-watch
          ];
        };
      });
}
