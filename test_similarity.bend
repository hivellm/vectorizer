# Test Bend Vector Similarity
# Simple test with 2 vectors

def dot_product(a: List(Float), b: List(Float)):
  case a:
    []:
      return 0.0
    (a_head, a_tail):
      case b:
        []:
          return 0.0
        (b_head, b_tail):
          return (a_head * b_head) + dot_product(a_tail, b_tail)

def magnitude(v: List(Float)):
  case v:
    []:
      return 0.0
    (head, tail):
      return sqrt((head * head) + magnitude_squared(tail))

def magnitude_squared(v: List(Float)):
  case v:
    []:
      return 0.0
    (head, tail):
      return (head * head) + magnitude_squared(tail)

def sqrt(x: Float):
  if x < 0.0:
    return 0.0
  else:
    return newton_sqrt(x, x / 2.0, 3)

def newton_sqrt(x: Float, guess: Float, iterations: u24):
  if iterations == 0:
    return guess
  else:
    new_guess = (guess + x / guess) / 2.0
    return newton_sqrt(x, new_guess, iterations - 1)

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

def query_vector():
  return [1.0, 0.0, 0.0]

def get_vectors():
  return [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0]
  ]

def parallel_similarity_search(vectors: List(List(Float)), threshold: Float):
  case vectors:
    []:
      return []
    (head, tail):
      similarity = cosine_similarity(query_vector(), head)
      if similarity >= threshold:
        remaining_results = parallel_similarity_search(tail, threshold)
        return [similarity] + remaining_results
      else:
        return parallel_similarity_search(tail, threshold)

def main():
  vectors = get_vectors()
  results = parallel_similarity_search(vectors, 0.1)
  return length(results)

def length(list):
  case list:
    []:
      return 0
    (head, tail):
      return 1 + length(tail)