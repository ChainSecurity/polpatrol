Getting started
===============


Polkadot is a multichain where the Polkadot *relay-chain* is the core chain connecting different *parachains*. The parachains can implement arbitrary blockchains with custom functionality and architecture.

A *runtime* is at the heart of each chain, no matter if it is a relay-chain or parachain. Each runtime is a Wasm blob that is executed within a runtime environment. The relay-chain runtime is executed within the Polkadot Runtime Environment (PRE), which provides access to the network, state storage, and memory.

The Polkadot relay-chain runtime consists of multiple modules, each dedicated to performing a specific task. Examples modules are `Balances`, `Council`, and `Parachains`. Some of these modules are imported from the Substrate Runtime Module Library (SRML), while others are implemented specifically for the Polkadot relay-chain. Modules can define call functions, storage entries, and events.  

*Extrinsics* are described as *pieces of information from the outside world* that are contained in the blocks. When an Extrinsics is applied, it invokes a function call of a module. This happens either if the extrinsic is applied with `BlockBuilder_apply_extrinsic` during the building of a new block or applied inside a block in `core_execute_block`.
There are two types of Unchecked extrinsics:

* *Signed* extrinsics: These are generally known as transactions. These must be signed by the respective account to be considered valid.
* *Inherent* extrinsics: These specify information (such as an update of the Timestamp) that a block producer may add to the block.


Building
--------

Prerequisites
~~~~~~~~~~~~~

- Rust 2018
- polkadot_mod
- substrate_mod


Build
~~~~~

To build PolPatrol, execute the following command from the root folder of the project:

.. code-block:: bash

	git submodule update --init --recursive
	cargo build --release

The above command generates an executable called `runtime-checker` in folder `target/build`.



Usage
-----

.. code-block:: bash

	./target/release/runtime-checker

Runs the tool using the default values on the default runtime.

.. code-block:: bash

	./target/release/runtime-checker --help

Displays information about available cmd line parameters.

The most important cmd line options are the following:

.. code-block:: bash

	-g, --genesisfile

Allows to specify the genesis file defining the initial state of the storage

.. code-block:: bash

	-r, --runtimefile

Allows to specify the relay chain runtime wasm file to be tested

.. code-block:: bash

	-n, --num

Allows to specify the number of unchecked extrinsics to be include per Block (default: 3)

.. code-block:: bash

	-s, --seed

Allows to specify the seed for the random generator as u64 (default: 0)

Supported runtimes & available examples
----------------------------------------

The tool depends heavily on the datatypes used by the runtime under test, as these are passed as an argument and received as a return value (scale-encoded). Therefore it is vital that the datatypes match as otherwise unexpected errors will occur.

To avoid any such issue, PolPatrol should be compiled including the Polkadot and
Substrate dependencies at the very same commit as the runtime to be tested does.

The provided version of the tools is based on following commits:

`73104d3ae1ec061c4efd981a83cdd09104ba159f` for Substrate
`d517dbeb1d27b8068952e086c9ae472d49b707bd` for Polkadot

and works with any runtime using the datatypes as defined in these commits.

Some example runtimes are available in `/res`:

`polkadot_runtime.compactnewsnoonlystakingandclaims.wasm` The Polkadot runtime, modified. The `OnlyStakingAndClaims` check has been deactivated to allow successful calls to all included modules.

`polkadot_runtime.compactcontaininginifiniteloopslotsbid.wasm` The Polkadot runtime, modified. The `OnlyStakingAndClaims` check has been deactivated and an infinite loop has been added inside the `bid()` function of the `slots` module.

`polkadot_runtime.compact_memoryleak_slotsbid_release.wasm` The Polkadot runtime, modified. The `OnlyStakingAndClaims` check has been deactivated and an infinite loop writing to memory has been added inside the `bid()` function of the `slots` module.

Note that other `.wasm` files inside the `/res` folder, such as `adder.compact.wasm`, are parachain runtimes that are used as inputs for tests and not meant as example runtimes for PolPatrol.


Test your own runtime
----------------------

As mentioned above, it's important that the datatypes of objects exchanged between the runtime blob and the tool match, or decoding may fail. To support your runtime, build
PolPatrol including the Substrate and Polkadot repositories at the same commit as used for the runtime to be tested.

 .. code-block:: bash

	 cargo build --release
	 ./target/release -r /path/to/your/runtime.wasm
