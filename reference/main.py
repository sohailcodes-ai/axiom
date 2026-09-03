import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from lexer import Lexer
from parser import Parser
from name_resolver import NameResolver
from type_checker import TypeChecker
from hir_lower import HIRLowerer
from codegen import CodeGenerator
from vm import VM
from errors import CompileError


def compile_and_run(source: str, filename: str = "<input>") -> int:
    # Lex
    lexer = Lexer(source, filename)
    tokens = lexer.tokenize()

    # Parse
    parser = Parser(tokens, filename)
    try:
        ast = parser.parse_program()
    except Exception as e:
        print(f"parse error: {e}", file=sys.stderr)
        return 1

    # Name resolution
    resolver = NameResolver(filename)
    resolver.resolve_program(ast)
    if resolver.errors:
        for error in resolver.errors:
            print(error, file=sys.stderr)
        return 1

    # Type checking
    checker = TypeChecker(filename)
    checker.check_program(ast)
    if checker.errors:
        for error in checker.errors:
            print(error, file=sys.stderr)
        return 1

    # Lower to HIR
    lowerer = HIRLowerer(filename)
    hir = lowerer.lower_program(ast)
    if lowerer.errors:
        for error in lowerer.errors:
            print(error, file=sys.stderr)
        return 1

    # Generate bytecode
    codegen = CodeGenerator(filename)
    program = codegen.generate(hir)
    if codegen.errors:
        for error in codegen.errors:
            print(error, file=sys.stderr)
        return 1

    # Run VM
    vm = VM(program)
    exit_code = vm.run()

    for line in vm.output:
        print(line)

    if vm.errors:
        for error in vm.errors:
            print(error, file=sys.stderr)
        return 1

    return exit_code


def main():
    if len(sys.argv) < 2:
        print("usage: axiom <command> <file>", file=sys.stderr)
        print("commands:", file=sys.stderr)
        print("  run <file.ax>    compile and run", file=sys.stderr)
        print("  lex <file.ax>    tokenize", file=sys.stderr)
        print("  parse <file.ax>  parse", file=sys.stderr)
        print("  ast <file.ax>    print AST", file=sys.stderr)
        return 1

    command = sys.argv[1]

    if command == "run":
        if len(sys.argv) < 3:
            print("usage: axiom run <file.ax>", file=sys.stderr)
            return 1
        filename = sys.argv[2]
        with open(filename, "r") as f:
            source = f.read()
        return compile_and_run(source, filename)

    if command == "lex":
        if len(sys.argv) < 3:
            print("usage: axiom lex <file.ax>", file=sys.stderr)
            return 1
        filename = sys.argv[2]
        with open(filename, "r") as f:
            source = f.read()
        lexer = Lexer(source, filename)
        tokens = lexer.tokenize()
        for tok in tokens:
            print(tok)
        return 0

    if command == "parse":
        if len(sys.argv) < 3:
            print("usage: axiom parse <file.ax>", file=sys.stderr)
            return 1
        filename = sys.argv[2]
        with open(filename, "r") as f:
            source = f.read()
        lexer = Lexer(source, filename)
        tokens = lexer.tokenize()
        parser = Parser(tokens, filename)
        try:
            ast = parser.parse_program()
            print("parse successful")
            return 0
        except Exception as e:
            print(f"parse error: {e}", file=sys.stderr)
            return 1

    print(f"unknown command: {command}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
