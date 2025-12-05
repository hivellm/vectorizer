/**
 * Password Strength Indicator Component
 *
 * Shows visual feedback on password strength with:
 * - Strength bar with color gradient
 * - Strength label
 * - Requirements checklist
 */

import { useState, useEffect, useCallback } from 'react';

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

export function PasswordStrengthIndicator({
  password,
  showRequirements = true,
  onValidationChange,
}: PasswordStrengthIndicatorProps) {
  const [result, setResult] = useState<PasswordStrengthResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // Debounced validation against the API
  const validatePassword = useCallback(async (pwd: string) => {
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
  }, [onValidationChange]);

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

  // Debounce the API call
  useEffect(() => {
    const timer = setTimeout(() => {
      validatePassword(password);
    }, 300);

    return () => clearTimeout(timer);
  }, [password, validatePassword]);

  // Get color based on strength
  const getStrengthColor = (strength: number) => {
    if (strength >= 80) return 'bg-green-500';
    if (strength >= 60) return 'bg-blue-500';
    if (strength >= 40) return 'bg-yellow-500';
    if (strength >= 20) return 'bg-orange-500';
    return 'bg-red-500';
  };

  const getStrengthTextColor = (strength: number) => {
    if (strength >= 80) return 'text-green-600 dark:text-green-400';
    if (strength >= 60) return 'text-blue-600 dark:text-blue-400';
    if (strength >= 40) return 'text-yellow-600 dark:text-yellow-400';
    if (strength >= 20) return 'text-orange-600 dark:text-orange-400';
    return 'text-red-600 dark:text-red-400';
  };

  if (!password || password.length < 4) {
    return null;
  }

  return (
    <div className="mt-2 space-y-2">
      {/* Strength Bar */}
      <div className="flex items-center gap-2">
        <div className="flex-1 h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div
            className={`h-full transition-all duration-300 ${result ? getStrengthColor(result.strength) : 'bg-gray-400'}`}
            style={{ width: `${result?.strength || 0}%` }}
          />
        </div>
        {result && (
          <span className={`text-xs font-medium ${getStrengthTextColor(result.strength)}`}>
            {isLoading ? '...' : result.strength_label}
          </span>
        )}
      </div>

      {/* Requirements Checklist */}
      {showRequirements && (
        <div className="grid grid-cols-2 gap-1">
          {requirements.map((req, index) => {
            const passed = req.test(password);
            return (
              <div
                key={index}
                className={`flex items-center gap-1.5 text-xs ${
                  passed
                    ? 'text-green-600 dark:text-green-400'
                    : 'text-gray-400 dark:text-gray-500'
                }`}
              >
                {passed ? (
                  <svg className="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
                    <path
                      fillRule="evenodd"
                      d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                      clipRule="evenodd"
                    />
                  </svg>
                ) : (
                  <svg className="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
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
