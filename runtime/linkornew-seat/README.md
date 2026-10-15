# LinkornEW session

Installed only on the `linkornew` image, under `/usr/lib/linkornew/`.
The FoFline packager does not copy this directory.

`linkornew-session` is the tty1 program. It refuses any other terminal,
points labwc at `config/labwc/autostart`, and that autostart runs
`linkornew-seat` only. `empiieros-shell-gui` is not started.
The console window is `console.c` in this directory, compiled with
`make`. The staff image installs that binary as
`/usr/lib/linkornew/console`. It is not a distro terminal package.

The staff image keeps getty@tty2 through getty@tty6 masked
(`systemd/*.service` → `/dev/null`). This session does not unmask them.
