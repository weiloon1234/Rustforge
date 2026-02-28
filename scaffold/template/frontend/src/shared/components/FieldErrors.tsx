export interface FieldErrorsProps {
  error?: string;
  errors?: string[];
}

export function FieldErrors({ error, errors }: FieldErrorsProps) {
  const all = [
    ...(errors ?? []),
    ...(error && !(errors ?? []).includes(error) ? [error] : []),
  ];
  if (all.length === 0) return null;
  return (
    <>
      {all.map((msg, i) => (
        <p key={i} className="rf-error-message">{msg}</p>
      ))}
    </>
  );
}

export function hasFieldError(error?: string, errors?: string[]): boolean {
  return !!error || (errors != null && errors.length > 0);
}
