# CBOR-Web Generative & Commerce Specification v2.1

**Companion to:** CBOR-Web Core Specification v2.1 (CBOR-WEB-CORE.md)
**Status:** Draft
**Date:** 2026-03-24
**Authors:** ExploDev (Eddie Plot, Claude)

---

## 1. Overview

This document defines generative content blocks, forms, commerce structures, and site capability declarations. Generative blocks enable an AI agent to go beyond reading — they provide structured intelligence for creating, acting, and transacting.

**Core principle:** Generative blocks carry explicit trust levels (CBOR-WEB-SECURITY.md §8). An agent MUST enforce its trust policy before processing any generative block.

---

## 2. Generative Content Blocks

Generative blocks appear in page key 7 (`generative` array). They are separate from editorial content (key 4) to allow agents to filter them independently.

### 2.1 Block Summary

| Code | Type | Trust | Section | Description |
|------|------|-------|---------|-------------|
| `"schema"` | Schema | 0 | §3 | Data structure description |
| `"constraint"` | Constraint | 0 | §4 | Business rule declaration |
| `"template"` | Template | 1 | §5 | Variable substitution pattern |
| `"executable"` | Executable | 2 | §6 | Sandboxed code |
| `"api_endpoint"` | API Endpoint | 3 | §7 | Network API call |
| `"workflow"` | Workflow | 3 | §8 | Multi-step autonomous process |

---

## 3. Schema Block (Trust 0)

Describes a data structure. Pure declaration — no execution, no side effects.

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"schema"` |
| `"name"` | text | REQUIRED | Schema name |
| `"schema"` | map | REQUIRED | JSON Schema-compatible structure |
| `"description"` | text | OPTIONAL | Human-readable description |
| `"examples"` | array | OPTIONAL | Example instances |

```cbor-diag
{
  "t": "schema",
  "name": "Product",
  "description": "Schema for a Verdetao product",
  "schema": {
    "type": "object",
    "required": ["name", "price", "currency"],
    "properties": {
      "name": {"type": "string"},
      "price": {"type": "number"},
      "currency": {"type": "string", "enum": ["EUR", "USD", "GBP"]},
      "stock": {"type": "integer", "minimum": 0},
      "certifications": {"type": "array", "items": {"type": "string"}}
    }
  },
  "examples": [
    {"name": "Lion's Mane", "price": 29.90, "currency": "EUR", "stock": 142, "certifications": ["Bio EU", "Vegan"]}
  ]
}
```

**Agent use:** An agent reading this schema knows the exact structure of product data. It can validate data, generate queries, or build integrations without guessing field names.

---

## 4. Constraint Block (Trust 0)

Declares a business rule. Constraints tell agents what is allowed, what limits exist, and what conditions apply.

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"constraint"` |
| `"name"` | text | REQUIRED | Constraint name |
| `"condition"` | text | REQUIRED | Condition expression (see §4.1) |
| `"message"` | text | REQUIRED | Human-readable explanation |
| `"scope"` | text | OPTIONAL | What this applies to: `"order"`, `"product"`, `"user"`, `"global"` |

```cbor-diag
{
  "t": "constraint",
  "name": "minimum_order",
  "scope": "order",
  "message": "Minimum order is 2 items for wholesale customers",
  "condition": "order.quantity >= 2"
}
```

### 4.1 Condition Expression Language

Constraints use a minimal expression language. The grammar is intentionally limited to keep trust at level 0 (no execution, only evaluation).

**Supported operators:**

| Operator | Meaning | Example |
|----------|---------|---------|
| `==` | Equal | `product.currency == "EUR"` |
| `!=` | Not equal | `order.status != "cancelled"` |
| `>`, `>=`, `<`, `<=` | Comparison | `order.quantity >= 2` |
| `AND` | Logical AND | `product.stock > 0 AND product.active == true` |
| `OR` | Logical OR | `user.country == "FR" OR user.country == "ES"` |
| `IN` | Membership | `product.category IN ["supplements", "food"]` |
| `NOT` | Negation | `NOT product.discontinued` |

**Dot notation** accesses nested fields: `order.items.count`, `product.price.value`.

**No function calls, no arithmetic, no assignments.** This is a predicate language, not a programming language. An agent evaluates constraints to boolean true/false.

**EBNF grammar:**

```ebnf
expression = comparison ((AND | OR) comparison)*
comparison = value (operator value)?
           | NOT comparison
           | value IN array_literal
operator   = "==" | "!=" | ">" | ">=" | "<" | "<="
value      = field_path | string_literal | number_literal | boolean_literal
field_path = identifier ("." identifier)*
string_literal = '"' [^"]* '"'
number_literal = [0-9]+ ("." [0-9]+)?
boolean_literal = "true" | "false"
array_literal = "[" (value ("," value)*)? "]"
identifier = [a-zA-Z_][a-zA-Z0-9_]*
```

---

## 5. Template Block (Trust 1)

Contains a text template with variable substitution. Output depends on context provided by the agent.

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"template"` |
| `"trust"` | uint | IMPLICIT | 1 |
| `"name"` | text | REQUIRED | Template name |
| `"template"` | text | REQUIRED | Template string with `{variable}` placeholders |
| `"variables"` | map | REQUIRED | Variable definitions (name → type + description) |
| `"output_type"` | text | OPTIONAL | Expected output: `"text"`, `"html"`, `"email"`, `"json"` |

```cbor-diag
{
  "t": "template",
  "name": "order_confirmation",
  "template": "Bonjour {customer_name},\n\nVotre commande #{order_id} de {product_count} articles pour {total_price} EUR a été confirmée.\n\nLivraison estimée : {delivery_date}.\n\nMerci,\nL'équipe {site_name}",
  "variables": {
    "customer_name": {"type": "string", "description": "Customer full name"},
    "order_id": {"type": "string", "description": "Order reference number"},
    "product_count": {"type": "integer", "description": "Number of items"},
    "total_price": {"type": "number", "description": "Total in EUR"},
    "delivery_date": {"type": "string", "description": "Estimated delivery date"},
    "site_name": {"type": "string", "description": "Site name", "default": "Verdetao"}
  },
  "output_type": "text"
}
```

### 5.1 Template Syntax

| Syntax | Meaning | Example |
|--------|---------|---------|
| `{variable}` | Simple substitution | `{customer_name}` → `"Jean Dupont"` |
| `{if condition}...{endif}` | Conditional block | `{if order.express}Livraison express{endif}` |
| `{for item in collection}...{endfor}` | Loop | `{for p in products}{p.name}: {p.price} EUR\n{endfor}` |
| `{variable\|default:"value"}` | Default value | `{phone\|default:"non renseigné"}` |

**Security:** Templates MUST NOT contain executable code. A template renderer MUST NOT evaluate expressions beyond variable substitution, conditionals, and loops. No function calls, no import statements, no file access.

An agent MUST validate template syntax before rendering. A malformed template SHOULD be rejected with a warning, not crash the agent.

---

## 6. Executable Block (Trust 2)

Contains code intended for sandboxed execution. The agent runs this code in an isolated environment.

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"executable"` |
| `"trust"` | uint | REQUIRED | `2` |
| `"lang"` | text | REQUIRED | Language: `"python"`, `"javascript"`, `"wasm"` |
| `"code"` | text | REQUIRED | Source code |
| `"description"` | text | REQUIRED | What this code does (agent reads this first) |
| `"inputs"` | map | OPTIONAL | Expected input parameters |
| `"outputs"` | map | OPTIONAL | Expected output structure |
| `"sandbox"` | map | OPTIONAL | Sandbox requirements |

```cbor-diag
{
  "t": "executable",
  "lang": "python",
  "code": "def calculate_shipping(weight_kg, country):\n    base = 5.0\n    if country in ['FR', 'ES', 'DE']:\n        return base + weight_kg * 0.5\n    return base + weight_kg * 1.2",
  "trust": 2,
  "inputs": {
    "weight_kg": {"type": "number", "description": "Package weight in kg"},
    "country": {"type": "string", "description": "ISO 3166-1 alpha-2 country code"}
  },
  "outputs": {
    "return": {"type": "number", "description": "Shipping cost in EUR"}
  },
  "sandbox": {
    "network": false,
    "filesystem": false,
    "max_memory_mb": 64,
    "max_time_seconds": 5
  },
  "description": "Calculates shipping cost based on weight and destination country. EU countries have lower rates."
}
```

**Agent behavior:**
1. Read `"description"` first — understand what the code does before running it
2. Check trust policy — does the agent allow trust level 2?
3. If yes → execute in sandboxed environment matching `"sandbox"` requirements
4. If no → use `"description"` + `"inputs"` + `"outputs"` to infer the function's behavior without executing

**Sandbox requirements:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `"network"` | bool | `false` | Whether the code needs network access |
| `"filesystem"` | bool | `false` | Whether the code needs filesystem access |
| `"max_memory_mb"` | uint | 64 | Maximum memory allocation |
| `"max_time_seconds"` | uint | 5 | Maximum execution time |

If `"network"` or `"filesystem"` is `true`, the effective trust level is 3 (not 2), regardless of the declared `"trust"` value.

---

## 7. API Endpoint Block (Trust 3)

Describes an API endpoint that an agent can call. Requires network access and authenticated interaction.

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"api_endpoint"` |
| `"trust"` | uint | REQUIRED | `3` |
| `"method"` | text | REQUIRED | HTTP method: `"GET"`, `"POST"`, `"PUT"`, `"DELETE"` |
| `"url"` | text | REQUIRED | Endpoint URL (absolute or relative to site domain) |
| `"description"` | text | REQUIRED | What this endpoint does |
| `"auth"` | text | OPTIONAL | Required auth tier: `"T0"`, `"T1"`, `"T2"` |
| `"request"` | map | OPTIONAL | Request schema (headers, body, query params) |
| `"response"` | map | OPTIONAL | Response schema |
| `"rate_limit"` | map | OPTIONAL | Endpoint-specific rate limit |

```cbor-diag
{
  "t": "api_endpoint",
  "url": "/api/v1/products/{product_id}/stock",
  "auth": "T1",
  "trust": 3,
  "method": "GET",
  "request": {
    "path_params": {
      "product_id": {"type": "string", "description": "Product identifier"}
    }
  },
  "response": {
    "type": "object",
    "properties": {
      "stock": {"type": "integer"},
      "last_updated": {"type": "string", "format": "date-time"}
    }
  },
  "rate_limit": {"requests_per_minute": 60},
  "description": "Returns current stock level for a product. Requires T1 authentication."
}
```

**Agent behavior:** The agent reads the endpoint description, checks auth requirements, constructs a request according to the schema, and processes the response. This enables **autonomous API consumption** without documentation lookup.

---

## 8. Workflow Block (Trust 3)

Describes a multi-step autonomous process. Workflows chain multiple actions with conditions and error handling.

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"workflow"` |
| `"trust"` | uint | REQUIRED | `3` |
| `"name"` | text | REQUIRED | Workflow name |
| `"description"` | text | REQUIRED | What this workflow accomplishes |
| `"steps"` | array of maps | REQUIRED | Ordered steps |
| `"requires_consent"` | bool | OPTIONAL | Whether human consent is needed before execution. Default: `true` |

```cbor-diag
{
  "t": "workflow",
  "name": "purchase_product",
  "trust": 3,
  "steps": [
    {
      "id": "check_stock",
      "action": "api_call",
      "endpoint": "/api/v1/products/{product_id}/stock",
      "method": "GET",
      "on_fail": "abort"
    },
    {
      "id": "verify_stock",
      "action": "evaluate",
      "condition": "check_stock.response.stock > 0",
      "on_fail": "abort",
      "fail_message": "Product out of stock"
    },
    {
      "id": "add_to_cart",
      "action": "api_call",
      "endpoint": "/api/v1/cart/add",
      "method": "POST",
      "body": {"product_id": "{product_id}", "quantity": "{quantity}"},
      "on_fail": "retry",
      "max_retries": 2
    },
    {
      "id": "checkout",
      "action": "api_call",
      "endpoint": "/api/v1/checkout",
      "method": "POST",
      "requires_consent": true,
      "on_fail": "abort"
    }
  ],
  "description": "Complete product purchase: check stock → add to cart → checkout. Requires human consent before payment.",
  "requires_consent": true
}
```

### 8.1 Step Actions

| Action | Description | Trust |
|--------|------------|-------|
| `"api_call"` | HTTP request to an endpoint | 3 |
| `"evaluate"` | Evaluate a condition expression (§4.1 syntax) | 0 |
| `"template"` | Render a template with context | 1 |
| `"wait"` | Pause for specified duration | 0 |

### 8.2 Error Handling

| Strategy | Behavior |
|----------|----------|
| `"abort"` | Stop workflow, report error |
| `"retry"` | Retry step (up to `"max_retries"`, default 1) |
| `"skip"` | Skip step, continue to next |
| `"fallback"` | Execute alternative step (specified in `"fallback_step"`) |

### 8.3 Consent Requirement

If `"requires_consent"` is `true` (default), the agent MUST obtain human approval before executing the workflow. This is critical for workflows that involve financial transactions, data submission, or irreversible actions.

An agent MUST NOT execute a `requires_consent: true` workflow autonomously. The human's explicit approval MUST be obtained via the agent's UI or communication channel.

---

## 9. Form Blocks (Page Key 8)

Forms appear in page key 8 (`forms` array). They describe user input structures that an agent can fill and submit.

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"form"` |
| `"trust"` | uint | REQUIRED | `3` |
| `"name"` | text | REQUIRED | Form name |
| `"action"` | text | REQUIRED | Submit URL |
| `"method"` | text | REQUIRED | `"POST"` or `"GET"` |
| `"fields"` | array of maps | REQUIRED | Form fields |
| `"description"` | text | OPTIONAL | Form purpose |
| `"requires_consent"` | bool | OPTIONAL | Default: `true` |

```cbor-diag
{
  "t": "form",
  "name": "contact",
  "trust": 3,
  "action": "/api/contact",
  "method": "POST",
  "fields": [
    {"name": "name", "type": "text", "label": "Nom complet", "required": true},
    {"name": "email", "type": "email", "label": "Email", "required": true},
    {"name": "subject", "type": "select", "label": "Sujet", "options": ["Devis", "Support", "Partenariat"], "required": true},
    {"name": "message", "type": "textarea", "label": "Message", "required": true, "max_length": 5000}
  ],
  "description": "Formulaire de contact — les messages sont traités sous 24h",
  "requires_consent": true
}
```

### 9.1 Field Types

| Type | HTML Equivalent | Value Type |
|------|----------------|-----------|
| `"text"` | `<input type="text">` | string |
| `"email"` | `<input type="email">` | string (email format) |
| `"tel"` | `<input type="tel">` | string (phone format) |
| `"number"` | `<input type="number">` | number |
| `"select"` | `<select>` | string (from `"options"` list) |
| `"textarea"` | `<textarea>` | string |
| `"checkbox"` | `<input type="checkbox">` | boolean |
| `"date"` | `<input type="date">` | string (ISO 8601) |
| `"file"` | `<input type="file">` | binary (URL after upload) |
| `"hidden"` | `<input type="hidden">` | string |

---

## 10. Commerce (Page Key 9)

Commerce data appears in page key 9. It enables an agent to understand products, pricing, and purchase mechanics.

### 10.1 Product Block

```cbor-diag
{
  "t": "product",
  "id": "lions-mane-90",
  "name": "Lion's Mane — 90 capsules",
  "price": 29.90,
  "currency": "EUR",
  "availability": "in_stock",
  "stock": 142,
  "variants": [
    {"id": "lm-90", "name": "90 capsules", "price": 29.90},
    {"id": "lm-180", "name": "180 capsules", "price": 49.90}
  ],
  "certifications": ["Bio EU", "Vegan", "Sans OGM"],
  "shipping": {
    "eu": {"price": 5.00, "delivery_days": 3},
    "international": {"price": 12.00, "delivery_days": 7}
  }
}
```

**CDDL:**

```cddl
product-block = {
  "t" => "product",
  "id" => tstr,
  "name" => tstr,
  "price" => float / uint,
  "currency" => tstr,
  ? "availability" => "in_stock" / "out_of_stock" / "pre_order" / tstr,
  ? "stock" => uint,
  ? "variants" => [+ product-variant],
  ? "certifications" => [+ tstr],
  ? "shipping" => { * tstr => shipping-option },
  ? "images" => [+ tstr],
  ? "sku" => tstr,
  ? "ean" => tstr,
  * tstr => any
}

product-variant = {
  "id" => tstr,
  "name" => tstr,
  ? "price" => float / uint,
  * tstr => any
}

shipping-option = {
  ? "price" => float / uint,
  ? "delivery_days" => uint,
  * tstr => any
}
```

### 10.2 Cart Action Block (Trust 3)

```cbor-diag
{
  "t": "cart_action",
  "trust": 3,
  "actions": {
    "add": {
      "method": "POST",
      "url": "/api/v1/cart/add",
      "body_schema": {
        "product_id": "string",
        "variant_id": "string",
        "quantity": "integer"
      }
    },
    "view": {
      "method": "GET",
      "url": "/api/v1/cart"
    },
    "checkout": {
      "method": "POST",
      "url": "/api/v1/checkout",
      "requires_consent": true
    }
  }
}
```

---

## 11. Capabilities Declaration (Manifest Key 7)

The manifest key 7 declares what the site offers beyond static content. An agent reads capabilities before fetching pages.

```cbor-diag
7: {
  "conformance": "standard",
  "multimedia": {
    "available": true,
    "types": ["image", "video", "document"]
  },
  "generative": {
    "available": true,
    "types": ["schema", "constraint", "template", "api_endpoint"],
    "workflows": false
  },
  "commerce": {
    "available": true,
    "currency": ["EUR", "USD"],
    "payment_methods": ["card", "paypal"],
    "checkout_url": "/api/v1/checkout"
  },
  "forms": {
    "available": true,
    "types": ["contact", "newsletter"]
  },
  "search": {
    "available": true,
    "endpoint": "/api/v1/search",
    "auth_required": "T1"
  },
  "languages": ["fr", "en", "es"]
}
```

**CDDL:**

```cddl
capabilities = {
  ? "conformance" => "minimal" / "standard" / "full",
  ? "multimedia" => {
    "available" => bool,
    ? "types" => [+ tstr],
    * tstr => any
  },
  ? "generative" => {
    "available" => bool,
    ? "types" => [+ tstr],
    ? "workflows" => bool,
    * tstr => any
  },
  ? "commerce" => {
    "available" => bool,
    ? "currency" => [+ tstr],
    ? "payment_methods" => [+ tstr],
    ? "checkout_url" => tstr,
    * tstr => any
  },
  ? "forms" => {
    "available" => bool,
    ? "types" => [+ tstr],
    * tstr => any
  },
  ? "search" => {
    "available" => bool,
    ? "endpoint" => tstr,
    ? "auth_required" => "T0" / "T1" / "T2",
    * tstr => any
  },
  * tstr => any
}
```

---

## 12. Agent Processing Guidelines

### 12.1 Trust-First Processing

Before processing any generative block, an agent MUST:

1. Read the `"trust"` value (implicit or explicit)
2. Check against its trust policy (CBOR-WEB-SECURITY.md §8.3)
3. If trust level not supported → skip the block, log a warning
4. If trust level supported → proceed with appropriate safeguards

### 12.2 Description-First Approach

Every generative block has a `"description"` field. An agent that cannot or will not execute a block SHOULD still read the description to understand what the block offers. The description enables the agent to inform the user about available capabilities without executing them.

### 12.3 Consent Before Action

Blocks with `"requires_consent": true` MUST NOT be executed without human approval. The default for trust level 3 blocks is `requires_consent: true`. Publishers MAY set it to `false` for low-risk trust-3 actions (e.g., a read-only API call), but agents SHOULD treat all trust-3 blocks as requiring consent unless the agent has an explicit autonomous policy.

---

*CBOR-Web Generative & Commerce Specification v2.1 — Document 4 of 6*

*ExploDev 2026*
