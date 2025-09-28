# Simple Bend Test for Vectorizer Integration
# This is a basic test to verify Bend is working

# Simple recursive function
def factorial(n: u24):
  if n == 0:
    return 1
  else:
    return n * factorial(n - 1)

# Main function
def main():
  return factorial(10)
