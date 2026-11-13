'use client';

import type { Collection } from '@blazers/sdk';
import { useCallback, useEffect, useState } from 'react';
import { getBlazers } from '@/lib/blazers';

const EMPTY_RULES = {
  list_rule: '',
  view_rule: '',
  create_rule: '',
  update_rule: '',
  delete_rule: '',
};

export default function CollectionsTab() {
  const [collections, setCollections] = useState<Collection[]>([]);
  const [name, setName] = useState('');
  const [schema, setSchema] = useState('{}');
  const [rules, setRules] = useState(EMPTY_RULES);
  const [editing, setEditing] = useState<Collection | null>(null);
  const [error, setError] = useState('');

  const load = useCallback(async () => {
    try {
      setCollections(await getBlazers().collections.list());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  function startEdit(c: Collection) {
    setEditing(c);
    setName(c.name);
    setSchema(c.schema || '{}');
    setRules({
      list_rule: c.list_rule,
      view_rule: c.view_rule,
      create_rule: c.create_rule,
      update_rule: c.update_rule,
      delete_rule: c.delete_rule,
    });
    setError('');
  }

  async function save() {
    setError('');
    try {
      if (editing) {
        await getBlazers().collections.update(editing.name, {
          schema,
          ...rules,
        });
      } else {
        await getBlazers().collections.create(name.trim(), schema);
      }
      setEditing(null);
      setName('');
      setSchema('{}');
      setRules(EMPTY_RULES);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function remove(c: Collection) {
    if (!confirm(`Delete collection "${c.name}" and all its records?`)) return;
    try {
      await getBlazers().collections.delete(c.name);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="grid gap-6 lg:grid-cols-[1fr_380px]">
      <div className="rounded-box border-base-300 bg-base-100 overflow-x-auto border">
        <table className="table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Type</th>
              <th>Rules</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {collections.map((c) => (
              <tr key={c.name}>
                <td className="font-mono text-sm">{c.name}</td>
                <td>
                  <span
                    className={`badge badge-sm ${c.type === 'auth' ? 'badge-info' : 'badge-ghost'}`}>
                    {c.type}
                  </span>
                </td>
                <td className="text-base-content/60 font-mono text-xs">
                  {[
                    c.list_rule,
                    c.view_rule,
                    c.create_rule,
                    c.update_rule,
                    c.delete_rule,
                  ]
                    .filter(Boolean)
                    .join(' · ') || '—'}
                </td>
                <td className="flex gap-1">
                  <button
                    className="btn btn-ghost btn-xs"
                    onClick={() => startEdit(c)}>
                    edit
                  </button>
                  <button
                    className="btn btn-ghost btn-xs text-error"
                    onClick={() => void remove(c)}>
                    del
                  </button>
                </td>
              </tr>
            ))}
            {collections.length === 0 && (
              <tr>
                <td
                  colSpan={4}
                  className="text-base-content/50 py-8 text-center">
                  No collections
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <div className="card border-base-300 bg-base-100 border">
        <div className="card-body gap-3 p-4">
          <h2 className="card-title text-base">
            {editing ? `Edit ${editing.name}` : 'New collection'}
          </h2>
          {!editing && (
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Name</legend>
              <input
                className="input input-sm border-base-300 font-mono"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="posts"
              />
            </fieldset>
          )}
          <fieldset className="fieldset">
            <legend className="fieldset-legend">JSON schema (optional)</legend>
            <textarea
              className="textarea border-base-300 h-20 font-mono text-xs"
              value={schema}
              onChange={(e) => setSchema(e.target.value)}
            />
          </fieldset>
          {(
            [
              'list_rule',
              'view_rule',
              'create_rule',
              'update_rule',
              'delete_rule',
            ] as const
          ).map((key) => (
            <fieldset key={key} className="fieldset">
              <legend className="fieldset-legend font-mono text-xs">
                {key}
              </legend>
              <input
                className="input input-sm border-base-300 font-mono"
                value={rules[key]}
                onChange={(e) => setRules({ ...rules, [key]: e.target.value })}
                placeholder="@request.auth.id != ''"
              />
            </fieldset>
          ))}
          {error && (
            <div className="alert alert-error py-2 text-sm">{error}</div>
          )}
          <div className="flex gap-2">
            <button
              className="btn btn-primary btn-sm"
              onClick={() => void save()}>
              {editing ? 'Save' : 'Create'}
            </button>
            {editing && (
              <button
                className="btn btn-ghost btn-sm"
                onClick={() => {
                  setEditing(null);
                  setName('');
                }}>
                Cancel
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
