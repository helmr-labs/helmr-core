# HELmR

Constrained Autonomy Runtime for AI Agents.

HELmR is a runtime governance layer that sits between autonomous agents and the real world.

Agent → HELmR → World

Every action must be authorized before it executes.

NO TOKEN → NO ACTION.

---

## Why HELmR Exists

Autonomous AI agents can execute:

• filesystem operations  
• network requests  
• shell commands  
• API calls  

without governance.

HELmR provides a runtime control layer that:

• authorizes agent actions  
• issues capability tokens  
• executes actions through airlock endpoints  
• records deterministic trace logs  
• allows operator intervention  

---

## Core Features

• Authorization gateway  
• Capability token system  
• Airlock execution layer  
• Replay protection  
• Breadcrumb tracing  
• Live activity console  
• Operator freeze controls  

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

All operations pass through HELmR.

---

## Run HELmR in 60 Seconds

Clone the repository:

git clone https://github.com/helmr/helmr-core

cd helmr-core

Start HELmR:

cargo run

HELmR will start locally at:

http://127.0.0.1:7070

Open the console:

http://127.0.0.1:7070/console/board

---

## Run the Example Agent

In another terminal:

python examples/demo_agent.py

You will see the agent request authorization and HELmR will execute the action through the airlock.

Replay attempts will be blocked.

---

## Example Endpoints

Authorization:

POST /authorize

Airlock execution:

POST /airlock/write_file

Console:

GET /console/board

---

## Product Structure

HELmR Core  
Open runtime governance engine.

HELmR Guard  
Visual control center for autonomous agents.

HELmR Platform  
Enterprise governance system for fleets of agents.

---

## License

MIT