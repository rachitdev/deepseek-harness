/**
 * 10-step browser navigation spike driven by `ctx.decision`.
 *
 * Demonstrates an autonomous browser-stepping loop over 10 consecutive steps:
 * - Each step observes a new page state and dynamic interactive elements.
 * - `decideEgoStep` evaluates candidate clicks alongside ride-along completion, blocking, and risk checks.
 * - Logs per-step latency, confidence probabilities, and escalation behavior.
 */

import { describe, expect, it } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime, { type DecisionRequest, type DecisionResult } from '@deepseek-ai/dsh-decision'
import { decideEgoStep } from '../src/ego-driver.ts'

interface StepLog {
  readonly step: number
  readonly action: string
  readonly confidence: number
  readonly riskLevel: number
  readonly latencyMs: number
  readonly escalated: boolean
}

describe('10-step ego-browser decision spike', () => {
  it('executes a 10-step navigation trajectory with latency and confidence logging', async () => {
    // 10 realistic sequential pages in an e-commerce checkout & verification flow
    const trajectory = [
      {
        page: 'Home page with top deals and search bar',
        candidates: [
          { id: 'search_input', label: 'Search input', description: 'Search catalog' },
          { id: 'deals_banner', label: 'Black Friday Deals', description: 'Promotional banner' },
          { id: 'cart_icon', label: 'Shopping Cart (0)', description: 'View cart' },
        ],
        expectedPick: 'search_input',
        confidence: 0.92,
        risk: 0.0,
      },
      {
        page: 'Search auto-suggest popup showing categories and queries',
        candidates: [
          { id: 'query_laptop', label: 'gaming laptop in Electronics', description: 'Category suggestion' },
          { id: 'query_cable', label: 'usb-c cable in Accessories', description: 'Category suggestion' },
          { id: 'clear_btn', label: 'Clear search', description: 'Reset query' },
        ],
        expectedPick: 'query_cable',
        confidence: 0.88,
        risk: 0.1,
      },
      {
        page: 'Search results for "usb-c cable" (142 products found)',
        candidates: [
          { id: 'filter_length', label: 'Filter: 2m & longer', description: 'Facet checkbox' },
          { id: 'sort_rating', label: 'Sort: Customer Rating', description: 'Dropdown' },
          { id: 'first_product', label: 'Budget USB-C 1m ($4.99)', description: 'Product card' },
        ],
        expectedPick: 'filter_length',
        confidence: 0.84,
        risk: 0.1,
      },
      {
        page: 'Filtered results: 2m & longer USB-C cables (28 products)',
        candidates: [
          { id: 'prod_braided_2m', label: 'Anker Braided USB-C 2m ($12.99)', description: 'Sponsored top rated' },
          { id: 'prod_basic_3m', label: 'Generic 3m Cable ($7.99)', description: 'Standard cable' },
          { id: 'next_page', label: 'Next page (2)', description: 'Pagination link' },
        ],
        expectedPick: 'prod_braided_2m',
        confidence: 0.94,
        risk: 0.2,
      },
      {
        page: 'Product details page: Anker Braided USB-C 2m (In Stock)',
        candidates: [
          { id: 'color_black', label: 'Color: Midnight Black', description: 'Swatch radio button' },
          { id: 'color_white', label: 'Color: Pearl White', description: 'Swatch radio button' },
          { id: 'add_to_cart', label: 'Add to Cart ($12.99)', description: 'Primary action button' },
        ],
        expectedPick: 'add_to_cart',
        confidence: 0.96,
        risk: 0.3,
      },
      {
        page: 'Cart slide-over modal: 1 item added (Subtotal: $12.99)',
        candidates: [
          { id: 'continue_shopping', label: 'Continue Shopping', description: 'Secondary button' },
          { id: 'view_cart', label: 'View Cart & Edit', description: 'Secondary link' },
          { id: 'proceed_checkout', label: 'Proceed to Checkout', description: 'Primary checkout button' },
        ],
        expectedPick: 'proceed_checkout',
        confidence: 0.91,
        risk: 0.4,
      },
      {
        page: 'Checkout Step 1: Shipping Address Selection',
        candidates: [
          { id: 'addr_home', label: 'Deliver to Home (Default: 123 Main St)', description: 'Radio selection' },
          { id: 'addr_new', label: 'Add a new shipping address', description: 'Form toggle' },
          { id: 'ship_continue', label: 'Continue to Delivery Options', description: 'Next step button' },
        ],
        expectedPick: 'addr_home',
        confidence: 0.86,
        risk: 0.4,
      },
      {
        page: 'Checkout Step 2: Shipping Speed & Packaging',
        candidates: [
          { id: 'ship_standard', label: 'Standard Delivery (3-5 days, Free)', description: 'Free tier radio' },
          { id: 'ship_express', label: 'Next Day Air ($9.99)', description: 'Paid tier radio' },
          { id: 'gift_option', label: 'Mark as gift', description: 'Checkbox' },
        ],
        expectedPick: 'ship_standard',
        confidence: 0.89,
        risk: 0.3,
      },
      {
        page: 'Ambiguous Cookie Consent / Third-Party Banner Interstitial',
        candidates: [
          { id: 'accept_all_cookies', label: 'Accept all tracking and ad cookies', description: 'Third party dialog' },
          { id: 'manage_cookies', label: 'Manage cookie preferences', description: 'Preferences link' },
          { id: 'close_banner', label: 'X (Close banner)', description: 'Dismiss overlay' },
        ],
        // Ambiguous interstitial: lower confidence triggers escalation guard!
        expectedPick: 'close_banner',
        confidence: 0.52,
        risk: 0.5,
      },
      {
        page: 'Order Review & Final Confirmation Screen',
        candidates: [
          { id: 'place_order', label: 'Place Your Order ($12.99)', description: 'Final destructive purchase button' },
          { id: 'back_to_cart', label: 'Return to Cart', description: 'Back link' },
          { id: 'promo_code', label: 'Apply Gift Card or Promo Code', description: 'Expandable input' },
        ],
        expectedPick: 'place_order',
        confidence: 0.95,
        risk: 2.1,
      },
    ]

    // Mount decision seam with a scripted mock simulating calibrated provider responses
    let stepIndex = 0
    const ctx = new Context()
    await ctx.plugin(DecisionRuntime, {})

    ctx.decision.registerProvider({
      id: 'mock-jev',
      available: () => true,
      evaluate: (_request: DecisionRequest): Promise<DecisionResult> => {
        const current = trajectory[stepIndex]
        if (!current) throw new Error(`Step index out of range: ${String(stepIndex)}`)

        const probabilities: Record<string, number> = {}
        const otherProb = Math.max(0, (1.0 - current.confidence) / (current.candidates.length - 1))
        for (const c of current.candidates) {
          probabilities[c.id] = c.id === current.expectedPick ? current.confidence : otherProb
        }

        const answers = {
          element: {
            type: 'choice' as const,
            choice: current.expectedPick,
            probabilities,
          },
          task_complete: {
            type: 'boolean' as const,
            probability: stepIndex === trajectory.length - 1 ? 0.95 : 0.05,
          },
          page_blocked: {
            type: 'boolean' as const,
            probability: 0.01,
          },
          risk_level: {
            type: 'score' as const,
            score: current.risk,
            probabilities: { '0': 0.8, '1': 0.2, '2': 0 },
          },
        }

        return Promise.resolve({
          answers,
          providerId: 'mock-jev',
        })
      },
    })

    const logs: StepLog[] = []

    for (let i = 0; i < trajectory.length; i++) {
      stepIndex = i
      const step = trajectory[i]
      if (!step) continue

      const decision = await decideEgoStep(
        ctx.decision,
        {
          taskGoal: 'Purchase 2m Braided USB-C Cable for delivery to home address',
          pageState: step.page,
          candidates: step.candidates,
        },
        { confidenceThreshold: 0.70 },
      )

      logs.push({
        step: i + 1,
        action: decision.action.type === 'click' ? `click(${decision.action.elementId})` : decision.action.type,
        confidence: Math.round(decision.confidence * 100) / 100,
        riskLevel: decision.riskLevel,
        latencyMs: decision.latencyMs,
        escalated: decision.action.type === 'escalate',
      })

      if (i === 8) {
        // Step 9 had confidence 0.52 (< 0.70 threshold) -> correctly escalated!
        expect(decision.action.type).toBe('escalate')
      } else if (i === 9) {
        // Step 10 was final order page -> task complete!
        expect(decision.taskComplete).toBe(true)
      } else {
        expect(decision.action.type).toBe('click')
      }
    }

    // Verify all 10 steps were recorded
    expect(logs).toHaveLength(10)

    // Formatted telemetry output for the run
    console.log('\n--- 10-STEP EGO-BROWSER DECISION TRAJECTORY LOG ---')
    console.table(logs)

    const avgConfidence = logs.reduce((sum, l) => sum + l.confidence, 0) / logs.length
    expect(avgConfidence).toBeGreaterThan(0.80)
    console.log(`Average trajectory confidence: ${(avgConfidence * 100).toFixed(1)}%`)
  })
})
