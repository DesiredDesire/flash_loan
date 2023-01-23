# flash_loan

in src/trait one can find definitions of FlashLoan and FlashLoanReceiver traits.
in src/impl the default implementation of FlashLoan is written.
in src/contracts one can find:
  - flash_loan_simple -- a very simple usage example of flash loan with no overridden functions
  - flash_loan_contract -- a simple usage examplse of flash loan with some adjustments done by overriding functions
  - flash_loan_receiver_mock -- a contract that implemets FlashLoanReceiver trait and is used for testing
  - PSP22Mintable -- a PSP22 token with no access controlled mint method. It is used for testing.
in tests one can find flash loan tests.

to build run:
0. have npm and yarn installed
1. run: yarn install
2. run: yarn build

to test run:
0. build
1: run: yarn test
