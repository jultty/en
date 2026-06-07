{
    description = "A non-linear writing instrument.";

    inputs.nixpkgs.url = "nixpkgs/nixos-26.05";

    outputs = { nixpkgs, self }: let
        name = "en";
        version = "0.4.0";

        supportedSystems = [
            "x86_64-linux"
            "aarch64-linux"
        ];

        forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

        nixpkgsFor = forAllSystems (system: import nixpkgs {
            inherit system;
        });

    in {
        packages = forAllSystems (system: let
            pkgs = nixpkgsFor.${system};
        in {
            default = pkgs.rustPlatform.buildRustPackage {
                inherit name version;
                src = ./.;

                cargoHash =
                    "sha256-"
                    + "em229cShq/IShRnxlp5mgcIu7pIOf0LflV8Pw0lLUEY=";
            };
        });

        apps = forAllSystems (system: {
            default = {
                type = "app";
                program =
                    "${self.packages.${system}.default}/bin/en";
            };
        });

        devShells = forAllSystems (system:
            let pkgs = nixpkgsFor.${system}; in {
                default = pkgs.mkShell {
                    buildInputs = with pkgs; [
                        rustup
                        just
                        watchexec
                        cargo-deny
                        cargo-llvm-cov
                        cargo-mutants
                        go-tools
                        typos
                        taplo
                    ];
                };
            }
        );


        nixosModules.bot = { config, lib, system, ... }: {
            options.within.services.en.enable =
                lib.mkEnableOption "enable en server";

            config = lib.mkIf config.within.services.en.enable {
                systemd.services.en = {
                    wantedBy = [ "multi-user.target" ];
                    serviceConfig = {
                        Restart = "always";
                        ExecStart =
                            "${self.packages."${system}".default}/bin/en";
                    };
                };
            };
        };


    };
}
