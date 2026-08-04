<!-- RUBY:START -->
# Ruby rules

## Non-negotiables

1. Ruby 3.2+ (prefer 3.3+); `.rubocop.yml` with `TargetRubyVersion` and `NewCops: enable`.
2. `bundle exec rubocop` zero offenses before commit; never silence with inline disables lacking justification.
3. Never auto-correct in CI parity checks — run plain `rubocop` locally like CI, not `rubocop -a`.
4. `bundle exec bundler-audit check --update` after dependency changes.
5. No global variables, no monkey-patching core classes, no `eval`.
6. Commit `Gemfile.lock` for applications; gitignore it for gems.

## Conventions

- snake_case methods/variables, symbols for hash keys, frozen string literals.
- RuboCop plugins: `rubocop-performance` + `rubocop-rspec`; exclude `spec/**/*` from `Metrics/BlockLength`/`MethodLength`.
- Validate inputs with bang methods raising `ArgumentError`; handle exceptions explicitly.
- Use `options.fetch(:key, default)` over `options[:key] ||`.
- Gems: `required_ruby_version = '>= 3.2.0'` in gemspec, code in `lib/`, executables in `exe/`.
- `~>` pessimistic version constraints in Gemfile/gemspec.

## Testing

- RSpec 3.12+ in `spec/` (or Minitest in `test/`); `described_class`, `let`, `describe`/`context` blocks per behavior.
- `spec_helper.rb`: `disable_monkey_patching!`, `order = :random`, `verify_partial_doubles = true`.
- SimpleCov gated behind `COVERAGE=true`, `minimum_coverage 80`, lcov formatter for CI upload.
- Run tests via `bundle exec rspec`; never `focus`-filter in committed code.

## Build & tooling

- Order per iteration: `rubocop` → `rspec` → `COVERAGE=true rspec` → `bundler-audit`; local commands MUST match GitHub Actions exactly.
- CI matrix: Ruby 3.2 + 3.3 on ubuntu/windows/macos.
- Gem release: bump `version.rb`, `gem build *.gemspec`, tag `v1.0.0`, `gem push` (API key via `gem signin`, `RUBYGEMS_API_KEY` secret in CI).
- Never commit `.byebug_history` or debug artifacts.
<!-- RUBY:END -->