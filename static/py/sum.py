import sys

def sum(a, b):
    return a + b

try:
    a = float(sys.argv[1])
    b = float(sys.argv[2])

    print(sum(a, b))

except Exception as e:
    print("execution failed:", e)
    sys.exit()