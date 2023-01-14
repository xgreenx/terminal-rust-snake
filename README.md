## Terminal Snake to Unravel the message

The base code from the [rs-snake](https://github.com/baurst/rs_snake) repository but with modified game logic. 
The goal is to eat all black squares to reveal the hidden message.

```shell
snake 0.3.0
Author: Green
Almost a classic snake game for your terminal. You need to eat black squares, and you will reveal something. The game
saves the state on exit and loads on start(if you didn't set the '--new' flag). You don't need to be afraid of dying
because you can't die=) The game will continue but with a shorter snake. If the default speed is to hight or low for
you, then you can change is with the '--speed {number of fps}' flag. If you are tired of the game and want to see the
final result, you can specify the '--reveal' flag.

USAGE:
    rs_snake [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -n, --new        starts a new game
    -r, --reveal     reveal the message without game
    -V, --version    Prints version information

OPTIONS:
    -s, --speed <speed>    the speed of the game in fps [default: 30]
```