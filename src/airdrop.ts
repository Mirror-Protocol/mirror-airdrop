import keccak256 from "keccak256";
import { MerkleTree } from "merkletreejs";

class Airdrop {
  private tree: MerkleTree;

  constructor(accounts: Array<{ address: string; amount: string }>) {
    const leaves = accounts.map((a) => keccak256(a.address + a.amount));
    this.tree = new MerkleTree(leaves, keccak256, { sort: true });
  }

  public getMerkleRoot(): string {
    return this.tree.getHexRoot();
  }

  public getMerkleProof(account: {
    address: string;
    amount: string;
  }): string[] {
    return this.tree.getHexProof(keccak256(account.address + account.amount));
  }

  public verify(
    proof: string[],
    account: { address: string; amount: string },
  ): boolean {
    let hashBuf = keccak256(account.address + account.amount);

    proof.forEach((proofElem) => {
      const proofBuf = Buffer.from(proofElem.replace("0x", ""), "hex");

      if (hashBuf < proofBuf) {
        hashBuf = keccak256(Buffer.concat([hashBuf, proofBuf]));
      } else {
        hashBuf = keccak256(Buffer.concat([proofBuf, hashBuf]));
      }
    });

    return this.tree.getHexRoot().replace("0x", "") === hashBuf.toString("hex");
  }
}

export = Airdrop;
