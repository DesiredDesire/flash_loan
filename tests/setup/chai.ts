import chai from "chai";
import chaiAsPromised from "chai-as-promised";
import type { AccountId } from "@polkadot/types/interfaces";
import BN from "bn.js";
import { flush, proxy } from "tests/soft-assert";
const softExpect = proxy(chai.expect);

interface ExpectStaticWithSoft extends Chai.ExpectStatic {
  soft: (val: any, message?: string) => Chai.Assertion;
  flushSoft: () => void;
}
declare global {
  export namespace Chai {
    interface Assertion {
      output(
        value:
          | AccountId
          | string
          | number
          | boolean
          | string[]
          | number[]
          | unknown,
        msg?: string
      ): void;
      almostEqualOrEqualNumberE12<TData extends BN | number | string>(
        expected: TData
      ): void;
      equalUpTo1Digit<TData extends BN | number | string>(
        expected: TData
      ): void;
    }
  }
}

chai.use(chaiAsPromised);

// eslint-disable-next-line @typescript-eslint/no-unused-vars
chai.use((c, utils) => {
  c.Assertion.addMethod("output", async function (param, message) {
    await new c.Assertion(this._obj).to.eventually.have
      .property("output")
      .to.equal(param, message);
  });
});
chai.config.truncateThreshold = 0;
chai.use(
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  require("chai-formatter-monkeypatch")(function (obj) {
    return `:\n${JSON.stringify(obj, null, 2)}`;
  })
);

const expectWithSoft = chai.expect as ExpectStaticWithSoft;
expectWithSoft.soft = function (val: any, message?: string) {
  return softExpect(val, message);
};
expectWithSoft.flushSoft = flush;

export const expect = expectWithSoft;
