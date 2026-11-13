import type { HttpClient } from './http.js';
import type { LoginResponse, LoginResult, User } from './types.js';

export interface Credentials {
  email: string;
  password: string;
}

export interface UpdateMeOptions {
  data?: Record<string, unknown>;
  /** Requires `currentPassword` when the account already has a password. */
  password?: string;
  currentPassword?: string;
}

export interface MfaSetup {
  secret: string;
  otpauth_url: string;
}

/**
 * Authentication client — mirrors `/api/auth/*`.
 * On successful login/refresh the session token is stored on the shared
 * `HttpClient`, so every subsequent request is authenticated automatically.
 */
export class AuthClient {
  constructor(private readonly http: HttpClient) {}

  /** Creates the account and returns the user — call {@link login} next. */
  async register(creds: Credentials): Promise<User> {
    return this.http.post<User>('/api/auth/register', creds);
  }

  /**
   * Email + password login. When the account has MFA enabled the response is
   * `{ mfa_required: true, user }` — finish with {@link mfaLogin}.
   */
  async login(creds: Credentials): Promise<LoginResult> {
    const res = await this.http.post<LoginResult>('/api/auth/login', creds);
    if (!('mfa_required' in res && res.mfa_required)) {
      this.http.setToken((res as LoginResponse).token);
    }
    return res;
  }

  /** Password + TOTP code — completes an MFA login challenge. */
  async mfaLogin(
    creds: Credentials & { code: string }
  ): Promise<LoginResponse> {
    const res = await this.http.post<LoginResponse>('/api/auth/mfa/login', {
      email: creds.email,
      password: creds.password,
      code: creds.code,
    });
    this.http.setToken(res.token);
    return res;
  }

  /** Exchange the current token for a fresh one (PocketBase `auth-refresh`). */
  async refresh(): Promise<LoginResponse> {
    const res = await this.http.post<LoginResponse>('/api/auth/refresh');
    this.http.setToken(res.token);
    return res;
  }

  /** Clears the in-memory session token. */
  logout(): void {
    this.http.setToken(undefined);
  }

  me(): Promise<User> {
    return this.http.get<User>('/api/auth/me');
  }

  updateMe(opts: UpdateMeOptions): Promise<User> {
    return this.http.patch<User>('/api/auth/me', {
      data: opts.data,
      password: opts.password,
      current_password: opts.currentPassword,
    });
  }

  /** Sends a password-reset email; response may include `dev_token` w/o SMTP. */
  forgotPassword(
    email: string
  ): Promise<{ message?: string; dev_token?: string }> {
    return this.http.post('/api/auth/forgot-password', { email });
  }

  resetPassword(token: string, password: string): Promise<unknown> {
    return this.http.post('/api/auth/reset-password', { token, password });
  }

  /** Sends a verification email; response may include `dev_token` w/o SMTP. */
  requestVerification(
    email: string
  ): Promise<{ message?: string; dev_token?: string }> {
    return this.http.post('/api/auth/verify-email', { email });
  }

  confirmVerification(token: string): Promise<unknown> {
    return this.http.post('/api/auth/verify-email/confirm', { token });
  }

  /** Sends a 6-digit OTP; response may include `dev_otp` w/o SMTP. */
  requestOtp(email: string): Promise<{ message?: string; dev_otp?: string }> {
    return this.http.post('/api/auth/request-otp', { email });
  }

  authWithOtp(email: string, otp: string): Promise<LoginResponse> {
    return this.http
      .post<LoginResponse>('/api/auth/auth-with-otp', { email, otp })
      .then((res) => {
        this.http.setToken(res.token);
        return res;
      });
  }

  /** Browser redirect URL that starts the OAuth2 flow for `provider`. */
  oauthUrl(provider: string): string {
    return this.http.url(`/api/auth/oauth/${encodeURIComponent(provider)}`);
  }

  // ---- MFA / TOTP ----

  /** Begin TOTP enrollment — returns the secret + otpauth URI to scan. */
  mfaEnable(): Promise<MfaSetup> {
    return this.http.post<MfaSetup>('/api/auth/mfa/enable');
  }

  /** Confirm enrollment with a current TOTP code. */
  mfaConfirm(code: string): Promise<unknown> {
    return this.http.post('/api/auth/mfa/confirm', { code });
  }

  mfaDisable(code: string): Promise<unknown> {
    return this.http.post('/api/auth/mfa/disable', { code });
  }
}
