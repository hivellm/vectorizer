# Simple Vector Operations with Bend
# Demonstrates parallel vector operations

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
    return newton_sqrt(x, x / 2.0, 3)

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

# Generate a simple test vector
def generate_vector(size: u24, seed: Float):
  if size == 0:
    return []
  else:
    next_seed = (seed * 1.618) % 1.0
    value = (next_seed - 0.5) * 2.0
    return [value] + generate_vector(size - 1, next_seed)

# Main function
def main():
  # Create test vectors
  vector_a = generate_vector(3, 0.1)
  vector_b = generate_vector(3, 0.2)
  
  # Calculate similarity
  similarity = cosine_similarity(vector_a, vector_b)
  
  # Return similarity as integer (multiplied by 100 for visibility)
  return (similarity * 100.0)
