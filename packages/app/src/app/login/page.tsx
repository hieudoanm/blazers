'use client';

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { BlazersError } from '@blazers/sdk';
import { getBlazers, isAuthed, saveSession } from '@/lib/blazers';

export default function LoginPage() {
  const router = useRouter();
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [otp, setOtp] = useState('');
  const [otpStage, setOtpStage] = useState(false);
  const [mfaStage, setMfaStage] = useState(false);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  if (typeof window !== 'undefined' && isAuthed()) {
    router.replace('/dashboard');
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setError('');
    setBusy(true);
    const bz = getBlazers();
    try {
      if (otpStage) {
        const res = await bz.auth.authWithOtp(email, otp);
        saveSession(res.token);
        router.push('/dashboard');
        return;
      }
      if (mfaStage) {
        const res = await bz.auth.mfaLogin({ email, password, code: otp });
        saveSession(res.token);
        router.push('/dashboard');
        return;
      }
      if (mode === 'register') {
        await bz.auth.register({ email, password });
        const res = await bz.auth.login({ email, password });
        if ('mfa_required' in res && res.mfa_required) {
          setError('Unexpected MFA challenge on fresh account');
          return;
        }
        saveSession((res as { token: string }).token);
        router.push('/dashboard');
        return;
      }
      const res = await bz.auth.login({ email, password });
      if ('mfa_required' in res && res.mfa_required) {
        setMfaStage(true);
        setOtp('');
        return;
      }
      saveSession((res as { token: string }).token);
      router.push('/dashboard');
    } catch (err) {
      setError(err instanceof BlazersError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function requestOtp() {
    setError('');
    setBusy(true);
    try {
      const res = await getBlazers().auth.requestOtp(email);
      setOtpStage(true);
      if (res.dev_otp) setOtp(res.dev_otp);
    } catch (err) {
      setError(err instanceof BlazersError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="mx-auto flex min-h-screen max-w-sm flex-col justify-center px-6">
      <Link
        href="/"
        className="text-base-content/60 mb-6 text-sm hover:underline">
        ← back
      </Link>
      <div className="card border-base-300 bg-base-100 border">
        <div className="card-body">
          <h1 className="card-title text-2xl">
            {otpStage
              ? 'Enter OTP'
              : mfaStage
                ? 'Two-factor auth'
                : mode === 'login'
                  ? 'Sign in'
                  : 'Create account'}
          </h1>

          <form onSubmit={submit} className="flex flex-col gap-3">
            {!otpStage && (
              <>
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Email</legend>
                  <input
                    type="email"
                    className="input border-base-300 w-full"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    required
                  />
                </fieldset>
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Password</legend>
                  <input
                    type="password"
                    className="input border-base-300 w-full"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                  />
                </fieldset>
              </>
            )}
            {(otpStage || mfaStage) && (
              <fieldset className="fieldset">
                <legend className="fieldset-legend">
                  {mfaStage ? 'Authenticator code' : 'One-time code'}
                </legend>
                <input
                  className="input border-base-300 w-full font-mono"
                  value={otp}
                  onChange={(e) => setOtp(e.target.value)}
                  required
                  autoFocus
                />
              </fieldset>
            )}

            {error && <div className="alert alert-error text-sm">{error}</div>}

            <button className="btn btn-primary" disabled={busy}>
              {busy ? '…' : 'Continue'}
            </button>
          </form>

          {!otpStage && !mfaStage && (
            <div className="mt-2 flex flex-col gap-1 text-sm">
              <button
                className="link link-primary text-left"
                onClick={() =>
                  setMode(mode === 'login' ? 'register' : 'login')
                }>
                {mode === 'login'
                  ? 'Need an account? Register'
                  : 'Have an account? Sign in'}
              </button>
              {mode === 'login' && (
                <button
                  className="link link-primary text-left"
                  onClick={requestOtp}>
                  Sign in with email OTP instead
                </button>
              )}
            </div>
          )}
        </div>
      </div>
    </main>
  );
}
