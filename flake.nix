{
  description = "MCP server for accessing Rust crate documentation via docs.rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        craneLib = crane.mkLib pkgs;

        # Common arguments shared between all builds
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            zstd
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin-specific inputs can be added here
            pkgs.libiconv
          ];
        };

        # Build only the cargo dependencies
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate
        docsrs-mcp = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          # The binary should be named docsrs-mcp
          cargoExtraArgs = "--bin docsrs-mcp";
        });
      in
      {
        packages = {
          default = docsrs-mcp;
          inherit docsrs-mcp;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from the package
          inputsFrom = [ docsrs-mcp ];

          # Additional dev tools
          packages = with pkgs; [
            rust-analyzer
            rustfmt
            clippy
            cargo-watch
          ];
        };

        # Run checks
        checks = {
          inherit docsrs-mcp;

          # Run clippy
          docsrs-mcp-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          # Check formatting
          docsrs-mcp-fmt = craneLib.cargoFmt {
            inherit (commonArgs) src;
          };
        };
      });
}
