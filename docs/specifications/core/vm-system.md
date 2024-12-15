---
authors:
  - Matt Faherty
date: '2024-11-03'
status: draft
title: Virtual Machine (VM)
type: specification
version: 1.1.0
---

# Virtual Machine (VM) Specification

## 1. Overview

### 1.1 Purpose

The Virtual Machine (VM) is a core component of the Inter-Cooperative Network (ICN), responsible for executing cooperative-specific smart contracts. It interprets a set of predefined OpCodes, allowing for a range of operations that support governance, resource allocation, reputation management, and cooperative administration.

### 1.2 Core Components

- **Instruction Set**: A collection of OpCodes defining operations the VM can execute.
- **Execution Engine**: Processes instructions, manages the stack, memory, and control flow.
- **Security Mechanisms**: Enforces permissions, reputation requirements, and resource limits.
- **Event System**: Generates events during execution for logging and auditing.

## 2. Detailed Specifications

### 2.1 Data Structures

#### 2.1.1 OpCode Enumeration

Defines the set of operations supported by the VM, categorized by functionality:

- **Arithmetic Operations**
  - `Add`, `Sub`, `Mul`, `Div`, `Mod`
- **Stack Operations**
  - `Push`, `Pop`, `Dup`, `Swap`
- **Memory Operations**
  - `Store`, `Load`
- **Control Flow Operations**
  - `Jump`, `JumpIf`, `Call`, `Return`
- **Cooperative Operations**
  - `CreateCooperative`, `JoinCooperative`, `LeaveCooperative`, `AllocateResource`, `TransferResource`
- **Governance Operations**
  - `CreateProposal`, `CastVote`, `DelegateVotes`, `ExecuteProposal`, `UpdateQuorum`
- **Reputation Operations**
  - `UpdateReputation`, `GetReputation`
- **Identity Operations**
  - `VerifyDID`, `UpdateDIDDocument`
- **System Operations**
  - `Log`, `Halt`, `EmitEvent`, `GetTimestamp`, `GetCaller`

#### 2.1.2 Contract Structure

```rust
struct Contract {
    id: String,
    code: Vec<(OpCode, Option<i64>)>,
    state: HashMap<String, i64>,
    required_reputation: i64,
    cooperative_metadata: CooperativeMetadata,
    version: String,
    dependencies: Vec<String>,
    permissions: Vec<String>,
}
```

- **id**: Unique identifier for the contract.
- **code**: Sequence of OpCodes and optional arguments.
- **state**: Persistent state specific to the contract.
- **required_reputation**: Minimum reputation required to execute the contract.
- **cooperative_metadata**: Metadata providing context for the cooperative.
- **version**: Contract versioning for updates and compatibility.
- **dependencies**: List of other contracts or libraries required.
- **permissions**: Access control permissions required.

#### 2.1.3 Execution Context

```rust
struct ExecutionContext {
    caller_did: String,
    cooperative_id: String,
    timestamp: u64,
    block_number: u64,
    reputation_score: i64,
    permissions: Vec<String>,
}
```

- **caller_did**: DID of the entity invoking the contract.
- **cooperative_id**: Identifier of the cooperative context.
- **timestamp**: Current timestamp of execution.
- **block_number**: Blockchain block number.
- **reputation_score**: Reputation score of the caller.
- **permissions**: Permissions of the caller.

### 2.2 Interfaces

#### 2.2.1 VM Methods

##### Execute Contract

- **Purpose**: Executes a contract from start to finish, enforcing all constraints.
- **Input**: `Contract`, `ExecutionContext`
- **Process**:
  1. **Permission Check**: Verify caller has necessary permissions.
  2. **Reputation Validation**: Ensure caller's reputation meets the requirement.
  3. **Initialize Stack and Memory**: Set up execution environment.
  4. **Instruction Execution**: Process OpCodes sequentially.
  5. **Event Emission**: Generate events as specified.
  6. **State Update**: Persist any changes to the contract state.
- **Output**: Execution result, updated state, events generated.

##### Execute Instruction

- **Purpose**: Processes a single OpCode and modifies the VM state accordingly.
- **Input**: `OpCode`, `Option<i64>`
- **Process**:
  - Handle operation based on the type of OpCode.
  - Update stack, memory, or control flow as needed.
- **Output**: Updated VM state.

### 2.3 Behaviors

#### 2.3.1 Stack Management

- **Structure**: LIFO (Last-In-First-Out) stack of 64-bit integers.
- **Operations**:
  - **Push**: Add value to the top of the stack.
  - **Pop**: Remove and return the top value.
  - **Dup**: Duplicate the top value.
  - **Swap**: Swap the top two values.

#### 2.3.2 Memory Management

- **Structure**: Key-value store (`HashMap<String, i64>`) for temporary storage.
- **Operations**:
  - **Store**: Save a value with a specified key.
  - **Load**: Retrieve a value by key.

#### 2.3.3 Control Flow

- **Jump**: Redirect execution to a specific instruction index.
- **JumpIf**: Conditional jump based on the top of the stack.
- **Call**: Invoke a function or subroutine.
- **Return**: Exit from a function or contract execution.

#### 2.3.4 Error Handling

- **Exception Types**:
  - **Stack Underflow/Overflow**
  - **Invalid Opcode**
  - **Permission Denied**
  - **Reputation Insufficient**
  - **Runtime Errors**: Division by zero, invalid memory access.

- **Handling Strategy**:
  - Execution halts on errors.
  - Generate error events with detailed messages.
  - Rollback any state changes made during execution.

### 2.4 Security Model

- **Reputation Enforcement**: Contracts specify minimum reputation; the VM enforces this before execution.
- **Permission Checks**: Caller permissions are validated against contract requirements.
- **Resource Limits**:
  - **Instruction Limit**: Maximum number of instructions per execution to prevent infinite loops.
  - **Stack Depth Limit**: Prevent stack overflows.
  - **Memory Usage Limit**: Cap on memory allocation.

- **Isolation**: Each contract execution is sandboxed, preventing interference with other contracts or global state.

## 3. Implementation Guidelines

### 3.1 Performance Requirements

- **Efficiency**: Optimize instruction execution for speed.
- **Scalability**: Handle multiple concurrent executions.
- **Lightweight**: Minimize resource consumption.

### 3.2 Security Requirements

- **Determinism**: Ensure contract execution is deterministic across all nodes.
- **Validation**: Strictly validate all inputs and OpCodes.
- **Auditability**: Maintain detailed logs and event histories.

### 3.3 Error Handling

- **Graceful Termination**: Contracts should fail safely without crashing the VM.
- **Clear Messaging**: Provide informative error messages for debugging.

## 4. Testing Requirements

- **Unit Tests**: Cover individual OpCodes and VM operations.
- **Integration Tests**: Test full contract executions in various scenarios.
- **Security Tests**: Include tests for permission enforcement and reputation checks.
- **Performance Tests**: Benchmark execution times and resource usage.

## 5. Monitoring and Metrics

- **Execution Logs**: Record each instruction executed.
- **Event Logs**: Capture all events emitted during execution.
- **Error Logs**: Record detailed information about any errors encountered.
- **Metrics**:
  - Execution time per contract.
  - Resource usage statistics.
  - Frequency of specific OpCodes.

## 6. Future Considerations

- **Extensibility**: Allow for new OpCodes and features to be added.
- **Language Support**: Develop higher-level languages that compile down to VM bytecode.
- **Interoperability**: Enable interaction with contracts on other blockchains or VMs.
- **Optimizations**: Implement Just-In-Time (JIT) compilation or other optimizations.

## Appendix

### A. OpCode Definitions

Provide detailed descriptions of each OpCode, including its function, expected stack state, and any arguments.

#### A.1 Arithmetic Operations

- **Add**
  - **Function**: Pops two values from the stack, pushes their sum.
  - **Stack Before**: `[ ... , a, b ]`
  - **Stack After**: `[ ... , a + b ]`
- **Sub**
  - **Function**: Pops two values, pushes the result of `a - b`.
  - **Stack Before**: `[ ... , a, b ]`
  - **Stack After**: `[ ... , a - b ]`
- *(Continue for other arithmetic OpCodes)*

#### A.2 Stack Operations

- *(Define `Push`, `Pop`, `Dup`, `Swap`)*

#### A.3 Memory Operations

- **Store**
  - **Function**: Stores a value in memory with a specified key.
  - **Arguments**: Key (from stack or instruction argument).
  - **Operation**:
    1. Pop value from stack.
    2. Use key to store the value in memory.

- **Load**
  - **Function**: Loads a value from memory onto the stack.
  - **Arguments**: Key.
  - **Operation**:
    1. Retrieve value associated with key.
    2. Push value onto the stack.

#### A.4 Control Flow Operations

- *(Define `Jump`, `JumpIf`, `Call`, `Return`)*

#### A.5 Cooperative Operations

- **CreateCooperative**
  - **Function**: Initiates a new cooperative.
  - **Requirements**: Caller must have sufficient reputation and permissions.
  - **Effect**: Emits `CooperativeCreated` event.

- *(Continue for other cooperative OpCodes)*

#### A.6 Governance Operations

- *(Define `CreateProposal`, `CastVote`, etc.)*

#### A.7 System Operations

- **Log**
  - **Function**: Logs a message or value for debugging.
  - **Operation**:
    1. Pop value from stack.
    2. Record value in execution logs.

- **Halt**
  - **Function**: Stops execution of the contract.
  - **Effect**: Returns control to the caller.

## References

- **ICN Architecture Overview**
- **Reputation System Specification**
- **Governance System Specification**

### B. Modular Structure

The VM management modules are now split into smaller submodules for better separation of concerns. Below is the updated structure:

#### vm/operations/stack.rs
- **stack_management**: Handles stack operations such as push, pop, dup, and swap.

#### vm/operations/federation.rs
- **federation_operations**: Manages federation-related operations such as creating and joining federations.
