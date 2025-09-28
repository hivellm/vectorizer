# Basic Bend Test
# Simple recursive function

# Calculate factorial
def factorial(n: u24):
  if n == 0:
    return 1
  else:
    return n * factorial(n - 1)

# Calculate sum of numbers from 1 to n
def sum_to_n(n: u24):
  if n == 0:
    return 0
  else:
    return n + sum_to_n(n - 1)

# Main function
def main():
  # Calculate factorial of 5
  fact = factorial(5)
  
  # Calculate sum from 1 to 5
  sum_val = sum_to_n(5)
  
  # Return factorial + sum
  return fact + sum_val
