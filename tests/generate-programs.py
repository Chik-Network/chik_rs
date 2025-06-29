#!/usr/bin/env python3

import os
from klvm import KEYWORD_TO_ATOM


def recursive_cons(filename: str, num: int) -> None:
    with open(filename, "w+") as f:
        f.write(
            """
;(mod (N)
;    (defun prepend (V N)
;        (if N (c V (prepend V (- N 1) V)) ())
;    )
;    (prepend 1337 N)
;)

(a (q 2 2 (c 2 (c (q . 1337) (c 5 ())))) (c (q 2 (i 11 (q 4 5 (a 2 (c 2 (c 5 (c (- 11 (q . 1)) (c 5 ())))))) ()) 1) 1))
"""
        )

    with open(filename[:-4] + "env", "w+") as f:
        f.write("(%d)" % num)


def many_args(filename: str, op: str, num: int) -> None:
    with open(filename + "-precompiled", "w+") as f:
        f.write(
            """
;(mod (n)
;    (defun large-atom (n)
;        (if n (lsh (large-atom (- n 1)) 65535) 0x80)
;    )
;    (defun raise (atom)
;        (%s atom ... )
;    )
;    (raise (large-atom n))
;)
"""
            % op
        )

        f.write(
            "(a (q 2 6 (c 2 (c (a 4 (c 2 (c 5 (q)))) (q)))) (c (q (a (i 5 (q 23 (a 4 (c 2 (c (- 5 (q . 1)) (q)))) (q . 0x00ffff)) (q 1 . -128)) 1) %s"
            % op
        )
        f.write(" 5" * num)
        f.write(") 1))")

    with open(filename[:-4] + "env", "w+") as f:
        f.write("(500)")

    if op.startswith("0x"):
        hexop = op[2:]
        if len(hexop) % 2 != 0:
            hexop = "0" + hexop
        if len(hexop) > 2 or hexop[0] in "fec8":
            # in this case we need a length prefix for the atom
            hexop = "8%x" % (len(hexop) // 2) + hexop
    else:
        hexop = KEYWORD_TO_ATOM[op].hex()

    with open(filename[:-4] + "hex", "w+") as f:
        f.write(
            "ff02ffff01ff02ff06ffff04ff02ffff04ffff02ff04ffff04ff02ffff04ff05ffff"
            "0180808080ffff0180808080ffff04ffff01ffff02ffff03ff05ffff01ff17ffff02"
            "ff04ffff04ff02ffff04ffff11ff05ffff010180ffff0180808080ffff018300ffff"
            "80ffff01ff01818080ff0180ff" + hexop + ("ff05" * num) + "80ff018080"
        )


def many_args_point(filename: str, op: str, num: int) -> None:
    with open(filename, "w+") as f:
        f.write(
            """;(mod (n)
;    (defun raise (atom)
;        (%s atom atom atom atom atom atom atom atom atom atom atom atom atom atom)
;    )
;    (raise (logxor n 0xb3b8ac537f4fd6bde9b26221d49b54b17a506be147347dae5d081c0a6572b611d8484e338f3432971a9823976c6a232b))
;)
"""
            % op
        )
        f.write(
            "(a (q 2 2 (c 2 (c (logxor 5 (q . 0xb3b8ac537f4fd6bde9b26221d49b54b17a506be147347dae5d081c0a6572b611d8484e338f3432971a9823976c6a232b)) (q)))) (c (q %s"
            % op
        )
        f.write(" 5" * num)
        f.write(") 1)))")

    with open(filename[:-4] + "env", "w+") as f:
        f.write("(0)")


def softfork_wrap(filename: str, val: str) -> None:

    with open(filename, "w+") as f:
        f.write(
            """;(mod (n)
;    (defun recurse (count)
;        (if (= 0 count) 42 (recurse (+ (- count 1) (softfork %s))))
;    )
;    (recurse n)
;)

(a (q 2 2 (c 2 (c 5 (q)))) (c (q 2 (i (= (q) 5) (q 1 . 42) (q 2 2 (c 2 (c (+ (- 5 (q . 1)) (softfork (q . %s))) (q))))) 1) 1))
"""
            % (val, val)
        )

    with open(filename[:-4] + "env", "w+") as f:
        f.write("(0xffffffff)")


def binary_recurse(filename: str, op: str, val: str, count: int) -> None:

    with open(filename, "w+") as f:
        f.write(
            """; (mod (N)
;   (defun iter (V N)
;     (if (= N 0) V (iter ({op} V V) (- N 1)))
;   )
;   (iter {val} N)
; )
(a (q 2 2 (c 2 (c (q . {val}) (c 5 ())))) (c (q 2 (i (= 11 ()) (q . 5) (q 2 2 (c 2 (c ({op} 5 5) (c (- 11 (q . 1)) ()))))) 1) 1))
""".format(
                op=op, val=val
            )
        )

    with open(filename[:-4] + "env", "w+") as f:
        f.write(f"({count})")


def unary_recurse(filename: str, op: str, second: str, count: int) -> None:

    if second != "":
        quoted_second = f" (q . {second})"
    else:
        quoted_second = ""

    with open(filename, "w+") as f:
        f.write(
            """; (mod (N)
;   (defun large-atom (n)
;       (if n (lsh (large-atom (- n 1)) 65535) 0x80)
;   )
;   (defun iter (V N)
;     (if (= N 0) V (iter ({op} V {second}) (- N 1)))
;   )
;   (iter (large-atom 6) N)
; )
(a (q 2 4 (c 2 (c (a 6 (c 2 (q 6))) (c 5 ())))) (c (q (a (i (= 11 ()) (q . 5) (q 2 4 (c 2 (c ({op} 5 {quoted_second}) (c (- 11 (q . 1)) ()))))) 1) 2 (i 5 (q 23 (a 6 (c 2 (c (- 5 (q . 1)) ()))) (q . 0x00ffff)) (q 1 . -128)) 1) 1))
""".format(
                op=op, second=second, quoted_second=quoted_second
            )
        )

    with open(filename[:-4] + "env", "w+") as f:
        f.write(f"({count})")


def serialized_atom_overflow(filename: str, size: int) -> None:
    with open(filename, "w+") as f:
        if size == 0:
            size_blob = b"\x80"
        elif size < 0x40:
            size_blob = bytes([0x80 | size])
        elif size < 0x2000:
            size_blob = bytes([0xC0 | (size >> 8), (size >> 0) & 0xFF])
        elif size < 0x100000:
            size_blob = bytes(
                [0xE0 | (size >> 16), (size >> 8) & 0xFF, (size >> 0) & 0xFF]
            )
        elif size < 0x8000000:
            size_blob = bytes(
                [
                    0xF0 | (size >> 24),
                    (size >> 16) & 0xFF,
                    (size >> 8) & 0xFF,
                    (size >> 0) & 0xFF,
                ]
            )
        elif size < 0x400000000:
            size_blob = bytes(
                [
                    0xF8 | (size >> 32),
                    (size >> 24) & 0xFF,
                    (size >> 16) & 0xFF,
                    (size >> 8) & 0xFF,
                    (size >> 0) & 0xFF,
                ]
            )
        else:
            size_blob = bytes(
                [
                    0xFC | ((size >> 40) & 0xFF),
                    (size >> 32) & 0xFF,
                    (size >> 24) & 0xFF,
                    (size >> 16) & 0xFF,
                    (size >> 8) & 0xFF,
                    (size >> 0) & 0xFF,
                ]
            )
        f.write(size_blob.hex())
        f.write("01" * 1000)


try:
    os.mkdir("programs")
except:
    pass

binary_recurse("programs/recursive-cat.klvm", "concat", '"ABCDEF"', 29)
binary_recurse(
    "programs/recursive-mul.klvm",
    "*",
    "0x7ffffffffffffffffffffffffffffffffffffffffffff",
    100,
)
binary_recurse(
    "programs/recursive-add.klvm",
    "+",
    "0x7ffffffffffffffffffffffffffffffffffffffffffffffff",
    5000000,
)
binary_recurse(
    "programs/recursive-sub.klvm",
    "-",
    "0x7ffffffffffffffffffffffffffffffffffffffffffffffff",
    5000000,
)
unary_recurse("programs/recursive-div.klvm", "/", "13", 1000000)
unary_recurse("programs/recursive-lsh.klvm", "lsh", "65535", 10000)
unary_recurse("programs/recursive-ash.klvm", "ash", "65535", 10000)
unary_recurse("programs/recursive-pubkey.klvm", "pubkey_for_exp", "", 10000)
unary_recurse("programs/recursive-not.klvm", "lognot", "", 10000000)
many_args("programs/args-mul.klvm", "*", 10000)
many_args("programs/args-add.klvm", "+", 10000)
many_args("programs/args-sub.klvm", "-", 10000)
many_args("programs/args-sha.klvm", "sha256", 10000)
many_args("programs/args-cat.klvm", "concat", 10000)
many_args("programs/args-any.klvm", "any", 300000)
many_args("programs/args-all.klvm", "all", 300000)
many_args("programs/args-and.klvm", "logand", 10000)
many_args("programs/args-or.klvm", "logior", 10000)
many_args("programs/args-xor.klvm", "logxor", 10000)
many_args_point("programs/args-point_add.klvm", "point_add", 12000)
many_args("programs/args-unknown-1.klvm", "0x7fffffff00", 5000)
many_args("programs/args-unknown-2.klvm", "0x7ff40", 3000)
many_args("programs/args-unknown-3.klvm", "0x7ff80", 3000)
many_args("programs/args-unknown-4.klvm", "0x7ffc0", 3000)
unary_recurse("programs/args-unknown-5.klvm", "0x7ff00", "0xffffffffffffff", 3000000)
unary_recurse("programs/args-unknown-6.klvm", "0x001", "0xfffffffffffff", 30000000)
unary_recurse("programs/args-unknown-7.klvm", "0x041", "0xfffffffffffff", 30000000)
unary_recurse("programs/args-unknown-8.klvm", "0x081", "0xfffffffffffff", 30000000)
unary_recurse("programs/args-unknown-9.klvm", "0x0c1", "0xfffffffffffff", 30000000)
recursive_cons("programs/recursive-cons.klvm", 10000000)

# this program attempts to wrap around a 64 bit cost counter
softfork_wrap("programs/softfork-1.klvm", "0x00ffffffffffffff45")
# this program attempts to wrap around a 32 bit cost counter
softfork_wrap("programs/softfork-2.klvm", "0x00ffffff45")

# tests that try to trick a parser into allocating too much memory
serialized_atom_overflow("programs/large-atom-1.hex.invalid", 0xFFFFFFFF)
serialized_atom_overflow("programs/large-atom-2.hex.invalid", 0x3FFFFFFFF)
serialized_atom_overflow("programs/large-atom-3.hex.invalid", 0xFFFFFFFFFF)
serialized_atom_overflow("programs/large-atom-4.hex.invalid", 0x1FFFFFFFFFF)
