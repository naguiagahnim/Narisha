{ rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "narisha";
  version = "1.0.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
}
