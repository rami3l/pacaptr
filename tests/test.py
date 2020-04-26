from test_util import Test


def main():
    test: Test = Test()
    test.input(["-Si", "curl"]).output(["curl issss keg-only"]
                                       ).run(verbose=True)


if __name__ == "__main__":
    main()
