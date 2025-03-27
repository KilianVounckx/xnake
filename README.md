# XNAKE

Simple game based on the classic Snake, with extra food types. Click
[here](https://kilianvounckx.github.io/xnake) to try out.

Written in Rust using [macroquad](https://macroquad.rs/).

## Gameplay

The basic gameplay is the same as in snake: move around with arrow keys, WASD,
or swiping on mobile. Try to eat the food without going outside of the grid or
going over yourself. Eating food will grow the snake.
The difference is the extra food types. Each type has a different color (I
haven't had the time to make proper assets). The red food is normal food. All
other food has a temporary en will despawn if not eaten in time:

| Inside | Border | Effect |
|--------|--------|--------|
| red | green | eating food will spawn two more |
| dark gray | gray | the snake's length is halved |
| green | dark green | the snake's speed is halved |
| gold | white | the snake's speed is doubled |
| orange | blue | the snake's tail becomes its head and vice-versa |
| white | light gray | the snake can pass through itself |
| dark blue | gold | the snake can wrap around the grid |
