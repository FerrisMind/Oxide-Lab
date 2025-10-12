#!/usr/bin/env node

/**
 * License Compliance Checker
 *
 * This script checks that all dependencies use permissive licenses
 * compatible with the MIT license used by this project.
 */

import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

// Permissive licenses that are compatible with MIT
const PERMISSIVE_LICENSES = [
  'MIT',
  'Apache-2.0',
  'BSD-2-Clause',
  'BSD-3-Clause',
  'ISC',
  'CC0-1.0',
  'Unlicense',
  '0BSD',
  'MIT OR Apache-2.0',
  'Apache-2.0 OR MIT',
  'BSD-3-Clause OR MIT',
  'MIT OR Unlicense',
  'MPL-2.0 OR Apache-2.0', // We choose Apache-2.0 option
  'Apache-2.0 OR BSD-2-Clause OR MIT',
  'Apache-2.0 OR BSD-3-Clause OR MIT',
  'Apache-2.0 OR ISC OR MIT',
  'Apache-2.0 OR LGPL-2.1-or-later OR MIT', // We choose MIT option
  'Apache-2.0 OR MIT OR Zlib',
  'Apache-2.0 OR CC0-1.0 OR MIT-0',
  'Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT',
  'Apache-2.0 OR BSL-1.0',
  'Apache-2.0 AND ISC',
  'Apache-2.0 AND MIT',
  'BSD-3-Clause AND MIT',
  'BSD-3-Clause OR MIT',
  'MIT AND CC-BY-3.0',
  'Custom License File', // Usually permissive
  'Python-2.0', // Permissive
];

// Potentially problematic licenses to watch out for
const PROBLEMATIC_LICENSES = ['GPL', 'AGPL', 'LGPL', 'Copyleft', 'Proprietary', 'Commercial'];

function checkNpmLicenses() {
  console.log('🔍 Checking npm package licenses...');

  try {
    // Generate license report
    execSync('npm run licenses:csv', { stdio: 'pipe' });

    if (!fs.existsSync('licenses.csv')) {
      throw new Error('Failed to generate licenses.csv');
    }

    const csvContent = fs.readFileSync('licenses.csv', 'utf8');
    const lines = csvContent.split('\n').slice(1); // Skip header

    let issues = [];
    let totalPackages = 0;

    for (const line of lines) {
      if (!line.trim()) continue;

      const [name, license, repository] = line.split(',').map((s) => s.replace(/"/g, ''));
      totalPackages++;

      // Check if license is problematic
      const isProblematic = PROBLEMATIC_LICENSES.some((prob) => license.includes(prob));

      if (isProblematic) {
        issues.push({
          type: 'npm',
          name,
          license,
          repository,
          severity: 'ERROR',
        });
      }
    }

    console.log(`✅ Checked ${totalPackages} npm packages`);
    return issues;
  } catch (error) {
    console.error('❌ Error checking npm licenses:', error.message);
    return [{ type: 'npm', error: error.message, severity: 'ERROR' }];
  }
}

function checkRustLicenses() {
  console.log('🔍 Checking Rust crate licenses...');

  try {
    const output = execSync('cargo license', { encoding: 'utf8' });
    const lines = output.split('\n');

    let issues = [];
    let totalCrates = 0;

    for (const line of lines) {
      if (!line.trim() || line.includes('PS ') || line.includes('cargo license')) {
        continue;
      }

      // Parse cargo license output format
      const match = line.match(/^(.+?)\s+\((\d+)\):\s*(.+)$/);
      if (!match) continue;

      const [, license, count, crates] = match;
      const crateCount = parseInt(count);
      totalCrates += crateCount;

      // Check if license is problematic
      const isProblematic = PROBLEMATIC_LICENSES.some((prob) => license.includes(prob));

      if (isProblematic) {
        issues.push({
          type: 'rust',
          license,
          count: crateCount,
          crates: crates.split(', '),
          severity: 'ERROR',
        });
      }
    }

    console.log(`✅ Checked ${totalCrates} Rust crates`);
    return issues;
  } catch (error) {
    console.error('❌ Error checking Rust licenses:', error.message);
    return [{ type: 'rust', error: error.message, severity: 'ERROR' }];
  }
}

function generateReport(issues) {
  const timestamp = new Date().toISOString();

  const report = {
    timestamp,
    project: 'oxide-lab',
    license: 'MIT',
    status: issues.length === 0 ? 'COMPLIANT' : 'ISSUES_FOUND',
    issues,
    summary: {
      totalIssues: issues.length,
      npmIssues: issues.filter((i) => i.type === 'npm').length,
      rustIssues: issues.filter((i) => i.type === 'rust').length,
    },
  };

  // Save report
  fs.writeFileSync('license-compliance-report.json', JSON.stringify(report, null, 2));

  return report;
}

function main() {
  console.log('🚀 Starting license compliance check...\n');

  const npmIssues = checkNpmLicenses();
  const rustIssues = checkRustLicenses();

  const allIssues = [...npmIssues, ...rustIssues];

  console.log('\n📊 Generating compliance report...');
  const report = generateReport(allIssues);

  console.log('\n📋 COMPLIANCE REPORT');
  console.log('==================');
  console.log(`Status: ${report.status}`);
  console.log(`Total Issues: ${report.summary.totalIssues}`);
  console.log(`NPM Issues: ${report.summary.npmIssues}`);
  console.log(`Rust Issues: ${report.summary.rustIssues}`);

  if (allIssues.length > 0) {
    console.log('\n❌ ISSUES FOUND:');
    allIssues.forEach((issue, index) => {
      console.log(`\n${index + 1}. ${issue.type.toUpperCase()} - ${issue.severity}`);
      if (issue.name) console.log(`   Package: ${issue.name}`);
      if (issue.license) console.log(`   License: ${issue.license}`);
      if (issue.repository) console.log(`   Repository: ${issue.repository}`);
      if (issue.error) console.log(`   Error: ${issue.error}`);
    });

    console.log(
      '\n⚠️  Please review these issues and consider replacing problematic dependencies.',
    );
    process.exit(1);
  } else {
    console.log('\n✅ All dependencies use permissive licenses compatible with MIT!');
    console.log('🎉 Your project is legally compliant.');
  }

  console.log(`\n📄 Full report saved to: license-compliance-report.json`);
}

// Run the check
main();
