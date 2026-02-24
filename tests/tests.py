class Main:
    def __init__(self, a, b):
        self.a = a 
        self.b = b

    def add(self) -> int:
        return self.a + self.b # adds two values a and b

    def print_values(self) -> None:
        """
        Prints values 0 to 3
        """
        for i in range(4):
            print(i)

        return None

    def syntax_error(self):
        """
        This should produce a syntax error for LSP testing (future)
        """

