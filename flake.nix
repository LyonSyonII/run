{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs = { self, fenix, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
    in
    with pkgs;
    {
      packages.x86_64-linux.fenix = fenix.packages.x86_64-linux.minimal.toolchain;

      devShells.${system}.default = mkShell {
        inherit nixpkgs;
        buildInputs = with pkgs.buildPackages; [
          (fenix.packages.${system}.complete.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
            "miri"
            "rustc-codegen-cranelift-preview"
            "rust-analyzer"
          ])
          nodePackages.pnpm
          nodejs
          python3Minimal
          sccache
        ];
/*         shellHook = ''
          rustup toolchain install nightly-2024-01-20 -c miri rustc-codegen-cranelift-preview rust-src rust-analyzer rustfmt clippy
          rustup default nightly-2024-01-20
        ''; */
      };
    };
}
