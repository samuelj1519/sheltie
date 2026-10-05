# Methods, state, and content judgment

English | [简体中文](../../zh-CN/explanation/workflow-model.md)

A Workbook is a reusable method; a Work is one run of a frozen method version. A Flow defines stages and explicit edges. A Node defines instructions, inputs, outputs, executor, gate, and limits for one step. Business vocabulary does not have to enter the engine to become a method.

## Arrivals, attempts, and feedback

Every arrival at a node forms an Occurrence; an execution record within that arrival is an Attempt. Retrying after execution failure creates a new Attempt. Returning along a back edge creates a new Occurrence. `max_retries` and `max_visits` limit these separately.

A completed review with a failing content verdict is still an execution that succeeded. The coordinator reads the report and selects a legal explicit edge for rework; the engine does not select edges from natural language. An executor crash or inability to deliver is failed. Administrative revocation of formal qualification is superseded, without counting as business failure or stopping the host process.

## Why briefs provide bound sources

Method authors write instructions in advance. The engine gives the executor frozen input paths and output requirements for this Attempt. Optional feedback may have no source at the first arrival; later it binds to a specific successful artifact, avoiding guesses about which file is newest.

After session interruption, resume lets a new context find the current brief, frozen inputs, and draft locations. It proves neither draft quality nor file existence. Operators still handle old executors and the shared workspace. Ordinary interruption does not automatically become failure or administrative revocation.

## Gates, success, and results are separate facts

executor determines who executes; gate determines whether approval is required after success before leaving. A human Node can deliver a conclusion. A gate records an approval call without authenticating an independent person or judging output content.

A Work can succeed after a terminal node with no outgoing edges finishes and no unapproved gate remains. Final results still depend on that terminal's explicit result selections. Without declarations there is no selected set: neither all outputs nor the last file in a directory is automatically a result. The two-step example deliberately has no final selections, making this distinction observable in the tutorial. code-change and spec-dev explicitly select their deliveries.

Use [CONTEXT](../../../CONTEXT.md) terminology. See [data reference](../reference/data-model.md) for fields and states and the [resume guide](../how-to/resume-work.md) for operations.
