// Pure, DOM-free form validators. Each returns a localized error message when the value is
// invalid, or undefined when it is valid, matching TanStack Form's validator contract. They
// live in shared/lib because more than one form needs them, and they are tested without
// mounting any component.
import { m } from '$lib/paraglide/messages';

const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export function emailError(value: string): string | undefined {
	return EMAIL.test(value.trim()) ? undefined : m['login.error_email']();
}

const LOGIN_CODE = /^[A-Z2-9]{8}$/;

export function loginCodeError(value: string): string | undefined {
	return LOGIN_CODE.test(value.trim().toUpperCase()) ? undefined : m['login.code_format']();
}
