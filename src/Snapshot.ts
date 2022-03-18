import * as request from 'request-promise';

interface Validator {
  operator_address: `terravaloper1${string}`,
  consensus_pubkey: {
    type: 'tendermint/PubKeyEd25519',
    value: string
  },
  status          : number,
  tokens          : string,
  delegator_shares: string,
  description     : {
    moniker: string,
    identity: string,
    website: string,
    details: string
  },
  unbonding_time: string,
  commission: {
    commission_rates: {
      rate: string,
      max_rate: string,
      max_change_rate: string
    },
    update_time: string
  },
  min_self_delegation: string
}

// type TerraDelegator = `terra1${string}`;
// type TerraValidator = `terravaloper1${string}`;

interface Delegation {
  delegation: {
    delegator_address: string
    validator_address: string
    shares           : string;
  },
  balance: {
    denom : `u${string}`;
    amount: string;
  };
}

type DelegationsDict = Record<string, bigint>

class Snapshot {
  URL     : string;
  noheight: boolean;
  snapshot: DelegationsDict

  constructor(
    URL: string,
    publicInfra = false,
    noheight = false
  ) {
    this.URL = URL;
    this.noheight = noheight;
    this.snapshot = {}
    if (publicInfra) {
      this.URL = 'https://lcd.terra.dev'
    }
  }

  async takeSnapshot(block: number): Promise<DelegationsDict> {


    const validators_resp = await request.get(`${this.URL}/staking/validators?height=${block}&limit=500`, { timeout: 10000000 })
    const validators: Validator[] = JSON.parse(validators_resp)['result'];

    // Sort the validators by stake
    const val_sorted = validators.sort((a, b) => {
      if (BigInt(a.tokens) > BigInt(b.tokens)) { return 1 }
      else if (BigInt(b.tokens) > BigInt(a.tokens)) { return -1 }
      else return 0
    })

    // Remove ten topmost
    const non_ten_topmost = val_sorted.slice(0, val_sorted.length - 10)
    for (let i = 0; i < non_ten_topmost.length; i++) {
      const operator_addr = validators[i]['operator_address'];

      console.log(`Awaiting delegators of ${operator_addr}`);
      const delegations: Delegation[] = JSON.parse(await request.get(`${this.URL}/staking/validators/${operator_addr}/delegations`))['result'];

      // Merge delegators of a given valoper into the global dict
      delegations.reduce(
        (acc: DelegationsDict, cur: Delegation) => {
          const delegator = cur.delegation.delegator_address
          const balance   = cur.balance.amount

          if (!Object.keys(acc).includes(delegator)) {
            acc[delegator] = BigInt(0)
          }

          acc[delegator] += BigInt(balance)
          return this.snapshot
        }, this.snapshot
      )
    }
    return this.snapshot
  }

}

export = Snapshot;
