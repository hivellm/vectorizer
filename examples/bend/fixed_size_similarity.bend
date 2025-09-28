# Simple Vector Similarity Test
# Using basic operations without complex list handling

# Calculate dot product for fixed-size vectors
def dot_product_3d(a1, a2, a3, b1, b2, b3):
  return (a1 * b1) + (a2 * b2) + (a3 * b3)

# Calculate magnitude for 3D vector
def magnitude_3d(x, y, z):
  return sqrt((x * x) + (y * y) + (z * z))

# Square root
def sqrt(x):
  if x < 0.0:
    return 0.0
  else:
    return newton_sqrt(x, x / 2.0, 3)

# Newton's method
def newton_sqrt(x, guess, iterations):
  if iterations == 0:
    return guess
  else:
    new_guess = (guess + x / guess) / 2.0
    return newton_sqrt(x, new_guess, iterations - 1)

# Cosine similarity for 3D vectors
def cosine_similarity_3d(a1, a2, a3, b1, b2, b3):
  dot = dot_product_3d(a1, a2, a3, b1, b2, b3)
  mag_a = magnitude_3d(a1, a2, a3)
  mag_b = magnitude_3d(b1, b2, b3)
  if mag_a == 0.0:
    return 0.0
  else:
    if mag_b == 0.0:
      return 0.0
    else:
      return dot / (mag_a * mag_b)

# Test similarity
def test_similarity():
  # Test identical vectors (should return 1.0)
  similarity = cosine_similarity_3d(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
  return (similarity * 1000.0)

# Main function
def main():
  return test_similarity()
