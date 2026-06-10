# Raffle Minter

This contract allows nft collections to be minted in a raffle style.

## Design

- random set of raffle tickets selected to mint each epoch
- ticket purchases include selection of fund destination
- each epoch, one of fund destinations randomyl chosen to recieve that epoch pot
admin features decided:
  - % of tickets cost can be refunded
  - % of ticket sales to go to artists
  - min-max % of ticket cost can be dedicated to specific wallet (from choices of lists)
