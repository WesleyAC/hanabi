# hanabi

hi! this is a pretty ok implementation of the cooperative card game "[hanabi](https://en.wikipedia.org/wiki/Hanabi_(card_game))," which is probably the best card or board game ever made. you can see what it looks like and play with your friends at [hanabi.site](https://hanabi.site).

why another implementation of hanabi? there's a bunch of things i don't like about current implementations that i've found:

* they require creating accounts.
* they are difficult to figure out how to use (i failed to even start a game on [boardgamearena](https://boardgamearena.com/)).
* they're often very slow ([hanabi.live](https://hanabi.live/) managed to render my friends computer almost unusable).
* they calculate and track too much information for you, pushing you to play hanabi as a very explicitly convention based game, which is not how i like to play.

this implementation is currently pretty rough around the edges, but i think it's currently better than existing sites on the axes that i care about, so maybe you'll enjoy it as well <3

## TODO

- [ ] allow reordering cards in hand
- [ ] add audible cues when people play
- [ ] clean up UX around playing/discarding cards on a slow internet connection
- [ ] add rainbow variant
- [ ] make game names random
- [ ] make the name input modal nicer
- [ ] fix bug with spaces in names
- [ ] make play drop target nicer
- [ ] make it clearer which drop target is play and which is discard
