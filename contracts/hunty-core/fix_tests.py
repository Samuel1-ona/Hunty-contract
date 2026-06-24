import re

with open('src/test.rs', 'r') as f:
    content = f.read()

# We need to find all calls to HuntyCore::submit_answer(...)
# It takes 5 arguments currently: env, hunt_id, clue_id, player, answer
# We need to add env.ledger().timestamp() as the 6th argument.

def repl(m):
    # m.group(1) is the arguments string inside submit_answer(...)
    # We just append `, env.ledger().timestamp()` to it.
    args = m.group(1)
    return f"HuntyCore::submit_answer({args}, env.ledger().timestamp())"

# Match HuntyCore::submit_answer( ... ) balancing parentheses is hard with pure regex, 
# but we can assume there are no nested unescaped ')' that would break a simple parser.
# Actually, since some answers are variables, we can just split by HuntyCore::submit_answer(
parts = content.split('HuntyCore::submit_answer(')
new_parts = [parts[0]]

for part in parts[1:]:
    # Find the closing parenthesis for submit_answer
    depth = 1
    for i, char in enumerate(part):
        if char == '(':
            depth += 1
        elif char == ')':
            depth -= 1
            if depth == 0:
                # Found the end of the arguments
                args = part[:i]
                rest = part[i+1:]
                new_part = f"HuntyCore::submit_answer({args}, env.ledger().timestamp()){rest}"
                new_parts.append(new_part)
                break

with open('src/test.rs', 'w') as f:
    f.write(''.join(new_parts))
