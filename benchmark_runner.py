#!/usr/bin/env python3
"""
VCR Benchmark Runner
Runs VCR commands and collects performance metrics
"""

import subprocess
import json
import time
import sys

def run_vcr_command(cmd, description):
    """Run a VCR command and return timing + output"""
    print(f"\n{'='*60}")
    print(f"Running: {description}")
    print(f"Command: {cmd}")
    print(f"{'='*60}")
    
    start = time.time()
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            capture_output=True,
            text=True,
            timeout=30
        )
        elapsed = (time.time() - start) * 1000  # Convert to ms
        
        print(f"Exit code: {result.returncode}")
        print(f"Time: {elapsed:.2f}ms")
        
        if result.stdout:
            print(f"Output: {result.stdout.strip()}")
        if result.stderr:
            print(f"Stderr: {result.stderr.strip()}")
        
        return {
            'success': result.returncode == 0,
            'time_ms': elapsed,
            'output': result.stdout.strip(),
            'stderr': result.stderr.strip()
        }
    except subprocess.TimeoutExpired:
        print("TIMEOUT!")
        return {'success': False, 'time_ms': 30000, 'output': '', 'stderr': 'Timeout'}
    except Exception as e:
        print(f"ERROR: {e}")
        return {'success': False, 'time_ms': 0, 'output': '', 'stderr': str(e)}

def main():
    print("VCR Benchmark Suite")
    print("=" * 60)
    
    results = {}
    
    # Test 1: Ingestion (single file)
    results['ingest_lib'] = run_vcr_command(
        'vcr ingest src/lib.rs',
        'Ingest lib.rs (determinism test)'
    )
    
    # Test 2: Ingestion again (should be identical)
    results['ingest_lib_2'] = run_vcr_command(
        'vcr ingest src/lib.rs',
        'Ingest lib.rs again (verify determinism)'
    )
    
    # Test 3: Snapshot save
    results['snapshot_save'] = run_vcr_command(
        'vcr snapshot save',
        'Save CPG snapshot'
    )
    
    # Test 4: Snapshot verify
    results['snapshot_verify'] = run_vcr_command(
        'vcr snapshot verify /tmp/vcr-snapshot-demo.bin',
        'Verify snapshot integrity'
    )
    
    # Summary
    print("\n" + "=" * 60)
    print("BENCHMARK SUMMARY")
    print("=" * 60)
    
    for name, result in results.items():
        status = "✓" if result['success'] else "✗"
        print(f"{status} {name:20s} {result['time_ms']:8.2f}ms")
    
    # Determinism check
    print("\n" + "=" * 60)
    print("DETERMINISM VERIFICATION")
    print("=" * 60)
    
    if results['ingest_lib']['success'] and results['ingest_lib_2']['success']:
        try:
            hash1 = json.loads(results['ingest_lib']['output']).get('cpg_hash', '')
            hash2 = json.loads(results['ingest_lib_2']['output']).get('cpg_hash', '')
            
            if hash1 == hash2:
                print(f"✓ DETERMINISTIC: Hash matches!")
                print(f"  Hash: {hash1[:16]}...")
            else:
                print(f"✗ NON-DETERMINISTIC: Hash mismatch!")
                print(f"  Run 1: {hash1}")
                print(f"  Run 2: {hash2}")
        except Exception as e:
            print(f"Could not parse JSON: {e}")
    
    print("\n" + "=" * 60)
    print("Benchmark complete!")
    print("=" * 60)

if __name__ == '__main__':
    main()
