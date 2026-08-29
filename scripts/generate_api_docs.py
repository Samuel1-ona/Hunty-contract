#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from pathlib import Path
from typing import Dict, List, Optional, Sequence

ROOT = Path(__file__).resolve().parent.parent
CONTRACTS_DIR = ROOT / "contracts"
DOCS_HEADER = """# Contract API Reference

This file is generated automatically from the Rust contract sources in `contracts/`.
Run `make build` to regenerate it whenever contract APIs change.

"""


def split_params(params_text: str) -> List[str]:
    params = []
    current = []
    depth = 0
    for char in params_text:
        if char == '<':
            depth += 1
        elif char == '>':
            depth -= 1
        elif char == ',' and depth == 0:
            params.append(''.join(current).strip())
            current = []
            continue
        current.append(char)
    if current:
        params.append(''.join(current).strip())
    return [p for p in params if p]


def normalize_type(type_text: str) -> str:
    return ' '.join(type_text.strip().split())


def parse_error_enums(text: str) -> Dict[str, List[Dict[str, str]]]:
    enums: Dict[str, List[Dict[str, str]]] = {}
    enum_iter = re.finditer(r'#[ \t]*\[contracterror\].*?enum\s+(\w+)\s*{', text, flags=re.S)
    for enum_match in enum_iter:
        enum_name = enum_match.group(1)
        start_index = enum_match.end()
        brace_depth = 1
        i = start_index
        while i < len(text) and brace_depth > 0:
            if text[i] == '{':
                brace_depth += 1
            elif text[i] == '}':
                brace_depth -= 1
            i += 1
        enum_body = text[start_index:i - 1]
        variants = []
        pending_doc_lines: List[str] = []
        for line in enum_body.splitlines():
            trimmed = line.strip()
            if trimmed.startswith('///'):
                pending_doc_lines.append(trimmed[3:].strip())
                continue
            if not trimmed or trimmed.startswith('//'):
                continue
            variant_match = re.match(r'^(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*(?:=\s*(?P<value>\d+))?\s*,?$', trimmed)
            if variant_match:
                name = variant_match.group('name')
                value = variant_match.group('value') or ''
                variants.append({
                    'name': name,
                    'value': value,
                    'description': ' '.join(pending_doc_lines).strip(),
                })
                pending_doc_lines = []
        enums[enum_name] = variants
    return enums


def parse_contract_impls(text: str) -> List[Dict[str, object]]:
    impls = []
    impl_iter = re.finditer(r'#[ \t]*\[contractimpl\].*?\n\s*impl\s+(?P<impl_name>\w+)\s*{', text, flags=re.S)
    for impl_match in impl_iter:
        impl_name = impl_match.group('impl_name')
        block_start = impl_match.end() - 1
        depth = 1
        i = block_start + 1
        while i < len(text) and depth > 0:
            if text[i] == '{':
                depth += 1
            elif text[i] == '}':
                depth -= 1
            i += 1
        block_text = text[block_start + 1:i - 1]
        lines = block_text.splitlines()
        doc_lines: List[str] = []
        for idx, line in enumerate(lines):
            stripped = line.strip()
            if stripped.startswith('///'):
                doc_lines.append(stripped[3:].strip())
                continue
            func_match = re.match(r'^\s*pub\s+fn\s+(?P<name>\w+)\s*\(', line)
            if not func_match:
                continue
            signature_lines = [line]
            j = idx + 1
            brace_found = '{' in line
            while j < len(lines) and not brace_found:
                next_line = lines[j]
                signature_lines.append(next_line)
                if '{' in next_line:
                    brace_found = True
                j += 1
            signature = ' '.join(l.strip() for l in signature_lines)
            sig_match = re.search(
                r'pub\s+fn\s+(?P<name>\w+)\s*\((?P<params>.*?)\)\s*(?:->\s*(?P<return>[^\{]+))?\s*\{',
                signature,
                flags=re.S,
            )
            if not sig_match:
                doc_lines = []
                continue
            name = sig_match.group('name')
            params = split_params(sig_match.group('params') or '')
            return_type_raw = sig_match.group('return') or '()'
            return_type = normalize_type(return_type_raw)
            error_type = None
            error_codes = None
            error_match = re.search(r'Result\s*<.*?,\s*([^>]+)>', return_type)
            if error_match:
                error_type = error_match.group(1).strip()
                error_type = error_type.split('::')[-1]
            function_doc = '\n'.join(doc_lines).strip()
            impls.append({
                'contract': impl_name,
                'name': name,
                'params': params,
                'return_type': return_type,
                'error_type': error_type,
                'description': function_doc,
            })
            doc_lines = []
        if doc_lines:
            doc_lines = []
    return impls


def gather_contract_data() -> Dict[str, Dict[str, object]]:
    contract_data: Dict[str, Dict[str, object]] = {}
    error_enums: Dict[str, List[Dict[str, str]]] = {}
    for contract_dir in sorted(CONTRACTS_DIR.iterdir()):
        if not contract_dir.is_dir():
            continue
        src_dir = contract_dir / 'src'
        if not src_dir.exists():
            continue
        contract_name = contract_dir.name
        contract_data[contract_name] = {
            'impls': [],
            'errors': {},
        }
        for rust_file in sorted(src_dir.glob('*.rs')):
            text = rust_file.read_text()
            enums = parse_error_enums(text)
            for key, variants in enums.items():
                if key not in error_enums:
                    error_enums[key] = variants
            impls = parse_contract_impls(text)
            for impl in impls:
                contract_data[contract_name]['impls'].append(impl)
    contract_data['errors'] = error_enums
    return contract_data


def render_markdown(data: Dict[str, Dict[str, object]]) -> str:
    lines: List[str] = [DOCS_HEADER]
    for crate_name, crate_data in sorted((k, v) for k, v in data.items() if k != 'errors'):
        lines.append(f'## `{crate_name}` Contract')
        lines.append('')
        impls = crate_data.get('impls', [])
        if not impls:
            lines.append('_No contract API functions found._')
            lines.append('')
            continue
        current_contract = None
        for impl in impls:
            contract = impl['contract']
            if contract != current_contract:
                lines.append(f'### `{contract}`')
                lines.append('')
                current_contract = contract
            lines.append(f'#### `{impl["name"]}`')
            lines.append('')
            if impl['description']:
                lines.append(impl['description'])
                lines.append('')
            signature = f'pub fn {impl["name"]}({", ".join(impl["params"])}) -> {impl["return_type"]}'
            lines.append('**Signature:**')
            lines.append('')
            lines.append('```rust')
            lines.append(signature)
            lines.append('```')
            lines.append('')
            if impl['params']:
                lines.append('**Parameters:**')
                lines.append('')
                for param in impl['params']:
                    lines.append(f'- `{param}`')
                lines.append('')
            lines.append(f'**Returns:** `{impl["return_type"]}`')
            lines.append('')
            if impl['error_type']:
                lines.append(f'**Error type:** `{impl["error_type"]}`')
                lines.append('')
                error_defs = data.get('errors', {}).get(impl['error_type'], [])
                if error_defs:
                    lines.append('**Error codes:**')
                    lines.append('')
                    for variant in error_defs:
                        code_str = f' = {variant["value"]}' if variant['value'] else ''
                        description = f' - {variant["description"]}' if variant['description'] else ''
                        lines.append(f'- `{variant["name"]}`{code_str}{description}')
                    lines.append('')
            lines.append('---')
            lines.append('')
    if data.get('errors'):
        lines.append('# Error Code Reference')
        lines.append('')
        for error_name, variants in sorted(data['errors'].items()):
            lines.append(f'## `{error_name}`')
            lines.append('')
            for variant in variants:
                code_str = f' = {variant["value"]}' if variant['value'] else ''
                description = f' - {variant["description"]}' if variant['description'] else ''
                lines.append(f'- `{variant["name"]}`{code_str}{description}')
            lines.append('')
    return '\n'.join(lines).rstrip() + '\n'


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description='Generate contract markdown API documentation.')
    parser.add_argument('--output', '-o', default=str(ROOT / 'docs' / 'contract-api.md'))
    args = parser.parse_args(argv)
    data = gather_contract_data()
    markdown = render_markdown(data)
    output_path = Path(args.output)
    output_path.write_text(markdown)
    print(f'Generated API docs: {output_path}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
