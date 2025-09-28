# Simple Vector Similarity Test
# Basic cosine similarity calculation

# Calculate dot product
def dot_product(a: List(Float), b: List(Float)):
  if a == []:
    return 0.0
  else:
    (a_head, a_tail) = a
    (b_head, b_tail) = b
    return (a_head * b_head) + dot_product(a_tail, b_tail)

# Calculate magnitude
def magnitude(v: List(Float)):
  if v == []:
    return 0.0
  else:
    (head, tail) = v
    return sqrt((head * head) + magnitude_squared(tail))

# Calculate sum of squares
def magnitude_squared(v: List(Float)):
  if v == []:
    return 0.0
  else:
    (head, tail) = v
    return (head * head) + magnitude_squared(tail)

# Square root
def sqrt(x: Float):
  if x < 0.0:
    return 0.0
  else:
    return newton_sqrt(x, x / 2.0, 3)

# Newton's method
def newton_sqrt(x: Float, guess: Float, iterations: u24):
  if iterations == 0:
    return guess
  else:
    new_guess = (guess + x / guess) / 2.0
    return newton_sqrt(x, new_guess, iterations - 1)

# Cosine similarity
def cosine_similarity(a: List(Float), b: List(Float)):
  dot = dot_product(a, b)
  mag_a = magnitude(a)
  mag_b = magnitude(b)
  if mag_a == 0.0:
    return 0.0
  else:
    if mag_b == 0.0:
      return 0.0
    else:
      return dot / (mag_a * mag_b)

# Test vectors
def test_similarity():
  vector_a = [1.0, 0.0, 0.0]
  vector_b = [1.0, 0.0, 0.0]  # Same vector
  similarity = cosine_similarity(vector_a, vector_b)
  return (similarity * 1000.0)

# Main function
def main():
  return test_similarity()
