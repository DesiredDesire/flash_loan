import { KeyringPair } from "@polkadot/keyring/types";
import BN from "bn.js";
import FlashLoanReceiverMock from "typechain/contracts/flash_loan_receiver_mock";
import FlashLoanContract from "typechain/contracts/flash_loan_contract";
import FlashLoanSimple from "typechain/contracts/flash_loan_simple";
import PSP22Mintable from "typechain/contracts/psp22_mintable";
import {
  deployFlashLoanReceiverMock,
  deployPSP22Mintable,
  deployFlashLoanContract,
  deployFlashLoanSimple,
} from "./setup/deploymentHelpers";
import { expect } from "./setup/chai";
import { getSigners } from "./setup/helpers";
import { ApiProviderWrapper } from "./setup/ApiProviderWrapper";
import { FlashLoanContractErrorBuilder } from "typechain/types-returns/flash_loan_contract";
import {
  FlashLoanErrorBuilder,
  PSP22ErrorBuilder,
} from "typechain/types-arguments/flash_loan_contract";
import { FlashLoanReceiverErrorBuilder } from "typechain/types-arguments/flash_loan_receiver_mock";

const E6 = Math.pow(10, 6);
const E12 = Math.pow(10, 12);

describe("Flash Loan tests. Preparing Env", () => {
  let owner: KeyringPair;
  let user: KeyringPair;
  let deployer: KeyringPair;
  let random: KeyringPair;
  let flashLoanReceiver: FlashLoanReceiverMock;
  let flashLoanContract: FlashLoanContract;
  let USDC: PSP22Mintable;
  let USDT: PSP22Mintable;
  let api: ApiProviderWrapper = new ApiProviderWrapper("ws://127.0.0.1:9944");
  let oneUSD;
  let milionUSD;
  beforeEach("preparing Env", async () => {
    owner = getSigners()[0];
    user = getSigners()[1];
    deployer = getSigners()[2];
    random = getSigners()[3];
    api.getAndWaitForReady();

    USDC = await deployPSP22Mintable(deployer, "USDC", 6);
    USDT = await deployPSP22Mintable(deployer, "USDT", 6);
    oneUSD = E6;
    milionUSD = E12;

    await USDC.tx.mint(owner.address, E12);
    await USDT.tx.mint(owner.address, E12);
    // console.log(
    //   `deployed tokens with addresses \n USDC : ${USDC.address} \n USDT : ${USDT.address}`
    // );
    flashLoanReceiver = await deployFlashLoanReceiverMock(deployer);
    // console.log(
    //   `deployed flashLoanReceiver with addresses : ${flashLoanReceiver.address}`
    // );
  });

  describe("Owner deploys FlashLoanContract with no allowed users and transfers 1milion of USDC and USDT to it. Then...", () => {
    beforeEach("", async () => {
      flashLoanContract = await deployFlashLoanContract(owner, [], [], []);
      // console.log(
      //   `deployed flashLoanContract with addresses : ${flashLoanContract.address}`
      // );
      await USDC.withSigner(owner).tx.transfer(
        flashLoanContract.address,
        milionUSD,
        []
      );
      await USDT.withSigner(owner).tx.transfer(
        flashLoanContract.address,
        milionUSD,
        []
      );
    });

    it("Owner call flash_loan and fails with as he is not on allowed list", async () => {
      await expect(
        flashLoanContract
          .withSigner(owner)
          .query.flashLoan(
            flashLoanReceiver.address,
            [USDC.address],
            [oneUSD],
            []
          )
      ).to.eventually.be.rejected.and.to.have.deep.property(
        "_err",
        FlashLoanErrorBuilder.Custom("NotAllowed")
      );
    });

    it("User call flash_loan and fails as he is not on allowed list", async () => {
      await expect(
        flashLoanContract
          .withSigner(user)
          .query.flashLoan(
            flashLoanReceiver.address,
            [USDC.address],
            [oneUSD],
            []
          )
      ).to.eventually.be.rejected.and.to.have.deep.property(
        "_err",
        FlashLoanErrorBuilder.Custom("NotAllowed")
      );
    });
  });

  describe("Owner deploys FlashLoanContract with allowed list = [user] and transfers 1milion of USDC and USDT to it. Then...", () => {
    beforeEach("", async () => {
      flashLoanContract = await deployFlashLoanContract(
        owner,
        [],
        [user.address],
        []
      );
      // console.log(
      //   `deployed flashLoanContract with addresses : ${flashLoanContract.address}`
      // );
      await USDC.withSigner(owner).tx.transfer(
        flashLoanContract.address,
        milionUSD,
        []
      );
      await USDT.withSigner(owner).tx.transfer(
        flashLoanContract.address,
        milionUSD,
        []
      );
    });

    it("User call flash_loan with wrong parameters, transaction fails ", async () => {
      await expect(
        flashLoanContract
          .withSigner(user)
          .query.flashLoan(
            flashLoanReceiver.address,
            [USDC.address],
            [oneUSD, oneUSD],
            []
          )
      ).to.eventually.be.rejected.and.to.have.deep.property(
        "_err",
        FlashLoanErrorBuilder.Parameters()
      );
    });

    it("User call flash_loan for 2 milions USDC, transaction fails ", async () => {
      await expect(
        flashLoanContract
          .withSigner(user)
          .query.flashLoan(
            flashLoanReceiver.address,
            [USDC.address],
            [2 * milionUSD],
            []
          )
      ).to.eventually.be.rejected.and.to.have.deep.property(
        "_err",
        FlashLoanErrorBuilder.PSP22Error(
          PSP22ErrorBuilder.InsufficientBalance()
        )
      );
    });

    it("User call flash_loan for 1 miolions USDC and 1 milion of USDT and succeeds", async () => {
      await expect(
        flashLoanContract
          .withSigner(user)
          .tx.flashLoan(
            flashLoanReceiver.address,
            [USDC.address, USDT.address],
            [milionUSD, milionUSD],
            []
          )
      ).to.eventually.be.fulfilled;
    });
  });

  describe("Owner deploys FlashLoanContract with allowed list = [user] and fees = [[USDC, 0], [USDT, 100_000 (10%)]], and transfers 1milion of USDC and USDT to it. Then...", () => {
    beforeEach("", async () => {
      flashLoanContract = await deployFlashLoanContract(
        owner,
        [
          [USDC.address, 0],
          [USDT.address, 100000],
        ],
        [user.address],
        []
      );
      // console.log(
      //   `deployed flashLoanContract with addresses : ${flashLoanContract.address}`
      // );
      await USDC.withSigner(owner).tx.transfer(
        flashLoanContract.address,
        milionUSD,
        []
      );
      await USDT.withSigner(owner).tx.transfer(
        flashLoanContract.address,
        milionUSD,
        []
      );
    });

    it("User setup malicious FlashLoanReciever that do not pay back fees. And ...\nUser call flash_loan for 1 miolions USDC and 1 milion of USDT", async () => {
      await flashLoanReceiver.tx.setSimulateBalanceToCoverFee(false);
      await expect(
        flashLoanContract
          .withSigner(user)
          .query.flashLoan(
            flashLoanReceiver.address,
            [USDC.address, USDT.address],
            [milionUSD, milionUSD],
            []
          )
      ).to.eventually.be.rejected.and.to.have.deep.property(
        "_err",
        FlashLoanErrorBuilder.PSP22Error(
          PSP22ErrorBuilder.InsufficientBalance()
        )
      );
    });

    it("User setup malicious FlashLoanReciever that do not give enough allowance. And ...\nUser call flash_loan for 1 miolions USDC and 1 milion of USDT", async () => {
      await flashLoanReceiver.tx.setCustomAmountToApprove(1);
      await expect(
        flashLoanContract
          .withSigner(user)
          .query.flashLoan(
            flashLoanReceiver.address,
            [USDC.address, USDT.address],
            [milionUSD, milionUSD],
            []
          )
      ).to.eventually.be.rejected.and.to.have.deep.property(
        "_err",
        FlashLoanErrorBuilder.PSP22Error(
          PSP22ErrorBuilder.InsufficientAllowance()
        )
      );
    });

    it("FlashLoanReciever when called will return Error. And ...\nUser call flash_loan for 1 miolions USDC and 1 milion of USDT", async () => {
      await flashLoanReceiver.tx.setFailExecuteOperation(true);
      await expect(
        flashLoanContract
          .withSigner(user)
          .query.flashLoan(
            flashLoanReceiver.address,
            [USDC.address, USDT.address],
            [milionUSD, milionUSD],
            []
          )
      ).to.eventually.be.rejected.and.to.have.deep.property(
        "_err",
        FlashLoanErrorBuilder.FlashLoanReceiverError(
          FlashLoanReceiverErrorBuilder.ExecuteOperationFailed()
        )
      );
    });

    it("User call flash_loan for 1 miolions USDC and 1 milion of USDT", async () => {
      await expect(
        flashLoanContract
          .withSigner(user)
          .query.flashLoan(
            random.address,
            [USDC.address, USDT.address],
            [milionUSD, milionUSD],
            []
          )
      ).to.eventually.be.rejected;
    });
  });
});
