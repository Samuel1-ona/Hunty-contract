const fs = require('fs');

const filePath = 'contracts/hunty-core/src/test.rs';
let content = fs.readFileSync(filePath, 'utf8');

// We want to find all occurrences of "HuntyCore::add_clue(" and balance parentheses to find the call.
let pos = 0;
let updatedCount = 0;

while (true) {
    const startIdx = content.indexOf('HuntyCore::add_clue(', pos);
    if (startIdx === -1) break;

    // Parse arguments
    let bracketCount = 0;
    let inString = false;
    let escape = false;
    let endIdx = -1;
    let args = [''];
    let currentArg = '';

    for (let i = startIdx + 'HuntyCore::add_clue('.length; i < content.length; i++) {
        const char = content[i];
        if (escape) {
            currentArg += char;
            escape = false;
            continue;
        }
        if (char === '\\') {
            currentArg += char;
            escape = true;
            continue;
        }
        if (char === '"') {
            inString = !inString;
            currentArg += char;
            continue;
        }
        if (inString) {
            currentArg += char;
            continue;
        }

        if (char === '(') {
            bracketCount++;
            currentArg += char;
        } else if (char === ')') {
            if (bracketCount === 0) {
                endIdx = i;
                args.push(currentArg.trim());
                break;
            } else {
                bracketCount--;
                currentArg += char;
            }
        } else if (char === ',' && bracketCount === 0) {
            args.push(currentArg.trim());
            currentArg = '';
        } else {
            currentArg += char;
        }
    }

    if (endIdx !== -1) {
        // Filter out empty arguments (e.g. trailing commas, first empty arg initialization)
        const cleanedArgs = args.map(a => a.trim()).filter(a => a.length > 0);
        if (cleanedArgs.length === 6) {
            // It has 6 arguments, we need to add the 7th argument: 1 (difficulty)
            const callSlice = content.slice(startIdx, endIdx);
            const newCallSlice = callSlice + ', 1';
            content = content.slice(0, startIdx) + newCallSlice + content.slice(endIdx);
            updatedCount++;
            pos = startIdx + newCallSlice.length + 1;
        } else {
            pos = endIdx + 1;
        }
    } else {
        pos = startIdx + 1;
    }
}

fs.writeFileSync(filePath, content, 'utf8');
console.log(`Updated ${updatedCount} calls to HuntyCore::add_clue in ${filePath}`);
