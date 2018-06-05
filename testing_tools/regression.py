#!/usr/bin/env python3.6

import argparse
import os
import subprocess as proc
import csv
from subprocess import run
import fnmatch
from pathlib import Path
from shutil import copyfile


# List of files that should be skipped.
CRASH_ALLOWED = [
    'e-svg-007.svg'
]


def change_ext(path, suffix, new_ext):
    return Path(path).stem + suffix + '.' + new_ext


def render_svg(in_svg, out_png):
    run(['node', 'svgrender.js', in_svg, out_png, '200'],
        check=True, cwd='chrome-svgrender')


def load_last_pos():
    path = args.work_dir / 'pos.txt'
    if path.exists():
        with open(path, 'r') as f:
            return int(f.read().splitlines()[0])
    return 0


def save_last_pos(pos):
    path = args.work_dir / 'pos.txt'
    with open(path, 'w') as out:
        out.write(str(pos) + '\n')


def rm_file(file_path):
    if file_path.exists():
        os.remove(file_path)


def remove_artifacts():
    rm_file(svg_copy_path)
    rm_file(svg_path_usvg)
    rm_file(png_path_orig)
    rm_file(png_path_usvg)
    rm_file(diff_path)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('svg_dir', type=Path, help='Sets an input directory with SVG files')
    parser.add_argument('work_dir', type=Path, help='Sets the working directory')
    parser.add_argument('--dpi', type=int, default=96, help='Sets the DPI')
    args = parser.parse_args()

    if not args.work_dir.exists():
        os.mkdir(args.work_dir)

    allowed_ae = {}
    with open('allow.csv') as f:
        for row in csv.reader(f):
            allowed_ae[row[0]] = int(row[1])

    start_idx = load_last_pos()
    files = os.listdir(args.svg_dir)
    files = fnmatch.filter(files, '*.svg')
    files = sorted(files)
    for idx, file in enumerate(files):
        svg_path = args.svg_dir / file
        svg_copy_path = args.work_dir / file
        svg_path_usvg = args.work_dir / change_ext(file, '_usvg', 'svg')
        png_path_orig = args.work_dir / change_ext(file, '_orig', 'png')
        png_path_usvg = args.work_dir / change_ext(file, '_usvg', 'png')
        diff_path = args.work_dir / change_ext(file, '_diff', 'png')

        remove_artifacts()

        if idx < start_idx:
            continue

        print('Test {} of {}: {}'.format(idx + 1, len(files), file))

        if file in CRASH_ALLOWED:
            continue

        try:
            run(['../target/debug/usvg', svg_path, svg_path_usvg, '--dpi', str(args.dpi)],
                check=True)
        except proc.CalledProcessError as e:
            print('Error: usvg crashed.')
            save_last_pos(idx)
            exit(1)

        render_svg(svg_path, png_path_orig)
        render_svg(svg_path_usvg, png_path_usvg)

        try:
            diff_val = run(['compare', '-metric', 'AE', '-fuzz', '1%',
                            png_path_orig, png_path_usvg, diff_path],
                           check=True, stdout=proc.PIPE, stderr=proc.STDOUT).stdout
        except proc.CalledProcessError as e:
            ae = int(e.stdout.decode('ascii'))
            if ae > 20 and ae != allowed_ae.get(file, 0):
                print('Error: images are different by {} pixels.'.format(ae))

                # copy original svg on error
                copyfile(svg_path, svg_copy_path)

                save_last_pos(idx)
                exit(1)

        remove_artifacts()

    save_last_pos(0)
