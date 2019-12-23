The PolPatrol tool
==================

In this section, we present the overall system model of the PolPatrol tool. This system model defines how PolPatrol interacts with different relay-chain runtimes under test, the type of trace information it collects during test execution, and the type of properties that PolPatrol supports. We also highlight the limitations of PolPatrol and highlight possible extensions.


High-level overview
-------------------

There are two main components in PolPatrol's system model:

1. Relay-chain runtime under test: This is the Wasm blob of the runtime that PolPatrol will test for performance and correctness issues.
2. Runtime environment: This is the environment that interacts with the relay-chain runtime under test.

The test execution follows the following steps:

1. *Test generation*: First, PolPatrol generates inputs for the relay-chain runtime under test. The critical functionality for relay-chain runtimes is block validation. To test this functionality while achieving high code coverage, PolPatrol must generate well-formed blocks and submit these as input to the relay-chain runtime under test. In particular, the blocks must contain different extrinsics that cover different parts of the relay-chain runtime.

2. *Instrumented test execution*: To evaluate the relay-chain runtime under test, PolPatrol executes the generated test inputs in an instrumented environment that logs all calls to the environment along with information about resource consumption. These interactions allow the extractions of different information including memory usage, storage consumption, and the number of sent messages.



Test generation
---------------

PolPatrol generates test inputs using its generation module. The generation module allows one to add extrinsics which will be executed against the relay-chain runtime environment. PolPatrol generates, signs these extrinsics (if required) and assembles them in blocks until no extrinsics are left. A minimum block number can be specified per extrinsic to ensure that an extrinsic is not included until a given block number. 

To add extrinsics to the blocks, PolPatrol relies on hand-crafted tests which are manually specified with the goal to execute relevant relay-chain runtime components (such as `Balances`).

Complementary, PolPatrol also supports two kinds of simple fuzzing. (1) generation of random calldata and (2) generation of randomly initialized Headers and Blocks (using extrinsics of the tx pool) which are then properly encoded before being passed as calldata.

We explain these below.

Hand-crafted tests
~~~~~~~~~~~~~~~~~~

PolPatrol's hand-crafted tests generate extrinsics that test the following list of modules:

- Balances: Multiple transfer scenarios (successful/unsuccessful) covering all 3x functions, including calls from root origin.

- Claim: One example is presented.

- Slots: Auctioning off parachain slots for a certain lease duration. A bid to an auction is placed. The `Adder` parachain runtime is used as an example to onboard a new parachain and set its head data.

- ImOnline: A heartbeat is crafted.

- FinalityTracker: A successful and a non-successful is available.

- Sudo: The sudo module is a helper module to dispatch root origin calls in runtimes to be tested. This is supported, in the default genesis, the root key is set to `Alice`.

Following these examples, it is easy to add additional tests for these or new modules. A `minblock` can be specified per extrinsic to ensure that the extrinsic is not applied before a certain block number.



Unchecked extrinsics are fully supported. There are two types of Unchecked extrinsics, signed extrinsics and inherent extrinsics. Inherent extrinsic can only be applied by the block author during the generation of the block, signed extrinsics are similar to transactions. Some extrinsics require `root_origin`. If the relay-chain runtime under test features the `Sudo` module, the `Sudo` module can be used to dispatch with `root_origin`. The pre-configured user `Alice` has been initialized as the `Sudo` key (see the initialization section below).

If the Sudo module is *not* available in the runtime under test (as we assume to be the case for releases of the Polkadot relay-chain runtime in the future), a solution to circumvent this might be to use `wasm2wat` and to deactivate the check of `ensure_root_origin`.


Fuzzing
~~~~~~~

PolPatrol supports two types of fuzzing for input generation:

1. *Fuzz calldata*: In this mode, PolPatrol generates completely random calldata and passes it to the `Runtime Entry` to be fuzzed.
2. *Fuzz components*: In this mode, PolPatrol generates random headers and blocks with the expected components (headers, extrinsics, and blocks) but it randomly generates the individual components (i.e., it will generate a completely random header for the block). Extrinsics are fetched randomly from the extrinsics pool, this ensures meaningful input.

Users can specify as a command-line argument the number of fuzzing inputs that will be generated by PolPatrol. By default, this number is set to zero. 

We note that the fuzzing tests are expected to trigger errors. To this end, PolPatrol warns the user in case a random test input results in a successful call.




Test execution
--------------

We now describe how PolPatrol initializes the tests.

Initialization
~~~~~~~~~~~~~~

The initial state of the environment is defined by the provided genesis storage file. Currently, PolPatrol uses a slightly modified version of the Kusama-cc2 genesis state.

At the initial state, there are five accounts: `Alice`, `Bob`, `Charlie`, `Dave`, and `Eve`.  Accounts `Alice`, `Bob`, and `Charlie` have positive balances at the initial state, while accounts `Dave` and `Eve` have their balances set to zero. 

Block generation and execution
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

To execute the tests, PolPatrol first loads the relay-chain runtime under test. Then, it queries the version of the loaded runtime, which is needed when generating the extrinsics. Further, it queries the metadata to know which extrinsics are available at the runtime (this can be used to skip the hand-crafted extrinsics that are not supported by the runtime).

Next, PolPatrol generates new blocks in a loop, including a fixed number of extrinsics in each one, until no more extrinsics are left to execute. 

PolPatrol uses two execution paths:

1. One execution path where the new block is generated. The new block is initialized using `Core_initialize_block()`

	- Inherent extrinsics are generated using `BlockBuilder_inherent_extrinsics`
	- Extrinsics to be included are fetched
	- Extrinsics are applied using `BlockBuilder_apply_extrinsic`
	- The new block is finalized using `BlockBuilder_finalize_block`

2. A second execution path where the newly generated block is applied against the previous state using `Core_execute_block`. 

Each execution path has its own execution environment with its own storage. At the end of a block, the roots of both storages are compared to ensure that they do not diverge.


Notes on test execution:

- Test execution stops upon detection of storage root mismatch (between the 2x execution paths: aggregation of the block vs execution of the block) at the end of a block, an appropriate error message tells the user to fix the affected extrinsic.
-  If the execution of an extrinsic fails with `Ok(Err(*))`, then this leads to a storage root mismatch while failures with `Ok(Ok(Fail))` (e.g., due to an insufficient balance or lack of permission) are do not result in a storage root mismatch. 
- For the current Kusama-cc2 runtime, the module `OnlyStakingandClaims` is active. This means the current code for the runtimes contains a hardcoded limitation to fail all other calls, see `/runtime/src/lib.rs` inside the Polkadot repository:

  .. code-block:: rust

  	fn validate(&self, _: &Self::AccountId, call: &Self::Call, _: DispatchInfo, _: usize)
  	  -> TransactionValidity
  	{
  	  match call {
  	    Call::Staking(_) | Call::Claims(_) | Call::Sudo(_) | Call::Session(_) =>
  	      Ok(Default::default()),
  	        => Err(InvalidTransaction::Custom(ValidityError::NoPermission.into()).into()),
  	  }
  	}

  To compile the Runtime for proper testing, these checks need to be commented out and instead a return statement `Ok(Default::default())` must be added. This allows to call any of the include module successfully.



Instrumentation
---------------

PolPatrol instruments all calls between the runtime under test and the runtime environment and produces a trace similar to the widely used `strace` tool. At each instrumentation point, PolPatrol logs the following information:

- Memory usage
- Timestamp

In the end, PolPatrol aggregates the logged information and outputs:

- Peak memory usage. High memory usage indicates possible resource exhaustion issues.
- Storage interactions. Too many storage accesses indicate possible performance degradtion issues.

An example output of the summary provided by PolPatrol is given below:

.. code-block:: bash

    ----------------------------------------------------------------------
    Category                            Total calls
    ----------------------------------------------------------------------
    Storage                             923
    ----------------------------------------------------------------------

    ----------------------------------------------------------------------
    Function name                       Avg time [ns]        Max time[ns]
    ----------------------------------------------------------------------
    Core_version                        204908496            204908496
    Core_initialize_block               316352510            799149126
    Metadata_metadata                   499686228            499686228
    BlockBuilder_inherent_extrinsics    213342475            364014756
    BlockBuilder_apply_extrinsic        274847949            580431098
    BlockBuilder_finalize_block         264116597            598055717
    Core_execute_block                  465311233            1094130323
    ----------------------------------------------------------------------


PolPatrol logs the overall execution time of a call to the runtime and reports it.

Limitations
-----------

In this section, we list several limitations of the current version of PolPatrol. For a discussion on how these can be lifted in future versions of the tool, see section Future Work.

1. No hidden functions

  The current version of PolPatrol generates tests for a pre-defined set of functions. Therefore, any other functions implemented in the runtime that may have undesirable behavior (e.g. unbounded loops) will not be tested. 


2. Support for testing parachain functionality

  For each block, the parachain inherent sets parachain heads to empty.

.. _Future Work:

Future Work
-----------

1. Checking typestate properties

  Typestate properties specify which sequence of calls to the environment are considered valid. Since the sequence of calls to the environment is logged by PolPatrol, one can directly encode and check typestate properties. Such custom properties can extend the checks that are already implemented in the Substrate executor.

2. Automatically extract supported function calls.

  Currently, PolPatrol generates test inputs for a pre-defined set of extrinsic calls. PolPatrol can be extended to support runtimes with arbitrary extrinsics by extracting the list of available calls from metadata.

3. Advanced fuzzing

  PolPatrol fuzzing is limited to the generation of either completely random calldata or blocks with random headers. One can improve the fuzzing capabilities of PolPatrol to well-formed blocks (i.e., blocks whose validation does not result in an error).

4. Ensure the runtime implements the correct dispatcher

  A malicious runtime may implement a dishonest dispatcher that exposes certain functions that are only known to the attacker. For example, the runtime may not expose these functions to any other users even if one explicitly queries the runtime.

5. Extend tests and improve coverage

  A malicious runtime may implement the functions with malicious behavior. For example, invoking a modified `transfer()` function of the balance module may behave differently when it is invoked by the attacker (e.g., it transfers more DOTs to the attacker).

6. Conformance with the specification

  - Ensure that the runtime exposes the minimum set of functions it is supposed to expose to the environment.
  - Check if standard Polkadot modules with the same name and functions were included in the runtime and whether these have been modified. Discrepancies should warn users.

7. Check for release compiler flag [minor]

   PolPatrol can detect if the Wasm blob has been built with the `--release` flag. By default, the compiler produces a Wasm blob using debug mode, which may result in significant performance slowdowns (approximately 10x slower). 
