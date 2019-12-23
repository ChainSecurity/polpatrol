.. Polkadot runtime checker documentation master file, created by
   sphinx-quickstart on Thu Nov  7 20:49:08 2019.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.


.. image:: _static/polpatrol.png
   :width: 600

PolPatrol: validator for Polkadot runtimes
==========================================

The Polkadot relay-chain runtime is the core of the Polkadot network. Ensuring that its implementation is secure and functionally correct is of critical importance for the entire Polkadot network. The relay-chain runtime allows updates of its implementation, i.e., replacing the current relay-chain WebAssembly binary (or, Wasm blob for short) with a new one. The security and correctness of new relay-chain runtimes will, therefore, be repeatedly tested in the future.

*PolPatrol* is a tool for validating and testing Polkadot runtimes. The current version of PolPatrol is designed to test the stability and security of relay-chain runtimes with respect to generic security and performance properties.


Who should use PolPatrol?
-------------------------

PolPatrol will be primarily used by two user groups:

1. Developers of new relay-chain runtimes who would like to test the correctness of their code.
2. DOT token holders who can vote on newly proposed relay-chain runtimes and would like to check whether a proposed runtime is safe and secure. 

For both user groups, PolPatrol automatically analyzes any given relay-chain Wasm blob and warns users upon violation of important security and performance properties.


How does it work?
-----------------

PolPatrol uses an instrumented Polkadot runtime environment to run the provided relay-chain runtime and log all calls that it makes to the environment. In addition to the calls, it also monitors important performance metrics such as execution time and memory usage. This enables PolPatrol to check arbitrary trace properties (such as typestate properties on how the runtime interacts with the environment) and performance properties (average and peak execution time and memory usage). PolPatrol aggregates the collected runtime information to let developers draw conclusions on the overall behavior of the runtime and compare different implementations.

What's next?
------------

PolPatrol is a proof-of-concept tool that can be extended with *fuzzing* capabilities (to automatically generate inputs for the runtime) and *dynamic analysis* (to amplify the results of the observed execution traces). For details on possible extensions, please refer to our :ref:`Future Work` section.



.. toctree::
   :maxdepth: 2
   :caption: Contents:

   getting-started.rst
   system.rst
   contributors.rst
