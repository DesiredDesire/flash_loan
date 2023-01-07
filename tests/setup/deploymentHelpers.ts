import { ApiPromise } from "@polkadot/api";
import { CodePromise, ContractPromise } from "@polkadot/api-contract";
import { CodeSubmittableResult } from "@polkadot/api-contract/base";
import { KeyringPair } from "@polkadot/keyring/types";
import type { Weight } from "@polkadot/types/interfaces";

import { readFileSync } from "fs-extra";

import FlashLoanReceiverMock from "typechain/contracts/flash_loan_receiver_mock";
import FlashLoanContract from "typechain/contracts/flash_loan_contract";
import FlashLoanSimple from "typechain/contracts/flash_loan_simple";
import PSP22Mintable from "typechain/contracts/psp22_mintable";

import { apiProviderWrapper } from "./helpers";
import { AccountId } from "typechain/types-arguments/flash_loan_contract";
import BN from "bn.js";

const getCodePromise = (api: ApiPromise, contractName: string): CodePromise => {
  const abi = JSON.parse(
    readFileSync(`./artifacts/${contractName}.json`).toString()
  );
  const wasm = readFileSync(`./artifacts/${contractName}.wasm`);

  return new CodePromise(api, abi, wasm);
};
export const setupContract = async (
  owner: KeyringPair,
  contractName: string,
  constructorName: string,
  ...constructorArgs: any[]
) => {
  const api = await apiProviderWrapper.getAndWaitForReady();
  const codePromise = getCodePromise(api, contractName);
  // maximum gas to be consumed for the instantiation. if limit is too small the instantiation will fail.
  const gasLimit = 100000n * 1000000n;
  const gasLimitFromNetwork = api.consts.system.blockWeights
    ? (api.consts.system.blockWeights as unknown as { maxBlock: Weight })
        .maxBlock
    : (api.consts.system.maximumBlockWeight as unknown as Weight);
  // a limit to how much Balance to be used to pay for the storage created by the instantiation
  // if null is passed, unlimited balance can be used
  const storageDepositLimit = null;
  // used to derive contract address,
  // use null to prevent duplicate contracts
  const salt = new Uint8Array();
  // balance to transfer to the contract account, formerly know as "endowment".
  // use only with payable constructors, will fail otherwise.
  const value = (
    await apiProviderWrapper.getAndWaitForReady()
  ).registry.createType("Balance", 1000);

  const deployedContract = await new Promise<ContractPromise>(
    (resolve, reject) => {
      let unsub: () => void;
      const tx = codePromise.tx[constructorName](
        {
          storageDepositLimit: null,
          gasLimit: gasLimitFromNetwork.muln(2).divn(10),
          salt: undefined,
          value: undefined,
        },
        ...constructorArgs
      );
      tx.signAndSend(owner, (result: CodeSubmittableResult<"promise">) => {
        const { status, dispatchError, contract } = result;
        if (status.isInBlock) {
          if (dispatchError || !contract) {
            reject(dispatchError?.toString());
          } else {
            resolve(contract);
          }

          unsub();
        }
      })
        .then((_unsub) => {
          unsub = _unsub;
        })
        .catch(reject);
    }
  );

  return { owner, deployedContract };
};

const deployWithLog = async <T>(
  deployer: KeyringPair,
  constructor: new (
    address: string,
    signer: KeyringPair,
    nativeAPI: ApiPromise
  ) => T,
  contractName: string,
  ...deployArgs
) => {
  const ret = await setupContract(deployer, contractName, "new", ...deployArgs);
  if (process.env.DEBUG)
    console.log(
      `Deployed ${contractName}: ${ret.deployedContract.address.toString()}`
    );
  return getContractObject<T>(
    constructor,
    ret.deployedContract.address.toString(),
    ret.owner
  );
};

export const deployPSP22Mintable = async (
  deployer: KeyringPair,
  name: string,
  decimals: number = 6
) => {
  return deployWithLog(
    deployer,
    PSP22Mintable,
    "psp22_mintable",
    name,
    `Reserve ${name} token `,
    decimals
  );
};

export const deployFlashLoanContract = async (
  owner: KeyringPair,
  fees_e6: [AccountId, BN | number | string][],
  allowed: AccountId[],
  free: AccountId[]
) => {
  return deployWithLog(
    owner,
    FlashLoanContract,
    "flash_loan_contract",
    fees_e6,
    allowed,
    free
  );
};
export const deployFlashLoanSimple = async (owner: KeyringPair) => {
  return deployWithLog(owner, FlashLoanSimple, "flash_loan_simple");
};

export const deployFlashLoanReceiverMock = async (deployer: KeyringPair) => {
  return deployWithLog(
    deployer,
    FlashLoanReceiverMock,
    "flash_loan_receiver_mock"
  );
};

export const getContractObject = async <T>(
  constructor: new (
    address: string,
    signer: KeyringPair,
    nativeAPI: ApiPromise
  ) => T,
  contractAddress: string,
  signerPair: KeyringPair
) => {
  return new constructor(
    contractAddress,
    signerPair,
    await apiProviderWrapper.getAndWaitForReady()
  );
};

const getEntryOrThrow = <T>(record: Record<string, T>, key: string) => {
  if (!(key in record))
    throw new Error(`Key "${key}" not found in record ${record}`);
  const value = record[key];
  return value;
};
