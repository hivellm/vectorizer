<!-- RUBY:START -->
# Ruby Project Rules

## Agent Automation Commands

**CRITICAL**: Execute these commands after EVERY implementation (see AGENT_AUTOMATION module for full workflow).

```bash
# Complete quality check sequence:
bundle exec rubocop       # Linting and formatting
bundle exec rspec         # All tests (100% pass)
bundle exec rspec --format documentation  # Test coverage

# Security audit:
bundle audit              # Vulnerability scan
bundle outdated           # Check outdated deps
```

## Ruby Configuration

**CRITICAL**: Use Ruby 3.2+ with RuboCop and modern tooling.

- **Version**: Ruby 3.2+
- **Recommended**: Ruby 3.3+
- **Style Guide**: Ruby Style Guide (RuboCop)
- **Testing**: RSpec (recommended) or Minitest
- **Type Checking**: RBS + Steep (optional but recommended)

### Gemfile Requirements

```ruby
source 'https://rubygems.org'

ruby '>= 3.2.0'

# Production dependencies
gem 'rake', '~> 13.0'

# Development dependencies
group :development do
  gem 'rubocop', '~> 1.60', require: false
  gem 'rubocop-performance', require: false
  gem 'rubocop-rspec', require: false
end

# Test dependencies
group :test do
  gem 'rspec', '~> 3.12'
  gem 'simplecov', require: false
  gem 'simplecov-lcov', require: false
end

# Both development and test
group :development, :test do
  gem 'pry'
  gem 'pry-byebug'
end
```

### Gemspec Requirements (for gems)

```ruby
Gem::Specification.new do |spec|
  spec.name = 'your_gem'
  spec.version = '0.1.0'
  spec.authors = ['Your Name']
  spec.email = ['you@example.com']

  spec.summary = 'Brief summary'
  spec.description = 'Longer description'
  spec.homepage = 'https://github.com/you/your_gem'
  spec.license = 'MIT'
  spec.required_ruby_version = '>= 3.2.0'

  spec.files = Dir.glob('{lib,bin}/**/*') + %w[README.md LICENSE.txt]
  spec.bindir = 'exe'
  spec.executables = spec.files.grep(%r{^exe/}) { |f| File.basename(f) }
  spec.require_paths = ['lib']

  spec.add_dependency 'rake', '~> 13.0'
  
  spec.add_development_dependency 'rspec', '~> 3.12'
  spec.add_development_dependency 'rubocop', '~> 1.60'
end
```

## Code Quality Standards

### Mandatory Quality Checks

**CRITICAL**: After implementing ANY feature, you MUST run these commands in order.

**IMPORTANT**: These commands MUST match your GitHub Actions workflows to prevent CI/CD failures!

```bash
# Pre-Commit Checklist (MUST match .github/workflows/*.yml)

# 1. Lint (MUST pass with no warnings - matches workflow)
bundle exec rubocop

# 2. Run all tests (MUST pass 100% - matches workflow)
bundle exec rspec
# or: bundle exec rake test (for Minitest)

# 3. Check coverage (MUST meet threshold - matches workflow)
COVERAGE=true bundle exec rspec

# 4. Security audit (matches workflow)
bundle exec bundler-audit check --update

# 5. Build gem (if gem project - matches workflow)
gem build *.gemspec

# If ANY fails: ❌ DO NOT COMMIT - Fix first!
```

**If ANY of these fail, you MUST fix the issues before committing.**

**Why This Matters:**
- Running different commands locally than in CI causes "works on my machine" failures
- CI/CD workflows will fail if commands don't match
- Example: Using `rubocop -a` (auto-correct) locally but `rubocop` in CI = failure
- Example: Missing security audit locally = CI catches vulnerabilities in dependencies

### Linting with RuboCop

- Configuration in `.rubocop.yml`
- Must pass with no offenses
- Auto-correct safe offenses only

Example `.rubocop.yml`:
```yaml
require:
  - rubocop-performance
  - rubocop-rspec

AllCops:
  TargetRubyVersion: 3.2
  NewCops: enable
  Exclude:
    - 'vendor/**/*'
    - 'tmp/**/*'
    - 'bin/**/*'

Style/StringLiterals:
  EnforcedStyle: single_quotes

Metrics/MethodLength:
  Max: 15
  Exclude:
    - 'spec/**/*'

Metrics/BlockLength:
  Exclude:
    - 'spec/**/*'
    - '*.gemspec'
```

### Testing

- **Framework**: RSpec (recommended) or Minitest
- **Location**: `/spec` (RSpec) or `/test` (Minitest)
- **Coverage**: SimpleCov (80%+ threshold)
- **Focus**: Write descriptive specs

Example RSpec test:
```ruby
# spec/my_class_spec.rb

RSpec.describe MyClass do
  let(:instance) { described_class.new(value: 'test') }
  
  describe '#process' do
    context 'with valid input' do
      it 'returns processed value' do
        result = instance.process('input')
        expect(result).to eq('PROCESSED: input')
      end
      
      it 'handles empty strings' do
        expect(instance.process('')).to be_nil
      end
    end
    
    context 'with invalid input' do
      it 'raises ArgumentError' do
        expect { instance.process(nil) }.to raise_error(ArgumentError)
      end
    end
  end
  
  describe '#validate' do
    it 'returns true for valid data' do
      expect(instance.validate('valid')).to be true
    end
    
    it 'returns false for invalid data' do
      expect(instance.validate('')).to be false
    end
  end
end
```

Example Minitest:
```ruby
# test/my_class_test.rb

require 'test_helper'

class MyClassTest < Minitest::Test
  def setup
    @instance = MyClass.new(value: 'test')
  end
  
  def test_process_returns_expected_value
    result = @instance.process('input')
    assert_equal 'PROCESSED: input', result
  end
  
  def test_process_handles_empty_strings
    assert_nil @instance.process('')
  end
  
  def test_process_raises_on_nil
    assert_raises(ArgumentError) { @instance.process(nil) }
  end
end
```

### Coverage Configuration

Create `spec/spec_helper.rb`:
```ruby
if ENV['COVERAGE']
  require 'simplecov'
  require 'simplecov-lcov'
  
  SimpleCov::Formatter::LcovFormatter.config.report_with_single_file = true
  SimpleCov.formatter = SimpleCov::Formatter::MultiFormatter.new([
    SimpleCov::Formatter::HTMLFormatter,
    SimpleCov::Formatter::LcovFormatter
  ])
  
  SimpleCov.start do
    add_filter '/spec/'
    add_filter '/test/'
    
    minimum_coverage 80
    minimum_coverage_by_file 70
  end
end

require 'your_gem'

RSpec.configure do |config|
  config.expect_with :rspec do |expectations|
    expectations.include_chain_clauses_in_custom_matcher_descriptions = true
  end

  config.mock_with :rspec do |mocks|
    mocks.verify_partial_doubles = true
  end

  config.shared_context_metadata_behavior = :apply_to_host_groups
  config.filter_run_when_matching :focus
  config.example_status_persistence_file_path = 'spec/examples.txt'
  config.disable_monkey_patching!
  config.warnings = true
  
  config.default_formatter = 'doc' if config.files_to_run.one?
  config.profile_examples = 10
  config.order = :random
  Kernel.srand config.seed
end
```

## Dependency Management

### Using Bundler

```bash
# Install dependencies
bundle install

# Update dependencies
bundle update

# Check for outdated gems
bundle outdated

# Security audit
bundle exec bundler-audit check --update
```

### Gemfile.lock

- **MUST** commit Gemfile.lock for applications
- For gems: Add to `.gitignore`
- Ensures reproducible builds

## Best Practices

### DO's ✅

- **USE** meaningful variable and method names
- **FOLLOW** Ruby naming conventions (snake_case)
- **WRITE** descriptive tests with context blocks
- **HANDLE** exceptions explicitly
- **VALIDATE** inputs
- **DOCUMENT** public APIs
- **USE** symbols for hash keys when possible
- **FREEZE** string literals in Ruby 3+

### DON'Ts ❌

- **NEVER** use global variables
- **NEVER** monkey-patch core classes without extreme caution
- **NEVER** skip tests
- **NEVER** commit `.byebug_history` or debug files
- **NEVER** use `eval` unless absolutely necessary
- **NEVER** ignore RuboCop offenses without justification
- **NEVER** commit with failing tests

Example code style:
```ruby
# ✅ GOOD: Clean Ruby code
class DataProcessor
  def initialize(options = {})
    @threshold = options.fetch(:threshold, 0.5)
    @verbose = options.fetch(:verbose, false)
  end
  
  def process(data)
    validate_input!(data)
    
    log('Processing data...') if @verbose
    
    data.select { |item| item[:value] > @threshold }
  end
  
  private
  
  def validate_input!(data)
    raise ArgumentError, 'Data must be an array' unless data.is_a?(Array)
    raise ArgumentError, 'Data cannot be empty' if data.empty?
  end
  
  def log(message)
    puts "[#{Time.now.iso8601}] #{message}"
  end
end

# ❌ BAD: Poor practices
class DataProcessor
  def process(data)
    $threshold = 0.5  # DON'T use globals!
    
    if data == nil  # Use nil? method
      return false
    end
    
    result = []
    for item in data  # Use .each or .map
      if item[:value] > $threshold
        result.push(item)
      end
    end
    
    puts result  # DON'T print in library code
    result
  end
end
```

## CI/CD Requirements

Must include GitHub Actions workflows:

1. **Testing** (`ruby-test.yml`):
   - Test on ubuntu-latest, windows-latest, macos-latest
   - Ruby versions: 3.2, 3.3
   - Upload coverage to Codecov

2. **Linting** (`ruby-lint.yml`):
   - RuboCop checks
   - Bundler audit for security

3. **Build** (`ruby-build.yml`):
   - Build gem
   - Verify gem structure

## Publishing to RubyGems

### Prerequisites

1. Create account at https://rubygems.org
2. Get API key: `gem signin`
3. Add `RUBYGEMS_API_KEY` to GitHub secrets

### Publishing Workflow

```bash
# 1. Update version in gemspec or version.rb
# 2. Update CHANGELOG.md
# 3. Run all quality checks
bundle exec rubocop
bundle exec rspec
gem build *.gemspec

# 4. Create git tag
git tag -a v1.0.0 -m "Release version 1.0.0"

# 5. Push (manual if SSH password)
# git push origin main
# git push origin v1.0.0

# 6. Publish to RubyGems (or use GitHub Actions)
gem push your_gem-1.0.0.gem
```

<!-- RUBY:END -->