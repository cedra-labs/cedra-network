#!/usr/bin/env python3

'''
This module automated the steps to
calculate gas parameters for `algebra.move` natives from benchmarking results,
then update the gas parameter definitions in rust.
'''

import argparse
from collections import defaultdict
import fit_linear_model
import load_bench_ns
import load_bench_datapoints
from math import ceil
from pathlib import Path
from time import time

# Typically you are making a new version of gas schedule,
# so this should be larger than `LATEST_GAS_FEATURE_VERSION` in `cedra-move/cedra-gas/src/gas_meter.rs`.
TARGET_GAS_VERSION = 12

def get_bench_ns_linear(bench_path):
    datapoints = load_bench_datapoints.main(bench_path)
    X,Y,k,b = fit_linear_model.main(datapoints)
    return X,Y,k,b

def prettify_number(x:int) -> str:
    s = str(x)
    n = len(s)
    b = n % 3
    chunks_0 = [s[:b]] if b>=1 else []
    chunks = chunks_0 + [s[i:i+3] for i in range(b,n,3)]
    return '_'.join(chunks)

def get_algebra_lines(gas_per_ns):
    nanoseconds = {}
    nanoseconds['ark_bn254_fr_add'] = load_bench_ns.main('target/criterion/ark_bn254/fr_add')
    nanoseconds['ark_bn254_fr_deser'] = load_bench_ns.main('target/criterion/ark_bn254/fr_deser')
    nanoseconds['ark_bn254_fr_div'] = load_bench_ns.main('target/criterion/ark_bn254/fr_div')
    nanoseconds['ark_bn254_fr_eq'] = load_bench_ns.main('target/criterion/ark_bn254/fr_eq')
    nanoseconds['ark_bn254_fr_from_u64'] = load_bench_ns.main('target/criterion/ark_bn254/fr_from_u64')
    nanoseconds['ark_bn254_fr_inv'] = load_bench_ns.main('target/criterion/ark_bn254/fr_inv')
    nanoseconds['ark_bn254_fr_mul'] = load_bench_ns.main('target/criterion/ark_bn254/fr_mul')
    nanoseconds['ark_bn254_fr_neg'] = load_bench_ns.main('target/criterion/ark_bn254/fr_neg')
    nanoseconds['ark_bn254_fr_one'] = load_bench_ns.main('target/criterion/ark_bn254/fr_one')
    nanoseconds['ark_bn254_fr_serialize'] = load_bench_ns.main('target/criterion/ark_bn254/fr_serialize')
    nanoseconds['ark_bn254_fr_square'] = load_bench_ns.main('target/criterion/ark_bn254/fr_square')
    nanoseconds['ark_bn254_fr_sub'] = load_bench_ns.main('target/criterion/ark_bn254/fr_sub')
    nanoseconds['ark_bn254_fr_zero'] = 1
    nanoseconds['ark_bn254_fq_add'] = load_bench_ns.main('target/criterion/ark_bn254/fq_add')
    nanoseconds['ark_bn254_fq_clone'] = load_bench_ns.main('target/criterion/ark_bn254/fq_clone')
    nanoseconds['ark_bn254_fq_deser'] = load_bench_ns.main('target/criterion/ark_bn254/fq_deser')
    nanoseconds['ark_bn254_fq_div'] = load_bench_ns.main('target/criterion/ark_bn254/fq_div')
    nanoseconds['ark_bn254_fq_eq'] = load_bench_ns.main('target/criterion/ark_bn254/fq_eq')
    nanoseconds['ark_bn254_fq_from_u64'] = load_bench_ns.main('target/criterion/ark_bn254/fq_from_u64')
    nanoseconds['ark_bn254_fq_inv'] = load_bench_ns.main('target/criterion/ark_bn254/fq_inv')
    nanoseconds['ark_bn254_fq_mul'] = load_bench_ns.main('target/criterion/ark_bn254/fq_mul')
    nanoseconds['ark_bn254_fq_neg'] = load_bench_ns.main('target/criterion/ark_bn254/fq_neg')
    nanoseconds['ark_bn254_fq_one'] = 1
    nanoseconds['ark_bn254_fq_pow_u256'] = load_bench_ns.main('target/criterion/ark_bn254/fq_pow_u256')
    nanoseconds['ark_bn254_fq_serialize'] = load_bench_ns.main('target/criterion/ark_bn254/fq_serialize')
    nanoseconds['ark_bn254_fq_square'] = load_bench_ns.main('target/criterion/ark_bn254/fq_square')
    nanoseconds['ark_bn254_fq_sub'] = load_bench_ns.main('target/criterion/ark_bn254/fq_sub')
    nanoseconds['ark_bn254_fq_zero'] = 1
    nanoseconds['ark_bn254_fq12_add'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_add')
    nanoseconds['ark_bn254_fq12_clone'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_clone')
    nanoseconds['ark_bn254_fq12_deser'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_deser')
    nanoseconds['ark_bn254_fq12_div'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_div')
    nanoseconds['ark_bn254_fq12_eq'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_eq')
    nanoseconds['ark_bn254_fq12_from_u64'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_from_u64')
    nanoseconds['ark_bn254_fq12_inv'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_inv')
    nanoseconds['ark_bn254_fq12_mul'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_mul')
    nanoseconds['ark_bn254_fq12_neg'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_neg')
    nanoseconds['ark_bn254_fq12_one'] = 1
    nanoseconds['ark_bn254_fq12_pow_u256'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_pow_u256')
    nanoseconds['ark_bn254_fq12_serialize'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_serialize')
    nanoseconds['ark_bn254_fq12_square'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_square')
    nanoseconds['ark_bn254_fq12_sub'] = load_bench_ns.main('target/criterion/ark_bn254/fq12_sub')
    nanoseconds['ark_bn254_fq12_zero'] = 1
    nanoseconds['ark_bn254_g1_affine_deser_comp'] = load_bench_ns.main('target/criterion/ark_bn254/g1_affine_deser_comp')
    nanoseconds['ark_bn254_g1_affine_deser_uncomp'] = load_bench_ns.main('target/criterion/ark_bn254/g1_affine_deser_uncomp')
    nanoseconds['ark_bn254_g1_affine_serialize_comp'] = load_bench_ns.main('target/criterion/ark_bn254/g1_affine_serialize_comp')
    nanoseconds['ark_bn254_g1_affine_serialize_uncomp'] = load_bench_ns.main('target/criterion/ark_bn254/g1_affine_serialize_uncomp')
    nanoseconds['ark_bn254_g1_proj_add'] = load_bench_ns.main('target/criterion/ark_bn254/g1_proj_add')
    nanoseconds['ark_bn254_g1_proj_double'] = load_bench_ns.main('target/criterion/ark_bn254/g1_proj_double')
    nanoseconds['ark_bn254_g1_proj_eq'] = load_bench_ns.main('target/criterion/ark_bn254/g1_proj_eq')
    nanoseconds['ark_bn254_g1_proj_generator'] = 1
    nanoseconds['ark_bn254_g1_proj_infinity'] = 1
    nanoseconds['ark_bn254_g1_proj_neg'] = 1
    nanoseconds['ark_bn254_g1_proj_scalar_mul'] = load_bench_ns.main('target/criterion/ark_bn254/g1_proj_scalar_mul')
    nanoseconds['ark_bn254_g1_proj_sub'] = load_bench_ns.main('target/criterion/ark_bn254/g1_proj_sub')
    nanoseconds['ark_bn254_g1_proj_to_affine'] = load_bench_ns.main('target/criterion/ark_bn254/g1_proj_to_affine')
    nanoseconds['ark_bn254_g2_affine_deser_comp'] = load_bench_ns.main('target/criterion/ark_bn254/g2_affine_deser_comp')
    nanoseconds['ark_bn254_g2_affine_deser_uncomp'] = load_bench_ns.main('target/criterion/ark_bn254/g2_affine_deser_uncomp')
    nanoseconds['ark_bn254_g2_affine_serialize_comp'] = load_bench_ns.main('target/criterion/ark_bn254/g2_affine_serialize_comp')
    nanoseconds['ark_bn254_g2_affine_serialize_uncomp'] = load_bench_ns.main('target/criterion/ark_bn254/g2_affine_serialize_uncomp')
    nanoseconds['ark_bn254_g2_proj_add'] = load_bench_ns.main('target/criterion/ark_bn254/g2_proj_add')
    nanoseconds['ark_bn254_g2_proj_double'] = load_bench_ns.main('target/criterion/ark_bn254/g2_proj_double')
    nanoseconds['ark_bn254_g2_proj_eq'] = load_bench_ns.main('target/criterion/ark_bn254/g2_proj_eq')
    nanoseconds['ark_bn254_g2_proj_generator'] = 1
    nanoseconds['ark_bn254_g2_proj_infinity'] = 1
    nanoseconds['ark_bn254_g2_proj_neg'] = 1
    nanoseconds['ark_bn254_g2_proj_scalar_mul'] = load_bench_ns.main('target/criterion/ark_bn254/g2_proj_scalar_mul')
    nanoseconds['ark_bn254_g2_proj_sub'] = load_bench_ns.main('target/criterion/ark_bn254/g2_proj_sub')
    nanoseconds['ark_bn254_g2_proj_to_affine'] = load_bench_ns.main('target/criterion/ark_bn254/g2_proj_to_affine')
    nanoseconds['ark_bn254_pairing'] = load_bench_ns.main('target/criterion/ark_bn254/pairing')
    _,_,nanoseconds['ark_bn254_multi_pairing_per_pair'],nanoseconds['ark_bn254_multi_pairing_base'] = get_bench_ns_linear('target/criterion/ark_bn254/pairing_product')

    quantity_types = defaultdict(lambda: "InternalGas")
    quantity_types['ark_bn254_multi_pairing_per_pair'] = 'InternalGasPerArg'

    gas_param_entries = { k:(int(gas_per_ns*v), quantity_types[k]) for k,v in nanoseconds.items()}
    lines = [f'        [algebra_{k}: {unt}, {{ {TARGET_GAS_VERSION}.. => "algebra.{k}" }}, {prettify_number(qty)}],' for k,(qty,unt) in sorted(gas_param_entries.items())]
    return lines

def main(gas_per_ns):
    path = Path(PATH_STR)
    lines = path.read_text().split('\n')
    striped_lines = [line.strip() for line in lines]
    line_id_begin = striped_lines.index('// BN254 algebra gas parameters begin.')
    line_id_end = striped_lines.index('// BN254 algebra gas parameters end.')
    generator_note_line = f'        // Generated at time {time()} by `scripts/algebra-gas/update_bn254_algebra_gas_params.py` with gas_per_ns={gas_per_ns}.'
    new_lines = lines[:line_id_begin+1] + [generator_note_line] + get_algebra_lines(gas_per_ns) + lines[line_id_end:]
    path.write_text('\n'.join(new_lines))

PATH_STR = 'cedra-move/cedra-gas-schedule/src/gas_schedule/cedra_framework.rs'
if __name__=='__main__':
    parser = argparse.ArgumentParser(
        description=f'Generate gas parameters for BN254 algebra module in `{PATH_STR}`.')
    parser.add_argument('--gas_per_ns', required=True, type=float)
    args = parser.parse_args()
    main(args.gas_per_ns)
