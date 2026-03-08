# HELmR Architecture

HELmR is a constrained autonomy runtime that governs autonomous agent actions.

Instead of allowing agents to interact directly with the operating system or network, HELmR sits between agents and the real world.

Agent → HELmR → World

Every action must pass through HELmR before it can execute.

---

## Core Principle

NO TOKEN → NO ACTION

Agents must obtain a capability token before performing any operation.

Tokens authorize a single action and are destroyed after use.

This prevents replay attacks and unauthorized execution.

---

## System Components

HELmR is composed of several core subsystems.

### Runtime Gateway

The runtime gateway is the entry point for all agent requests.

It exposes HTTP endpoints used by agents to request authorization and perform actions.

Key endpoints:

/authorize  
/airlock/*

The gateway is implemented in Rust using Axum.

---

### Policy Engine

The policy engine evaluates whether a requested action is allowed.

Example rules:

write_file → allowed only inside workspace directory  
http_request → allowed via HELmR airlock  
filesystem outside workspace → denied  
unknown actions → denied

Policies are deterministic and auditable.

---

### Capability Token System

When an agent requests authorization, HELmR generates a capability token.

Token fields include:

agent_id  
mission_id  
action_type  
target  
issued_at  
signature  

Tokens are signed using HMAC SHA256.

Tokens must:

• match the action being executed  
• verify the signature  
• not be reused

Single-use tokens prevent replay attacks.

---

### Airlock Execution Layer

HELmR performs all real-world operations through airlock endpoints.

Examples:

/airlock/write_file  
/airlock/http_request

Agents never interact with the system directly.

HELmR performs the operation after verifying the capability token.

---

### Trace System

HELmR produces deterministic trace artifacts for every action.

Trace events contain:

timestamp  
agent identity  
mission id  
action type  
target  
decision  
reason  
token hash

Trace logs are sealed using SHA256 to detect tampering.

---

### Live Operations Console

HELmR includes a live monitoring console.

Endpoint:

/console/board

The board displays:

TIME  
AGENT  
ACTION  
TARGET  
RESULT  
CONTROL

Operators can freeze an agent action directly from the console.

---

## Control Loop

The HELmR runtime follows this control sequence:

authorize → token issued → airlock execution → token burned → trace recorded

Replay attempts are blocked automatically.

---

## Technology Stack

Rust  
Axum  
Tokio  
Serde  
Reqwest  
HMAC SHA256  
SHA256 trace sealing

---

## Future Components

HELmR Guard

Consumer control interface for visualizing and controlling autonomous agents.

HELmR Platform

Enterprise governance layer for fleets of autonomous agents.

---

## Design Philosophy

HELmR is designed around a simple rule:

Autonomous agents must never execute actions directly.

All real-world operations must pass through an auditable control layer.

HELmR provides that layer.