{
  rustPlatform,
  lib,
}:
rustPlatform.buildRustPackage {
  pname = "narisha";
  version = "1.0.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  meta = with lib; {
    license = licenses.gpl3;
    description = "A window manager for the River compositor";
    homepage = "https://git.agahnim.dev/Agahnim/Narisha";

    maintainers = [ ];
    platforms = [ platforms.linux ];
  };
}
