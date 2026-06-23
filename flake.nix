# Note this is just a draft, not properly tested or documented yet

{
    description = "A non-linear writing instrument.";
    inputs.nixpkgs.url = "nixpkgs/nixos-26.05";

    outputs = { nixpkgs, self }: let
        name = "en";
        version = "0.4.0-alpha";

        supportedSystems = [
            "x86_64-linux"
        ];

        forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
        nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; });

    in {
        packages = forAllSystems (system: let pkgs = nixpkgsFor.${system};
        in {
            default = pkgs.rustPlatform.buildRustPackage {
                inherit name version;
                src = ./.;
                nativeBuildInputs = with pkgs; [
                    just
                ];
                dontUseJustInstall = true;
                dontUseJustBuild = true;
                cargoHash =
                    "sha256-U6jf48KmqcdCBYKyZhYPpiv5qj98SjlNqw8Ii2mJn0I=";
            };
        });

        devShells = forAllSystems (system:
            let
                pkgs = nixpkgsFor.${system};
                sets = with pkgs; rec {
                    base = [
                        cargo-deny
                        cargo-llvm-cov
                        cargo-msrv
                        git
                        just
                        rustup
                        taplo
                        typos
                    ];
                    cross = base ++ [
                        cargo-xwin
                        wine
                    ];
                    dev = [
                        cargo-mutants
                        watchexec
                    ];
                    edit = dev ++ [
                        neovim
                        tmux
                    ];
                };
            in with sets; {
                default = pkgs.mkShell { buildInputs = base; };
                cross = pkgs.mkShell { buildInputs = cross; };
                dev = pkgs.mkShell { buildInputs = base ++ dev; };
                cross-dev = pkgs.mkShell { buildInputs = cross ++ dev; };
                edit = pkgs.mkShell { buildInputs = edit; };
                cross-edit = pkgs.mkShell { buildInputs = cross ++ edit; };
            }
        );

        nixosModules.en = { config, lib, system, ... }:
            {
                options.within.services.en = {
                    enable = lib.mkEnableOption "enable en server";
                    port = lib.mkOption {
                        description = "Port to serve your graph on";
                        defaultText = "A random port chosen by the OS";
                        type = lib.types.port;
                    };
                    host = lib.mkOption {
                        description = "Address or hostname to serve your graph on";
                        default = "0.0.0.0";
                        type = lib.types.str;
                    };
                    graph = lib.mkOption {
                        description = "File path pointing to your graph file";
                        default = "./static/graph.toml";
                        type = lib.types.path;
                    };
                };

                config =
                    lib.mkIf config.within.services.en.enable {
                        systemd.services.en = {
                            wantedBy = [ "multi-user.target" ];
                            serviceConfig = {
                                Restart = "always";
                                ExecStart = let
                                        user_opts = config.within.services.en;
                                    in
                                    "${self.packages.${system}.default}/bin/en"
                                    + " --port ${user_opts.port}"
                                    + " --host ${user_opts.host}"
                                    + " --graph ${user_opts.graph}";
                            };
                        };
                    };
            };

    };
}
