# screenlock-to-systemd

Whenever the cinnamon screensaver locks or unlocks the screen, a DBus
signal is emitted. When this happens, `screenlock-to-systemd` starts
the (user defined) systemd user targets `lock-activated.target` or
`unlock-activated.target`.

My use case for this is to mute sound from my machine when I am away
from it.

This is based on the same concepts as
[systemd-lock-handler][https://github.com/WhyNotHugo/systemd-lock-handler],
but instead of activating the screen saver, it reacts to the screen
being locked.
