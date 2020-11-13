# Airdrop

## How to use


### Install package
```
$ npm install @mirror-protocol/mirror-airdrop
```

### Create distribution list
`airdrop.json`
```json
{
  "accounts": [
    {
      "address": "terra1qfqa2eu9wp272ha93lj4yhcenrc6ymng079nu8",
      "amount": "1000000"
    },
    {
      "address": "terra1ucp369yry6n70qq3zaxyt85cnug75r7ln8l6se",
      "amount": "2000000"
    },
    {
      "address": "terra1t849fxw7e8ney35mxemh4h3ayea4zf77dslwna",
      "amount": "3000000"
    },
    ...
  ]
}
```

### Get proof with user input
```javascript
import { accounts } from "../airdrop.json";

const airdrop = new Airdrop(accounts);
const proof = airdrop.getMerkleProof(accounts[3]);

console.log("Merkle Root", airdrop.getMerkleRoot());
console.log("Merkle Proof", proof);
console.log("Target Acc", accounts[3]);
console.log("Verified", airdrop.verify(proof, accounts[3]));
```

## How to build contract
```
$ docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_airdrop",target=/code/contracts/airdrop/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.10.2 ./contract
```
