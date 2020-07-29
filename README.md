# busy_beaver
Doing some busy beaver stuff in rust 4fun

want to run it?

in terminal:

-> cargo run --release <number of states> <number of symbols>
  
# examples

To get the one below, run

-> cargo run --release 2 2


States | Symbols
------------ | -------------
2 | 2

![Alt text](imgs/bb2state2symb.jpeg?raw=true "Title")

**score: 4**

--------------------------------------------------------

-> cargo run --release 3 2

States | Symbols
------------ | -------------
3 | 2

![Alt text](imgs/bb3state2symb.jpeg?raw=true "Title")

**score: 6**

--------------------------------------------------------

-> cargo run --release 2 3

States | Symbols
------------ | -------------
2 | 3


![Alt text](imgs/bb2state3symb.jpeg?raw=true "Title")


**score: 9**
