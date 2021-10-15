import * as request from 'request-promise';

class Snapshot {
  URL: string;

  constructor(URL: string) {
    this.URL = URL;
  }

  async takeSnapshot(block: number): Promise<{ [delegator: string]: bigint }> {
    const delegationSnapshot: { [delegator: string]: bigint } = {};
    const validators = JSON.parse(
      await request.get(`${this.URL}/staking/validators?height=${block}&limit=99999999`, {
        timeout: 10000000
      })
    )['result'];

    for (let i = 0; i < validators.length; i++) {
      const operator_addr = validators[i]['operator_address'];
      const delegators: Array<{
        delegation: {
          delegator_address: string;
          validator_address: string;  
          shares: string;
        };
        balance: {
          denom: string;
          amount: string;
        };
      }> = JSON.parse(
        await request.get(
          `${this.URL}/staking/validators/${operator_addr}/delegations?height=${block}&limit=99999999`
        )
      )['result'];

      delegators.forEach((del) => {
        if (delegationSnapshot[del.delegation.delegator_address] === undefined) {
          delegationSnapshot[del.delegation.delegator_address] = BigInt(0);
        }

        delegationSnapshot[del.delegation.delegator_address] += BigInt(
          del.balance.amount
        );
      });
    }

    return delegationSnapshot;
  }
}

export = Snapshot;
