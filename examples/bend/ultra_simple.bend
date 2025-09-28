# Ultra Simple Bend Test
# Basic vector operations

# Calculate sum of a list
def sum_list(lst: List(u24)):
  if lst == []:
    return 0
  else:
    (head, tail) = lst
    return head + sum_list(tail)

# Calculate product of a list
def product_list(lst: List(u24)):
  if lst == []:
    return 1
  else:
    (head, tail) = lst
    return head * product_list(tail)

# Generate a simple list
def generate_list(size: u24):
  if size == 0:
    return []
  else:
    return [size] + generate_list(size - 1)

# Main function
def main():
  # Create a test list
  test_list = generate_list(5)
  
  # Calculate sum and product
  sum_result = sum_list(test_list)
  product_result = product_list(test_list)
  
  # Return sum + product
  return sum_result + product_result
