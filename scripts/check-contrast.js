#!/usr/bin/env node

/**
 * WCAG Contrast Checker
 * Calculates contrast ratios for color combinations
 */

function hexToRgb(hex) {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result ? {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  } : null;
}

function getLuminance(r, g, b) {
  const [rs, gs, bs] = [r, g, b].map(c => {
    c = c / 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

function getContrastRatio(hex1, hex2) {
  const rgb1 = hexToRgb(hex1);
  const rgb2 = hexToRgb(hex2);
  
  if (!rgb1 || !rgb2) return null;
  
  const lum1 = getLuminance(rgb1.r, rgb1.g, rgb1.b);
  const lum2 = getLuminance(rgb2.r, rgb2.g, rgb2.b);
  
  const lighter = Math.max(lum1, lum2);
  const darker = Math.min(lum1, lum2);
  
  return (lighter + 0.05) / (darker + 0.05);
}

function checkWCAG(ratio, level = 'AA') {
  if (level === 'AA') {
    return {
      normalText: ratio >= 4.5,
      largeText: ratio >= 3.0,
    };
  }
  if (level === 'AAA') {
    return {
      normalText: ratio >= 7.0,
      largeText: ratio >= 4.5,
    };
  }
  return { normalText: false, largeText: false };
}

// Dark theme combinations to check (UPDATED with fixes)
const darkThemeCombinations = [
  { name: 'Body text on background', fg: '#9ca3af', bg: '#0a0a0a' },
  { name: 'Card text on card background', fg: '#9ca3af', bg: '#1a1a1a' },  // FIXED
  { name: 'Primary foreground on primary', fg: '#1a1a1a', bg: '#60a5fa' },
  { name: 'Secondary foreground on secondary', fg: '#d1d5db', bg: '#4b5563' },
  { name: 'Muted text on muted background', fg: '#9ca3af', bg: '#1f2937' },  // FIXED
  { name: 'Accent text on accent background', fg: '#93c5fd', bg: '#1e3a8a' },
];

console.log('\n='.repeat(60));
console.log('WCAG AA Contrast Audit - Dark Theme');
console.log('='.repeat(60));
console.log('\nStandards:');
console.log('  Normal text (< 18pt): 4.5:1');
console.log('  Large text (>= 18pt): 3.0:1');
console.log('='.repeat(60));

const failures = [];

darkThemeCombinations.forEach(combo => {
  const ratio = getContrastRatio(combo.fg, combo.bg);
  const wcag = checkWCAG(ratio);
  
  const normalStatus = wcag.normalText ? '✓ PASS' : '✗ FAIL';
  const largeStatus = wcag.largeText ? '✓ PASS' : '✗ FAIL';
  
  console.log(`\n${combo.name}`);
  console.log(`  Foreground: ${combo.fg}`);
  console.log(`  Background: ${combo.bg}`);
  console.log(`  Contrast: ${ratio.toFixed(2)}:1`);
  console.log(`  Normal text: ${normalStatus}`);
  console.log(`  Large text:  ${largeStatus}`);
  
  if (!wcag.normalText) {
    failures.push({
      ...combo,
      ratio: ratio.toFixed(2),
      required: 4.5
    });
  }
});

console.log('\n' + '='.repeat(60));
console.log(`Total combinations checked: ${darkThemeCombinations.length}`);
console.log(`Failures: ${failures.length}`);
console.log('='.repeat(60));

if (failures.length > 0) {
  console.log('\n⚠️  FAILURES REQUIRING CORRECTION:\n');
  failures.forEach((f, i) => {
    console.log(`${i + 1}. ${f.name}`);
    console.log(`   Current: ${f.ratio}:1 | Required: ${f.required}:1`);
    console.log(`   FG: ${f.fg} | BG: ${f.bg}\n`);
  });
  process.exit(1);
}

console.log('\n✓ All contrast ratios meet WCAG AA standards!\n');
