// Example: Monitor audit events from all Hunty contracts
const { Server } = require('soroban-client');

const server = new Server('https://horizon-testnet.stellar.org');

// Filter for audit events by topic
const auditFilter = {
  topics: [
    ['AUDIT'], // Match all audit events
  ],
};

server.events()
  .forContract('HUNTY_CORE_CONTRACT_ID')
  .filter(auditFilter)
  .cursor('now')
  .stream({
    onmessage: (event) => {
      const auditData = JSON.parse(event.value);
      console.log(`[AUDIT] ${auditData.action_type} by ${auditData.admin_address} at ${new Date(auditData.timestamp * 1000)}`);
      
      // Alert on emergency actions
      if (auditData.action_type === 'EMERGENCY') {
        sendSecurityAlert(auditData);
      }
      
      // Log all admin changes
      if (auditData.action_type.startsWith('ADM_')) {
        logAdminChange(auditData);
      }
    }
  });