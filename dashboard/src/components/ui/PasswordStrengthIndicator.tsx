/**
 * Password Strength Indicator — console design language.
 *
 * Public API preserved: `{ password, showRequirements?, onValidationChange? }`.
 * The strength bar uses the console `Bar` primitive; the requirement
 * checklist is rendered with console palette tokens (no Tailwind).
 */

import { useState, useEffect, useCallback } from 'react';
import { Bar } from '@/components/console';

interface PasswordStrengthResult {
  valid: boolean;
  errors: string[];
  strength: number;
  strength_label: string;
}

interface PasswordStrengthIndicatorProps {
  password: string;
  showRequirements?: boolean;
  onValidationChange?: (isValid: boolean) => void;
}

// Get API base URL
const getApiBaseUrl = () => {
  if (import.meta.env.DEV) {
    return 'http://localhost:15002';
  }
  return '';
};

// Password requirements for client-side checking
const requirements = [
  { label: 'At least 8 characters', test: (p: string) => p.length >= 8 },
  { label: 'One uppercase letter', test: (p: string) => /[A-Z]/.test(p) },
  { label: 'One lowercase letter', test: (p: string) => /[a-z]/.test(p) },
  { label: 'One number', test: (p: string) => /[0-9]/.test(p) },
];

function strengthTone(strength: number): 'teal' | 'magenta' | 'amber' {
  if (strength >= 60) return 'teal';
  if (strength >= 40) return 'magenta';
  return 'amber';
}

function strengthColor(strength: number): string {
  if (strength >= 60) return 'var(--teal)';
  if (strength >= 40) return 'var(--magenta)';
  return 'var(--amber)';
}

export function PasswordStrengthIndicator({
  password,
  showRequirements = true,
  onValidationChange,
}: PasswordStrengthIndicatorProps) {
  const [result, setResult] = useState<PasswordStrengthResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // Client-side fallback validation
  const validateClientSide = (pwd: string): PasswordStrengthResult => {
    const errors: string[] = [];
    let strength = 0;

    if (pwd.length < 8) {
      errors.push('Password must be at least 8 characters long');
    } else {
      strength += Math.min((pwd.length - 8) * 5, 25);
    }

    if (!/[A-Z]/.test(pwd)) {
      errors.push('Password must contain at least one uppercase letter');
    } else {
      strength += 15;
    }

    if (!/[a-z]/.test(pwd)) {
      errors.push('Password must contain at least one lowercase letter');
    } else {
      strength += 15;
    }

    if (!/[0-9]/.test(pwd)) {
      errors.push('Password must contain at least one number');
    } else {
      strength += 15;
    }

    if (/[^a-zA-Z0-9]/.test(pwd)) {
      strength += 20;
    }

    strength = Math.min(strength, 100);

    let strengthLabel = 'Very Weak';
    if (strength > 80) strengthLabel = 'Very Strong';
    else if (strength > 60) strengthLabel = 'Strong';
    else if (strength > 40) strengthLabel = 'Fair';
    else if (strength > 20) strengthLabel = 'Weak';

    return {
      valid: errors.length === 0,
      errors,
      strength,
      strength_label: strengthLabel,
    };
  };

  // Debounced validation against the API
  const validatePassword = useCallback(
    async (pwd: string) => {
      if (!pwd || pwd.length < 4) {
        setResult(null);
        onValidationChange?.(false);
        return;
      }

      setIsLoading(true);
      try {
        const response = await fetch(`${getApiBaseUrl()}/auth/validate-password`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ password: pwd }),
        });

        if (response.ok) {
          const data: PasswordStrengthResult = await response.json();
          setResult(data);
          onValidationChange?.(data.valid);
        }
      } catch {
        // Fallback to client-side validation if API fails
        const clientResult = validateClientSide(pwd);
        setResult(clientResult);
        onValidationChange?.(clientResult.valid);
      } finally {
        setIsLoading(false);
      }
    },
    [onValidationChange],
  );

  // Debounce the API call
  useEffect(() => {
    const timer = setTimeout(() => {
      validatePassword(password);
    }, 300);

    return () => clearTimeout(timer);
  }, [password, validatePassword]);

  if (!password || password.length < 4) {
    return null;
  }

  const strength = result?.strength ?? 0;

  return (
    <div style={{ marginTop: 8, display: 'flex', flexDirection: 'column', gap: 8 }}>
      {/* Strength Bar + label */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <div style={{ flex: 1 }}>
          <Bar
            percent={strength}
            tone={strengthTone(strength)}
            ariaLabel={`Password strength: ${result?.strength_label ?? ''}`}
          />
        </div>
        {result && (
          <span
            style={{
              fontSize: 11,
              fontWeight: 500,
              color: strengthColor(result.strength),
              minWidth: 80,
              textAlign: 'right',
            }}
          >
            {isLoading ? '...' : result.strength_label}
          </span>
        )}
      </div>

      {/* Requirements Checklist */}
      {showRequirements && (
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(2, 1fr)',
            gap: 4,
          }}
        >
          {requirements.map((req, index) => {
            const passed = req.test(password);
            return (
              <div
                key={index}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 6,
                  fontSize: 11,
                  color: passed ? 'var(--teal)' : 'var(--text-3)',
                }}
              >
                {passed ? (
                  <svg
                    width={14}
                    height={14}
                    fill="currentColor"
                    viewBox="0 0 20 20"
                    aria-hidden
                  >
                    <path
                      fillRule="evenodd"
                      d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                      clipRule="evenodd"
                    />
                  </svg>
                ) : (
                  <svg
                    width={14}
                    height={14}
                    fill="currentColor"
                    viewBox="0 0 20 20"
                    aria-hidden
                  >
                    <path
                      fillRule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                      clipRule="evenodd"
                    />
                  </svg>
                )}
                <span>{req.label}</span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

export default PasswordStrengthIndicator;
