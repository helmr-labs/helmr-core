import requests

HELmR = "http://127.0.0.1:7070"

print("Requesting authorization from HELmR...")

auth = requests.post(
    f"{HELmR}/authorize",
    json={
        "agent_id": "demo_agent",
        "mission_id": "demo_mission",
        "action_type": "write_file",
        "target": "workspace/demo.txt"
    }
)

data = auth.json()
print("Authorization response:", data)

token = data.get("token")

if token:
    print("Token received. Executing airlock action...")

    action = requests.post(
        f"{HELmR}/airlock/write_file",
        json={
            "token": token
        }
    )

    print("Airlock response:", action.text)

else:
    print("Authorization failed. No token issued.")