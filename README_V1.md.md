# HELmR

Constrained Autonomy Runtime for AI Agents.

A governance gateway that controls how autonomous agents interact with real systems.

Agent → HELmR → World

Every action must be authorized before it executes.

NO TOKEN → NO ACTION

---

## Why HELmR Exists

Autonomous agents are becoming increasingly capable.

They can:

• write files  
• call APIs  
• execute commands  
• interact with external systems  

But most agent frameworks provide little or no governance over what those agents are allowed to do.

HELmR introduces a runtime control layer between agents and the real world.

Instead of allowing agents to execute actions directly, HELmR requires every action to be:

1. authorized  
2. tokenized  
3. executed through an airlock  
4. recorded in a deterministic trace  

This creates a safe and auditable execution environment for autonomous systems.

---

## Core Features

• Authorization gateway for agent actions  
• Capability token system  
• Airlock execution layer  
• Replay protection (single-use tokens)  
• Deterministic breadcrumb tracing  
• Live activity console  
• Operator freeze controls  

HELmR acts as a runtime governance layer for autonomous agents.

---

## Architecture

Agent  
 ↓  
HELmR Gateway  
 ↓  
Policy Engine  
 ↓  
Capability Token  
 ↓  
Airlock Execution  
 ↓  
Trace Log  

Agents never execute real-world operations directly.

All execution flows through HELmR.

---

## Requirements

Before running HELmR install:

Rust (cargo)  
Python 3  

Rust runs the HELmR runtime.

Python runs the example agent.

---

## Run HELmR in 60 Seconds

Clone the repository:

git clone https://github.com/YOUR_USERNAME/helmr-core  
cd helmr-core

Start HELmR:

cargo run

HELmR will start locally at:

http://127.0.0.1:7070

Open the live console:

http://127.0.0.1:7070/console/board

---

## Run the Example Agent

Open a second terminal and run:

python examples/demo_agent.py

The example agent will:

1. request authorization from HELmR  
2. receive a capability token  
3. execute a write_file airlock action  
4. appear in the HELmR console board  

Replay attempts are automatically blocked.

---

## Example Endpoints

Authorization

POST /authorize

Airlock execution

POST /airlock/write_file

Console board

GET /console/board

---

## Product Structure

HELmR Core  
Open runtime governance engine.

HELmR Guard  
Visual control interface for monitoring and controlling agents.

HELmR Platform  
Enterprise governance system for fleets of autonomous agents.

---

## Contact

Questions or discussions:

Open a GitHub issue.

Security reports or private inquiries:

contact@publictrustshield.com

---

## License

MIT License