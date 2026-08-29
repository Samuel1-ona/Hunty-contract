#!/usr/bin/env node

const { execSync } = require('child_process');

function showHelp() {
  console.log(`
Hunty CLI Tool
Usage: node hunty-cli.js <command> [options]

Commands:
  create-hunt     --name <name> --start <timestamp> --end <timestamp> --creator <address>
  add-clue        --hunt <hunt_id> --answer <answer> --points <points>
  activate        --hunt <hunt_id> --caller <address>
  register        --hunt <hunt_id> --player <address>
  submit-answer   --hunt <hunt_id> --clue <clue_id> --answer <answer> --player <address>
  leaderboard     --hunt <hunt_id>
  player-stats    --hunt <hunt_id> --player <address>
  pause           --admin <address>
  unpause         --admin <address>

Global Options:
  --network       Network to use (default: testnet)
  --contract      Contract ID (required)
  --source        Source account/identity for the transaction
`);
}

function parseArgs(args) {
  const result = {};
  for (let i = 0; i < args.length; i++) {
    if (args[i].startsWith('--')) {
      const key = args[i].substring(2);
      const val = args[i + 1] && !args[i + 1].startsWith('--') ? args[i + 1] : true;
      result[key] = val;
      if (val !== true) i++;
    }
  }
  return result;
}

function runInvoke(method, contractArgs, options) {
  if (!options.contract) {
    console.error("Error: --contract is required.");
    process.exit(1);
  }
  
  let cmd = `soroban contract invoke --id ${options.contract} --network ${options.network || 'testnet'} `;
  if (options.source) {
    cmd += `--source ${options.source} `;
  }
  
  cmd += `-- ${method}`;
  
  for (const [key, val] of Object.entries(contractArgs)) {
    cmd += ` --${key} ${val}`;
  }
  
  console.log(`Executing: ${cmd}`);
  try {
    const output = execSync(cmd, { encoding: 'utf-8' });
    console.log("Result:", output);
  } catch (error) {
    console.error("Error executing command:");
    console.error(error.stdout || error.message);
  }
}

function main() {
  const args = process.argv.slice(2);
  if (args.length === 0 || args.includes('--help') || args.includes('-h')) {
    showHelp();
    return;
  }

  const command = args[0];
  const options = parseArgs(args.slice(1));

  switch (command) {
    case 'create-hunt':
      runInvoke('create_hunt', {
        creator: options.creator,
        name: options.name,
        start_time: options.start,
        end_time: options.end
      }, options);
      break;
    
    case 'add-clue':
      runInvoke('add_clue', {
        hunt_id: options.hunt,
        question: options.question || "Default Question",
        answer: options.answer,
        points: options.points,
        is_required: options.required || false,
        difficulty: options.difficulty || 1
      }, options);
      break;
      
    case 'activate':
      runInvoke('activate_hunt', {
        hunt_id: options.hunt,
        caller: options.caller
      }, options);
      break;
      
    case 'register':
      runInvoke('register_player', {
        hunt_id: options.hunt,
        player: options.player
      }, options);
      break;

    case 'submit-answer':
      runInvoke('submit_answer', {
        hunt_id: options.hunt,
        clue_id: options.clue,
        answer: options.answer,
        player: options.player
      }, options);
      break;
      
    case 'leaderboard':
      runInvoke('get_hunt_leaderboard', {
        hunt_id: options.hunt
      }, options);
      break;

    case 'player-stats':
      runInvoke('get_player_progress', {
        hunt_id: options.hunt,
        player: options.player
      }, options);
      break;
      
    case 'pause':
      runInvoke('pause_contract', {
        admin: options.admin
      }, options);
      break;
      
    case 'unpause':
      runInvoke('unpause_contract', {
        admin: options.admin
      }, options);
      break;
      
    default:
      console.error(`Unknown command: ${command}`);
      showHelp();
      process.exit(1);
  }
}

main();
