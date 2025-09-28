# Bend Vectorizer Integration Test
# This demonstrates real vector operations with Bend

# Calculate dot product of two vectors
def dot_product(a: List(Float), b: List(Float)):
  if a == []:
    return 0.0
  else:
    (a_head, a_tail) = a
    (b_head, b_tail) = b
    return (a_head * b_head) + dot_product(a_tail, b_tail)

# Calculate vector magnitude
def magnitude(v: List(Float)):
  if v == []:
    return 0.0
  else:
    (head, tail) = v
    return sqrt((head * head) + magnitude_squared(tail))

# Calculate sum of squares for magnitude
def magnitude_squared(v: List(Float)):
  if v == []:
    return 0.0
  else:
    (head, tail) = v
    return (head * head) + magnitude_squared(tail)

# Simple square root approximation
def sqrt(x: Float):
  if x < 0.0:
    return 0.0
  else:
    return newton_sqrt(x, x / 2.0, 5)

# Newton's method for square root
def newton_sqrt(x: Float, guess: Float, iterations: u24):
  if iterations == 0:
    return guess
  else:
    new_guess = (guess + x / guess) / 2.0
    return newton_sqrt(x, new_guess, iterations - 1)

# Calculate cosine similarity
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

# Real test vectors (3D for simplicity)
def query_vector():
  return [1.0, 0.0, 0.0]

def test_vectors():
  return [
    [1.0, 0.0, 0.0],    # Same as query (similarity = 1.0)
    [0.0, 1.0, 0.0],    # Orthogonal (similarity = 0.0)
    [0.0, 0.0, 1.0],    # Orthogonal (similarity = 0.0)
    [0.707, 0.707, 0.0], # 45 degrees (similarity ≈ 0.707)
    [0.5, 0.5, 0.707],   # Mixed direction
    [1.0, 1.0, 0.0],     # Same plane as query
    [0.0, 0.0, 0.0],     # Zero vector
    [2.0, 0.0, 0.0],     # Scaled query
  ]

# Simple similarity test
def test_similarity():
  query = query_vector()
  vectors = test_vectors()
  
  # Test first vector (should be identical to query)
  first_vector = [1.0, 0.0, 0.0]
  similarity = cosine_similarity(query, first_vector)
  
  # Return similarity multiplied by 1000 for visibility
  return (similarity * 1000.0)

# Main function
def main():
  return test_similarity()
