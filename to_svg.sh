#! /bin/sh
python3 to_dot.py $1 > /tmp/file.dot
dot /tmp/file.dot -Tsvg -o $2
