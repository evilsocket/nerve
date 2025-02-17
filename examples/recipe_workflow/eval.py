import json
import os
import sys
import random

def append_state_to_file(state: dict, filepath: str) -> None:
    try:
        if not os.path.exists(filepath):
            with open(filepath, 'w') as f:
                f.write('')
        
        with open(filepath, 'a') as f:
            f.write('\n' + json.dumps(state))
    except Exception as e:
        print(f"Error saving state: {e}", file=sys.stderr)

if __name__ == "__main__":
    raw = sys.stdin.read()
    state = json.loads(raw)
    
    append_state_to_file(state, 'state_history.json')
    random_score = random.random()
    if random_score > 0.5:
        print(json.dumps({'score': random_score}))
        exit(42)
    else:
        print(json.dumps({'score': random_score}))
        exit(0)

