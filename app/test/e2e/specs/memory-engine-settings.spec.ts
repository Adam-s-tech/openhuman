// @ts-nocheck
/**
 * E2E — Settings → Memory engine.
 *
 * Drives the panel through real UI against the real core sidecar, so it
 * depends on the core's `openhuman.memory_engines_list` / `memory_engine_get`
 * RPCs (they are not mocked: the mock server only stands in for the cloud
 * backend). No external memory service is contacted: the spec never completes a
 * switch, it only checks the list, the default (local) engine and the form
 * gating, so it stays hermetic.
 */
import { waitForApp } from '../helpers/app-helpers';
import { callOpenhumanRpc } from '../helpers/core-rpc';
import { clickTestId, textExists, waitForTestId, waitForText } from '../helpers/element-helpers';
import { resetApp } from '../helpers/reset-app';
import { navigateViaHash, waitForHomePage } from '../helpers/shared-flows';
import { startMockServer, stopMockServer } from '../mock-server';

const USER_ID = 'e2e-memory-engine';

describe('Memory engine settings panel (real UI flow)', () => {
  before(async function () {
    this.timeout(90_000);
    await startMockServer();
    await waitForApp();
    await resetApp(USER_ID);
    await waitForHomePage(20_000);
  });

  after(async () => {
    await stopMockServer();
  });

  it('lists the engines from the core with the local engine active', async function () {
    this.timeout(60_000);
    const list = await callOpenhumanRpc('openhuman.memory_engines_list', {});
    expect(list.ok).toBe(true);
    const engines = (list.result as { engines?: { id: string }[] } | undefined)?.engines ?? [];
    expect(engines.map(e => e.id)).toContain('tinymemory');

    await navigateViaHash('/settings/memory-engine');
    await waitForTestId('memory-engine-panel', 15_000);
    await waitForTestId('memory-engine-option-tinymemory', 10_000);
    await waitForText('Active', 5_000);
  });

  it('gates Switch until the selected engine has what it needs', async function () {
    this.timeout(60_000);
    await clickTestId('memory-engine-radio-supermemory', 10_000);
    // Supermemory's key is optional; its endpoint is required. Leave the
    // endpoint blank to verify the gate that the core advertises for it.
    const endpoint = await browser.$('#memory-engine-supermemory-endpoint');
    await endpoint.waitForExist({ timeout: 10_000 });
    await browser.execute(() => {
      const el = document.querySelector<HTMLInputElement>(
        '#memory-engine-supermemory-endpoint'
      );
      if (!el) return;
      const setter = Object.getOwnPropertyDescriptor(
        window.HTMLInputElement.prototype,
        'value'
      )?.set;
      if (setter) setter.call(el, '');
      else el.value = '';
      el.dispatchEvent(new Event('input', { bubbles: true }));
      el.dispatchEvent(new Event('change', { bubbles: true }));
    });
    const sw = await waitForTestId('memory-engine-switch', 10_000);
    expect(await sw.isEnabled()).toBe(false);
    expect(await textExists('Endpoint')).toBe(true);
  });

  it('keeps the local engine as the oracle-visible active engine', async () => {
    const state = await callOpenhumanRpc('openhuman.memory_engine_get', {});
    expect(state.ok).toBe(true);
    expect((state.result as { driver?: string } | undefined)?.driver).toBe('tinymemory');
  });
});
