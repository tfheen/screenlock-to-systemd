# screenlock-to-systemd

Whenever the cinnamon screensaver locks or unlocks the screen, a DBus signal is emitted. When this happens, `screenlock-to-systemd` starts the (user defined) systemd user targets `lock-activated.target` or `unlock-activated.target`.

My use case for this is to mute sound from my machine when I am away from it.

This is based on the same concepts as [systemd-lock-handler](https://github.com/WhyNotHugo/systemd-lock-handler), but instead of activating the screen saver, it reacts to the screen being locked.

To install and enable screenlock-to-systemd to run on login do the
following:

- build the binary: `cargo install --path .`
- copy `screenlock-to-systemd.service` (possibly adjusting the `ExecStart` line), `unlock-activated.target`, and `lock-activated.target` units into `~/.config/systemd/user/`.
- reload `systemd`: `systemctl --user daemon-reload`
- enable the service using `systemctl --user enable screenlock-to-systemd.service`

Two sample units are provided: `pulsemixer-mute.service` and `pulsemixer-unmute.service`. Those are installed and enabled in the same fashion.

## Limitations

Currently only listens for the signal from the cinnamon screensaver. This should be trivial to extend for other screen savers.
