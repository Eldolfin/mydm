(import ./lib.nix) rec {
  name = "launch";
  testScript = ''
    c.wait_for_x()
    c.screenshot("${name}")
  '';
}
