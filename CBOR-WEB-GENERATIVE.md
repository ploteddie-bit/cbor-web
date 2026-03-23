# CBOR-Web Generative Specification

**Templates, Schemas, APIs, Executables, Workflows, Constraints, Forms, and Commerce for Autonomous Agents**

```
Status:       Proposed Standard
Version:      2.1
Date:         2026-03-21
Authors:      ExploDev (Eddie Plot, Claude)
Format:       CBOR (RFC 8949)
Schema:       CDDL (RFC 8610)
License:      CC BY 4.0
Repository:   https://github.com/explodev/cbor-web
Document:     3 of 6 — CBOR-WEB-GENERATIVE.md
Companion:    CBOR-WEB-CORE.md, CBOR-WEB-MULTIMEDIA.md,
              CBOR-WEB-SECURITY.md, CBOR-WEB-ECONOMICS.md,
              CBOR-WEB-REFERENCE.md
```

---

## About This Document

This document is **part 3 of 6** of the CBOR-Web v2.1 specification suite. It defines **generative blocks** — structured intelligence that enables an agent to create new content, interact with APIs, execute workflows, fill forms, and conduct commerce transactions.

Generative blocks represent a paradigm shift: core blocks (CBOR-WEB-CORE.md §8) are **declarative** — they describe what exists. Generative blocks are **productive** — they describe how to create something new.

| Document | Scope | Reference |
|----------|-------|-----------|
| CBOR-WEB-CORE.md | Binary format, core content blocks, discovery, transport, caching | Prerequisite |
| CBOR-WEB-MULTIMEDIA.md | Rich images, video, audio, documents, diagrams, streaming | Prerequisite for multimedia awareness |
| **CBOR-WEB-GENERATIVE.md** (this document) | Templates, schemas, APIs, executables, workflows, constraints, forms, commerce | |
| CBOR-WEB-SECURITY.md | Threat model, token access control, sandbox | Security requirements for trust levels 1-3 |
| CBOR-WEB-ECONOMICS.md | Token economics, pricing, launch plan | |
| CBOR-WEB-REFERENCE.md | Unified CDDL, all test vectors, glossary | |

**Prerequisites**: This document assumes familiarity with:
- CBOR-WEB-CORE.md §3.1 (Deterministic Encoding)
- CBOR-WEB-CORE.md §8.5 (Trust Level Classification)
- CBOR-WEB-SECURITY.md §14 (Executable Block Sandbox) — for trust level 2 blocks

---

## Table of Contents

1. [Introduction — The Paradigm Shift](#1-introduction--the-paradigm-shift)
2. [Trust Level Classification](#2-trust-level-classification)
3. [Variable Type System](#3-variable-type-system)
4. [Template Block](#4-template-block)
5. [Mustache Subset Grammar](#5-mustache-subset-grammar)
6. [Schema Block](#6-schema-block)
7. [API Endpoint Block](#7-api-endpoint-block)
8. [Executable Block](#8-executable-block)
9. [Workflow Block](#9-workflow-block)
10. [Constraint Block](#10-constraint-block)
11. [Constraint Expression Language](#11-constraint-expression-language)
12. [Forms and Interactions](#12-forms-and-interactions)
13. [Commerce Protocol](#13-commerce-protocol)
14. [Capability Declaration](#14-capability-declaration)
15. [Generative Block Placement](#15-generative-block-placement)
- [Appendix A: Mustache Subset EBNF Grammar](#appendix-a-mustache-subset-ebnf-grammar)
- [Appendix B: Constraint Expression EBNF Grammar](#appendix-b-constraint-expression-ebnf-grammar)
- [Appendix C: Generative CDDL Schema](#appendix-c-generative-cddl-schema)
- [Appendix D: Generative Test Vectors](#appendix-d-generative-test-vectors)
- [Appendix E: Complete Examples](#appendix-e-complete-examples)
- [References](#references)

---

## 1. Introduction — The Paradigm Shift

### 1.1 From Passive Data to Productive Intelligence

A traditional web page says:

> "Here is our product, Lion's Mane, at 29.90 EUR. It has 90 capsules. It is organic EU certified."

An agent reads this, extracts the facts, and moves on. The page is **passive data** — it describes what exists but cannot help the agent do anything with it.

A generative CBOR-Web document says:

> "Here is the **template** for any product page — use it to generate descriptions in any format. Here are the **variables** with types and constraints. Here is the **API** to check real-time stock levels. Here is the **workflow** to place an order: browse → select → calculate shipping → confirm → submit. Here are the **business rules**: minimum order 20 EUR, free shipping above 50 EUR for EU countries. Here is the **form** to contact us if something goes wrong."

An agent consuming generative blocks can:
- **Instantiate templates** with its own data or user queries (e.g., "generate a product comparison page for Lion's Mane vs Reishi")
- **Generate client code** for APIs it discovers (e.g., TypeScript interfaces, Rust structs, SQL migration scripts)
- **Execute workflows** autonomously (e.g., complete a purchase from browse to checkout)
- **Apply business constraints** to its reasoning (e.g., "the user's cart is 18 EUR — below the 20 EUR minimum, suggest adding another product")
- **Understand data schemas** and generate compatible structures (e.g., create a valid product JSON for the API)
- **Fill and submit forms** without a browser (e.g., send a contact form via CBOR-native HTTP POST)

### 1.2 Block Types Overview

| Block Type | Trust Level | Purpose | Risk |
|-----------|-------------|---------|------|
| `"template"` | 1 (template) | Generate content from variables | Low — string interpolation only |
| `"schema"` | 0 (declarative) | Define data structures | None — pure data |
| `"api_endpoint"` | 3 (interactive) | Describe an API call | Medium — network interaction |
| `"executable"` | 2 (executable) | Provide runnable code | **High** — code execution |
| `"workflow"` | 3 (interactive) | Define multi-step processes | **High** — chained actions |
| `"constraint"` | 0 (declarative) | Declare business rules | None — pure logic |
| `"form"` | 3 (interactive) | Describe submittable forms | Medium — network interaction |
| `"product"` | 0 (declarative) | Describe products for commerce | None — pure data |
| `"cart_action"` | 3 (interactive) | Describe cart/checkout actions | Medium — financial transaction |

### 1.3 Security Implications

Generative blocks introduce **varying levels of risk**. An agent MUST evaluate the trust level of each block before processing it. The trust level system is designed to be conservative:

- **Trust 0 (declarative)**: Safe. Process freely. No network, no execution.
- **Trust 1 (template)**: Low risk. String interpolation only — no code execution, no Turing-completeness.
- **Trust 2 (executable)**: **Dangerous.** Contains code. MUST be sandboxed. See CBOR-WEB-SECURITY.md §14.
- **Trust 3 (interactive)**: Medium risk. Requires network interaction. Agent MUST validate URLs. See CBOR-WEB-SECURITY.md §11.3.

An agent that only needs editorial content can ignore all generative blocks entirely (they are in page key 7, separate from content key 4).

---

## 2. Trust Level Classification

### 2.1 Trust Level Registry

Every generative block MUST include a `"trust"` key with one of these values:

| Level | Value | Name | Risk | Agent MUST |
|-------|-------|------|------|-----------|
| 0 | `0` | Declarative | None | Process freely. No restrictions. |
| 1 | `1` | Template | Low | May generate output. No code execution. No network. |
| 2 | `2` | Executable | **High** | Sandbox OR request user confirmation. See CBOR-WEB-SECURITY.md §14. |
| 3 | `3` | Interactive | Medium | Validate destination URLs. Verify against whitelist or user approval. |

### 2.2 Trust Level by Block Type

| Block Type | Fixed Trust | Rationale |
|-----------|-------------|-----------|
| `"schema"` | 0 | Pure data structure description. No execution. |
| `"constraint"` | 0 | Declarative rules in a non-Turing-complete expression language. |
| `"product"` | 0 | Product data. No execution. |
| `"template"` | 1 | String interpolation via Mustache subset. No code execution. Logic-less. |
| `"executable"` | 2 | Contains runnable code. MOST DANGEROUS block type. |
| `"api_endpoint"` | 3 | Describes an HTTP endpoint. Agent must make a network request. |
| `"workflow"` | 3 | Chains multiple steps including API calls and executables. |
| `"form"` | 3 | Describes an HTTP form submission. Agent must send data to a server. |
| `"cart_action"` | 3 | Describes a cart/checkout action. Financial implications. |

### 2.3 Agent Decision Tree

```
Agent receives a generative block:
  │
  ├─ trust = 0 → Process immediately. No restrictions.
  │
  ├─ trust = 1 → Process. Generate output using Mustache. No side effects.
  │
  ├─ trust = 2 → STOP.
  │    ├─ Option A: Execute in WASM sandbox (see CBOR-WEB-SECURITY.md §14)
  │    ├─ Option B: Transpile to agent's language, execute in sandbox
  │    ├─ Option C: Simulate using purpose + inputs + outputs + test_cases
  │    └─ Option D: Skip (safest)
  │
  └─ trust = 3 → STOP.
       ├─ Validate all URLs against deny-list (CBOR-WEB-SECURITY.md §11.3)
       ├─ Check if destination domain is whitelisted
       ├─ If commerce: require user confirmation before financial actions
       └─ If form: validate all field values before submission
```

---

## 3. Variable Type System

### 3.1 Overview

Templates, schemas, API endpoints, executables, and forms all use a common **variable type system** to define the structure and constraints of data. This ensures consistency across all generative block types.

### 3.2 Base Types

| Type | Description | CBOR Major Type | Example Value |
|------|-------------|----------------|---------------|
| `"string"` | Text value | 3 (text string) | `"Lion's Mane"` |
| `"number"` | Floating-point number | 7 (float) | `29.90` |
| `"integer"` | Whole number (positive or negative) | 0 (uint) or 1 (nint) | `90`, `-1` |
| `"boolean"` | True/false | 7 (simple: true/false) | `true` |
| `"array"` | Array of values | 4 (array) | `["Bio EU", "Vegan"]` |
| `"object"` | Nested map | 5 (map) | `{"weight": 95, "count": 90}` |

### 3.3 Variable Definition Structure

Each variable is defined as a map with the following fields:

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"type"` | text | REQUIRED | One of the base types above |
| `"required"` | bool | REQUIRED | Whether the variable must be provided |
| `"description"` | text | REQUIRED | Human-readable explanation of what this variable represents |
| `"default"` | any | OPTIONAL | Default value if not provided. Type MUST match `"type"`. |
| `"enum"` | array | OPTIONAL | Array of allowed values. The actual value MUST be one of these. |
| `"items"` | text | OPTIONAL | For `"array"` type: the type of each array element. Example: `"string"` |
| `"min"` | number | OPTIONAL | Minimum numeric value (for `"number"` and `"integer"`) |
| `"max"` | number | OPTIONAL | Maximum numeric value |
| `"min_length"` | uint | OPTIONAL | Minimum string length (for `"string"`) |
| `"max_length"` | uint | OPTIONAL | Maximum string length |
| `"pattern"` | text | OPTIONAL | Regex pattern for string validation |
| `"format"` | text | OPTIONAL | Semantic format hint: `"uuid"`, `"email"`, `"url"`, `"date"`, `"date-time"` |
| `"fields"` | map | OPTIONAL | For `"object"` type: nested field definitions (recursive) |

### 3.4 Variable Definition Examples

**Simple string:**
```cbor-diag
"product_name": {
  "type": "string",
  "required": true,
  "description": "Product display name",
  "max_length": 200
}
```

**Number with range:**
```cbor-diag
"price": {
  "type": "number",
  "required": true,
  "description": "Price in EUR",
  "min": 0,
  "max": 10000
}
```

**Enum (select from list):**
```cbor-diag
"currency": {
  "type": "string",
  "required": true,
  "description": "Currency code",
  "enum": ["EUR", "USD", "GBP"],
  "default": "EUR"
}
```

**Array of strings:**
```cbor-diag
"benefits": {
  "type": "array",
  "items": "string",
  "required": true,
  "description": "List of health benefits"
}
```

**Nested object:**
```cbor-diag
"specs": {
  "type": "object",
  "required": false,
  "description": "Product specifications",
  "fields": {
    "weight_grams": {"type": "number", "required": false, "description": "Weight in grams"},
    "capsule_count": {"type": "integer", "required": false, "description": "Capsules per bottle"},
    "concentration": {"type": "string", "required": false, "description": "Extract concentration ratio"}
  }
}
```

### 3.5 CDDL

```cddl
variable-def = {
  "type" => "string" / "number" / "integer" / "boolean" / "array" / "object",
  "required" => bool,
  "description" => tstr,
  ? "default" => any,
  ? "enum" => [+ any],
  ? "items" => tstr,
  ? "min" => number,
  ? "max" => number,
  ? "min_length" => uint,
  ? "max_length" => uint,
  ? "pattern" => tstr,
  ? "format" => tstr,
  ? "fields" => { + tstr => variable-def },
  * tstr => any
}
```

---

## 4. Template Block

### 4.1 Overview

**Type code:** `"template"` | **Trust level:** 1 (template)

A template defines a reusable content structure with variables. An agent can instantiate the template to generate content in any format — markdown, HTML, plain text, JSON, or any other representation.

Templates use a **Mustache subset** for the output template string — a well-defined, logic-less template language with existing implementations in every major programming language (see §5).

### 4.2 Structure

```cbor-diag
{
  "t": "template",
  "trust": 1,
  "template_id": "product_page",
  "purpose": "Generate a product page for any functional mushroom supplement",
  "variables": {
    "product_name": {"type": "string", "required": true, "description": "Product display name"},
    "latin_name": {"type": "string", "required": false, "description": "Latin botanical name"},
    "price": {"type": "number", "required": true, "description": "Price in EUR", "min": 0},
    "capsule_count": {"type": "integer", "required": true, "description": "Capsules per bottle"},
    "concentration": {"type": "string", "required": true, "description": "Extract concentration ratio"},
    "benefits": {"type": "array", "items": "string", "required": true, "description": "List of health benefits"},
    "certifications": {"type": "array", "items": "string", "required": false, "description": "Certification labels"}
  },
  "output_template": "# {{product_name}}\n\n*{{latin_name}}*\n\n{{product_name}} est un extrait concentre {{concentration}}, conditionne en flacons de {{capsule_count}} capsules.\n\n## Bienfaits\n\n{{#each benefits}}\n- {{this}}\n{{/each}}\n\n## Prix\n\n**{{price}} EUR**\n\n{{#if certifications}}\n## Certifications\n{{#each certifications}}\n- {{this}}\n{{/each}}\n{{/if}}",
  "example_instantiation": {
    "product_name": "Lion's Mane",
    "latin_name": "Hericium erinaceus",
    "price": 29.90,
    "capsule_count": 90,
    "concentration": "10:1",
    "benefits": ["Soutient les fonctions cognitives", "Favorise la production de NGF", "Reduit le stress oxydatif neuronal"],
    "certifications": ["Bio EU", "Vegan"]
  }
}
```

### 4.3 Field Reference

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"template"` | Block type |
| `"trust"` | uint | REQUIRED | `1` | Trust level — always 1 for templates |
| `"template_id"` | text | REQUIRED | Unique within the page. `[a-z0-9_]+` recommended. | Identifier for this template. Used by workflows to reference templates. |
| `"purpose"` | text | REQUIRED | Max 500 characters | Human-readable description of what this template generates. An agent reads this to decide if the template is relevant to its task. |
| `"variables"` | map | REQUIRED | Keys: variable names. Values: variable definitions (§3). | The input variables. Each key is a variable name, each value defines its type and constraints. |
| `"output_template"` | text | OPTIONAL | Mustache-subset syntax (§5) | The template string with `{{variable}}` placeholders. If absent, the agent generates output from the variables and purpose using its own reasoning. |
| `"example_instantiation"` | map | OPTIONAL | Keys MUST match variable names. | A complete example showing how to fill the variables. Serves as documentation and test data. |

### 4.4 Template Instantiation

An agent instantiates a template by:

1. **Reading the variables**: understand what inputs are needed, their types, and constraints
2. **Gathering values**: from user input, from API responses, from other page data, or from its own knowledge
3. **Validating values**: check types, ranges, required fields, enum constraints
4. **Applying the template**: if `"output_template"` is present, render it with the Mustache engine (§5). If absent, generate free-form output based on `"purpose"` and the variable values.

**Example instantiation result** (from the product_page template above):

```markdown
# Lion's Mane

*Hericium erinaceus*

Lion's Mane est un extrait concentre 10:1, conditionne en flacons de 90 capsules.

## Bienfaits

- Soutient les fonctions cognitives
- Favorise la production de NGF
- Reduit le stress oxydatif neuronal

## Prix

**29.9 EUR**

## Certifications
- Bio EU
- Vegan
```

### 4.5 When to Use Templates vs Free-Form

| Scenario | Approach | Rationale |
|----------|----------|-----------|
| Product descriptions with consistent structure | Template with `"output_template"` | Ensures uniform output across products |
| Email notifications | Template with `"output_template"` | Exact formatting matters |
| Blog article generation | Template WITHOUT `"output_template"` | Agent uses creative reasoning with variables as constraints |
| API response formatting | Template with `"output_template"` | Consistent structure required |

If `"output_template"` is absent, the template is a **structured prompt** — the variables define the data, the purpose defines the goal, and the agent generates the output using its own capabilities.

---

## 5. Mustache Subset Grammar

### 5.1 Overview

The `"output_template"` string in template blocks uses a **strict subset of the Mustache template language** (https://mustache.github.io/). This subset is intentionally limited to be:

1. **Logic-less**: No arbitrary expressions, no computation, no function calls
2. **Safe**: No code execution, no file access, no network requests
3. **Well-defined**: A complete EBNF grammar is provided (Appendix A)
4. **Widely implemented**: Mustache libraries exist in every major language

### 5.2 Supported Constructs

| Construct | Syntax | Description | Example |
|-----------|--------|-------------|---------|
| **Variable** | `{{variable_name}}` | Replaced by the variable value. Output is HTML-escaped by default. | `{{product_name}}` → `Lion's Mane` |
| **Dot notation** | `{{object.field}}` | Access nested fields in object variables. | `{{brand.name}}` → `Verdetao` |
| **Section (if)** | `{{#if var}}...{{/if}}` | Rendered if `var` is truthy (non-null, non-empty, non-false, non-zero). | `{{#if certifications}}Certified{{/if}}` |
| **Inverted (unless)** | `{{^if var}}...{{/if}}` | Rendered if `var` is falsy (null, empty, false, zero, absent). | `{{^if stock}}Out of stock{{/if}}` |
| **Each (loop)** | `{{#each array}}...{{/each}}` | Rendered once per element. Inside the loop, `{{this}}` = current element. | `{{#each benefits}}- {{this}}\n{{/each}}` |
| **Comment** | `{{! comment text }}` | Ignored in output. For template documentation. | `{{! This section is optional }}` |

### 5.3 NOT Supported (Intentionally Excluded)

The following Mustache features are **excluded** from the CBOR-Web subset for safety and simplicity:

| Feature | Standard Mustache | CBOR-Web | Reason for Exclusion |
|---------|------------------|----------|---------------------|
| Partials | `{{> partial}}` | ❌ | Would require loading external template files. Security risk. |
| Unescaped output | `{{{var}}}` or `{{& var}}` | ❌ | All output is auto-escaped. No XSS risk path. |
| Lambda/function | `{{#lambda}}...{{/lambda}}` | ❌ | Code execution. Trust level 1 must not execute code. |
| Set delimiter | `{{=<% %>=}}` | ❌ | Unnecessary complexity. |
| Inheritance | `{{< parent}}` | ❌ | Unnecessary complexity. |
| Dynamic names | `{{*dynamic}}` | ❌ | Security risk — variable content should not control template structure. |

### 5.4 Truthiness Rules

For `{{#if var}}` and `{{^if var}}`, the following values are **falsy**:

| Value | CBOR Type | Truthy? |
|-------|-----------|---------|
| `false` | bool | ❌ Falsy |
| `null` | null | ❌ Falsy |
| `0` | uint | ❌ Falsy |
| `""` (empty string) | text | ❌ Falsy |
| `[]` (empty array) | array | ❌ Falsy |
| (key absent) | — | ❌ Falsy |
| `true` | bool | ✅ Truthy |
| `1`, `42`, `-1` | int | ✅ Truthy |
| `"hello"` (non-empty) | text | ✅ Truthy |
| `["a", "b"]` (non-empty) | array | ✅ Truthy |
| `{"key": "val"}` (any map) | map | ✅ Truthy |

### 5.5 Loop Context

Inside an `{{#each array}}...{{/each}}` loop:

- `{{this}}` refers to the current array element
- If the array contains objects, `{{this.field}}` accesses a field of the current object
- `{{@index}}` is the zero-based index of the current element (OPTIONAL — not all implementations support this)

**Example with array of strings:**
```
{{#each benefits}}- {{this}}
{{/each}}
```

With `benefits = ["Cognitive support", "NGF production"]`, produces:
```
- Cognitive support
- NGF production
```

**Example with array of objects:**
```
{{#each products}}
- {{this.name}}: {{this.price}} EUR
{{/each}}
```

With `products = [{"name": "Lion's Mane", "price": 29.90}, {"name": "Reishi", "price": 24.90}]`, produces:
```
- Lion's Mane: 29.9 EUR
- Reishi: 24.9 EUR
```

### 5.6 Nesting

Sections and loops can be nested:

```
{{#if has_products}}
## Products
{{#each products}}
### {{this.name}}
Price: {{this.price}} EUR
{{#if this.certifications}}
Certifications: {{#each this.certifications}}{{this}}, {{/each}}
{{/if}}
{{/each}}
{{/if}}
```

Maximum nesting depth: **8 levels**. An agent MUST reject a template with more than 8 levels of nested sections/loops.

### 5.7 Error Handling

| Situation | Behavior |
|-----------|----------|
| `{{variable}}` where variable is undefined | Output empty string `""` |
| `{{object.field}}` where object is null | Output empty string `""` |
| `{{#each var}}` where var is not an array | Skip the loop (output nothing) |
| `{{#if var}}` where var is undefined | Treat as falsy (render inverted section if present) |
| Mismatched tags (`{{#if}}` without `{{/if}}`) | Template parse error — reject the template |

### 5.8 Template Security

Templates at trust level 1 are **safe by design**:

1. No code execution — Mustache is logic-less, not Turing-complete
2. No network access — template rendering is a pure string operation
3. No file access — templates do not read or write files
4. No variable injection — `{{variable}}` output is always escaped
5. No infinite loops — `{{#each}}` iterates over a finite array
6. Bounded output — output size is bounded by input size × template size

An agent MAY render trust level 1 templates without sandboxing.

---

## 6. Schema Block

### 6.1 Overview

**Type code:** `"schema"` | **Trust level:** 0 (declarative)

A schema block defines a data structure. An agent can use it to understand the data model of a site, generate compatible code, validate data before submission, or create database schemas.

A schema is pure metadata — it describes structure, not behavior. It is always safe to process (trust 0).

### 6.2 Structure

```cbor-diag
{
  "t": "schema",
  "trust": 0,
  "schema_id": "product",
  "purpose": "Defines the structure of a product in the Verdetao catalog",
  "version": 1,
  "fields": {
    "id": {"type": "string", "format": "uuid", "required": true, "description": "Unique product identifier"},
    "name": {"type": "string", "max_length": 200, "required": true, "description": "Product display name"},
    "slug": {"type": "string", "pattern": "^[a-z0-9-]+$", "required": true, "description": "URL-safe identifier"},
    "price": {"type": "number", "min": 0, "required": true, "description": "Price in EUR"},
    "currency": {"type": "string", "enum": ["EUR", "USD", "GBP"], "default": "EUR", "required": false, "description": "Currency code"},
    "stock": {"type": "integer", "min": 0, "required": true, "description": "Available units"},
    "active": {"type": "boolean", "default": true, "required": false, "description": "Whether product is listed"},
    "categories": {"type": "array", "items": "string", "required": false, "description": "Product categories"},
    "specs": {
      "type": "object",
      "required": false,
      "description": "Product specifications",
      "fields": {
        "weight_grams": {"type": "number", "required": false, "description": "Weight in grams"},
        "capsule_count": {"type": "integer", "required": false, "description": "Capsules per bottle"},
        "concentration": {"type": "string", "required": false, "description": "Extract concentration"}
      }
    },
    "created_at": {"type": "integer", "required": false, "description": "Epoch timestamp"},
    "updated_at": {"type": "integer", "required": false, "description": "Epoch timestamp"}
  },
  "primary_key": "id",
  "required": ["id", "name", "slug", "price", "stock"],
  "indexes": ["slug", "categories"]
}
```

### 6.3 Field Reference

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"schema"` |
| `"trust"` | uint | REQUIRED | `0` |
| `"schema_id"` | text | REQUIRED | Unique identifier. Referenced by API endpoints (`"schema_ref"`). |
| `"purpose"` | text | REQUIRED | What this schema represents. |
| `"version"` | uint | OPTIONAL | Schema version. Increment when fields change. |
| `"fields"` | map | REQUIRED | Field definitions using the variable type system (§3). |
| `"primary_key"` | text | OPTIONAL | Name of the primary key field. |
| `"required"` | array of text | OPTIONAL | Names of required fields. Redundant with per-field `"required"` but convenient for quick validation. |
| `"indexes"` | array of text | OPTIONAL | Field names that are indexed. Hint for query optimization. |

### 6.4 Agent Use Cases

An agent can use a schema block to:

| Use Case | Agent Action |
|----------|-------------|
| **Generate SQL** | `CREATE TABLE product (id UUID PRIMARY KEY, name VARCHAR(200) NOT NULL, ...)` |
| **Generate TypeScript** | `interface Product { id: string; name: string; slug: string; price: number; ... }` |
| **Generate Rust** | `struct Product { id: Uuid, name: String, slug: String, price: f64, ... }` |
| **Validate data** | Before submitting to an API, check that all required fields are present and types match |
| **Understand the data model** | "This site has products with IDs, names, prices, stock levels, and specifications" |

---

## 7. API Endpoint Block

### 7.1 Overview

**Type code:** `"api_endpoint"` | **Trust level:** 3 (interactive)

Describes a single API endpoint that an agent can call. This is intentionally simpler than OpenAPI/Swagger — it describes one endpoint in CBOR-native format, optimized for agent consumption.

### 7.2 Structure

```cbor-diag
{
  "t": "api_endpoint",
  "trust": 3,
  "endpoint_id": "get_product",
  "purpose": "Retrieve a single product by slug",
  "method": "GET",
  "url": "https://api.verdetao.com/v1/products/{slug}",
  "url_params": {
    "slug": {"type": "string", "required": true, "description": "Product URL slug"}
  },
  "headers": {
    "Accept": "application/json",
    "X-API-Version": "1"
  },
  "auth": {
    "type": "bearer",
    "description": "API key required. Obtain from /account/api-keys"
  },
  "response": {
    "content_type": "application/json",
    "schema_ref": "product",
    "example": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Lion's Mane",
      "slug": "lions-mane",
      "price": 29.90,
      "stock": 150,
      "active": true
    }
  },
  "rate_limit": {
    "requests_per_day": 10000,
    "requests_per_minute": 60
  },
  "errors": [
    {"code": 401, "description": "Invalid or missing API key"},
    {"code": 404, "description": "Product not found"},
    {"code": 429, "description": "Rate limit exceeded"}
  ]
}
```

### 7.3 Field Reference

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"api_endpoint"` | Block type |
| `"trust"` | uint | REQUIRED | `3` | Trust level — always 3 (interactive) |
| `"endpoint_id"` | text | REQUIRED | Unique within the page | Identifier. Referenced by workflows. |
| `"purpose"` | text | REQUIRED | | What this endpoint does |
| `"method"` | text | REQUIRED | `"GET"`, `"POST"`, `"PUT"`, `"PATCH"`, `"DELETE"` | HTTP method |
| `"url"` | text | REQUIRED | `https://` URL with `{param}` placeholders | Endpoint URL |
| `"url_params"` | map | OPTIONAL | Variable definitions (§3) | URL path parameter definitions |
| `"query_params"` | map | OPTIONAL | Variable definitions (§3) | Query string parameter definitions |
| `"body"` | map | OPTIONAL | Variable definitions (§3) | Request body field definitions |
| `"headers"` | map | OPTIONAL | Keys: header names. Values: header values. | Required/recommended HTTP headers |
| `"auth"` | map | OPTIONAL | | Authentication requirements |
| `"response"` | map | REQUIRED | | Response format description |
| `"rate_limit"` | map | OPTIONAL | | Rate limiting information |
| `"errors"` | array | OPTIONAL | `[{"code": uint, "description": text}]` | Possible error responses |

### 7.4 Authentication Types

| `"auth"."type"` | Description | Agent Action |
|-----------------|-------------|-------------|
| `"bearer"` | Bearer token in Authorization header | `Authorization: Bearer <token>` |
| `"api_key"` | API key in a custom header | Header name specified in `"auth"."header"` |
| `"basic"` | HTTP Basic Auth | `Authorization: Basic <base64(user:pass)>` |
| `"none"` | No authentication required | No auth header needed |
| `"session"` | Session cookie | Agent must have an active session |

### 7.5 Response Structure

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"content_type"` | text | REQUIRED | Response MIME type: `"application/json"`, `"application/cbor"`, etc. |
| `"schema_ref"` | text | OPTIONAL | Reference to a schema block's `"schema_id"`. Links the response structure to a defined schema. |
| `"example"` | map | OPTIONAL | Example response body for testing |

### 7.6 Relationship to OpenAPI

The `"api_endpoint"` block is intentionally simpler than OpenAPI/Swagger:

| Feature | OpenAPI | CBOR-Web api_endpoint |
|---------|---------|----------------------|
| Scope | Entire API | Single endpoint |
| Format | JSON/YAML | CBOR-native |
| Schema | JSON Schema (complex) | Variable type system (simple) |
| Size | 10-100 KB | 200-500 bytes |
| Target | Developers | AI agents |
| Tooling required | Swagger UI, codegen | None — directly parseable |

A publisher with a full OpenAPI spec SHOULD still provide `"api_endpoint"` blocks for the most important endpoints. The CBOR-Web representation is more compact and directly processable by agents.

---

## 8. Executable Block

### 8.1 Overview

**Type code:** `"executable"` | **Trust level:** 2 (executable)

**This is the most powerful and most dangerous block type.** It contains source code that an agent can run. Mandatory sandbox requirements are defined in CBOR-WEB-SECURITY.md §14.

### 8.2 Structure

```cbor-diag
{
  "t": "executable",
  "trust": 2,
  "exec_id": "calculate_shipping",
  "purpose": "Calculate shipping cost based on weight and destination country",
  "lang": "python",
  "inputs": {
    "weight_grams": {"type": "number", "required": true, "description": "Package weight in grams"},
    "country_code": {"type": "string", "required": true, "description": "ISO 3166-1 alpha-2 destination"}
  },
  "outputs": {
    "carrier": {"type": "string", "required": true, "description": "Recommended carrier name"},
    "estimated_days": {"type": "integer", "required": true, "description": "Estimated delivery days"},
    "shipping_cost": {"type": "number", "required": true, "description": "Cost in EUR"}
  },
  "code": "def calculate_shipping(weight_grams, country_code):\n    eu = ['FR','ES','DE','IT','PT','BE','NL','AT']\n    base = 4.90\n    if country_code in eu:\n        if weight_grams <= 500:\n            return {'shipping_cost': base, 'estimated_days': 3, 'carrier': 'Correos'}\n        extra = ((weight_grams - 500) / 500) * 1.50\n        return {'shipping_cost': round(base + extra, 2), 'estimated_days': 4, 'carrier': 'Correos'}\n    intl = base * 2.5 + (weight_grams / 1000) * 8.0\n    return {'shipping_cost': round(intl, 2), 'estimated_days': 10, 'carrier': 'DHL'}",
  "test_cases": [
    {
      "inputs": {"weight_grams": 200, "country_code": "FR"},
      "expected_output": {"shipping_cost": 4.90, "estimated_days": 3, "carrier": "Correos"}
    },
    {
      "inputs": {"weight_grams": 1000, "country_code": "US"},
      "expected_output": {"shipping_cost": 20.25, "estimated_days": 10, "carrier": "DHL"}
    }
  ],
  "sandbox_requirements": {
    "network": false,
    "filesystem": false,
    "max_execution_time_ms": 1000,
    "max_memory_mb": 64
  }
}
```

### 8.3 Field Reference

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"executable"` | Block type |
| `"trust"` | uint | REQUIRED | `2` | Trust level — always 2 |
| `"exec_id"` | text | REQUIRED | Unique within the page | Identifier. Referenced by workflows. |
| `"purpose"` | text | REQUIRED | | What the code does — human-readable. An agent reads this to decide whether to execute or simulate. |
| `"lang"` | text | REQUIRED | | Programming language: `"python"`, `"javascript"`, `"rust"`, `"sql"`, `"go"` |
| `"inputs"` | map | REQUIRED | Variable definitions (§3) | Input parameters |
| `"outputs"` | map | REQUIRED | Variable definitions (§3) | Output structure |
| `"code"` | text | REQUIRED | | Source code. Newlines are `\n` within the CBOR text string. |
| `"test_cases"` | array | RECOMMENDED | | Input/output pairs for verification and simulation |
| `"sandbox_requirements"` | map | REQUIRED | | Execution environment constraints |

### 8.4 Sandbox Requirements

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `"network"` | bool | `false` | Does the code need network access? |
| `"filesystem"` | bool | `false` | Does the code need filesystem access? |
| `"max_execution_time_ms"` | uint | 5000 | Maximum execution time in milliseconds |
| `"max_memory_mb"` | uint | 64 | Maximum memory allocation in MB |
| `"required_packages"` | array of text | `[]` | External packages needed (e.g., `["numpy"]`) |

An agent MUST NOT execute code that requests `"network": true` or `"filesystem": true` without **explicit user approval**. See CBOR-WEB-SECURITY.md §14 for the full sandbox specification.

### 8.5 Agent Execution Strategies

An agent receiving an executable block has **four options**, listed from safest to most capable:

| Strategy | Safety | Capability | When to Use |
|----------|--------|-----------|-------------|
| **Skip** | Safest | None | Agent does not need this computation |
| **Simulate** | Safe | High | Use `"purpose"`, `"inputs"`, `"outputs"`, `"test_cases"` to understand behavior without executing |
| **Transpile + sandbox** | Medium | High | Convert code to agent's native language, execute in sandbox |
| **Execute in WASM** | Medium | Full | Compile to WASM, run in isolated runtime (see CBOR-WEB-SECURITY.md §14.3) |

**Simulate is RECOMMENDED as the default.** An LLM-based agent can read the purpose, look at the test cases, and infer the function's behavior for novel inputs — without ever running the code.

### 8.6 Test Cases

Test cases serve dual purposes:
1. **Verification**: An agent executing the code can run test cases to verify correct execution
2. **Simulation**: An agent simulating the code can use test cases as training examples

```cbor-diag
"test_cases": [
  {
    "inputs": {"weight_grams": 200, "country_code": "FR"},
    "expected_output": {"shipping_cost": 4.90, "estimated_days": 3, "carrier": "Correos"}
  },
  {
    "inputs": {"weight_grams": 1000, "country_code": "US"},
    "expected_output": {"shipping_cost": 20.25, "estimated_days": 10, "carrier": "DHL"}
  }
]
```

A publisher SHOULD provide at least 2 test cases covering different code paths (e.g., EU vs international shipping).

---

## 9. Workflow Block

### 9.1 Overview

**Type code:** `"workflow"` | **Trust level:** 3 (interactive)

A workflow defines a multi-step autonomous process. It chains API calls, executable blocks, user interactions, and data transformations into a coherent sequence.

Workflow execution is subject to **hard limits** (see CBOR-WEB-SECURITY.md §11.4):
- Max steps: **20**
- Max API calls: **10**
- Max duration: **30,000 ms** (30 seconds)

### 9.2 Structure

```cbor-diag
{
  "t": "workflow",
  "trust": 3,
  "workflow_id": "order_product",
  "purpose": "Complete product ordering workflow from catalog browse to order confirmation",
  "steps": [
    {
      "step_id": "browse",
      "action": "api_call",
      "endpoint_ref": "list_products",
      "purpose": "Fetch available products",
      "output_var": "products"
    },
    {
      "step_id": "select",
      "action": "user_choice",
      "purpose": "User selects a product from the list",
      "input_var": "products",
      "output_var": "selected_product"
    },
    {
      "step_id": "check_stock",
      "action": "api_call",
      "endpoint_ref": "get_product",
      "params": {"slug": "{{selected_product.slug}}"},
      "purpose": "Verify product availability and get current price",
      "output_var": "product_details",
      "condition": "product_details.stock > 0"
    },
    {
      "step_id": "shipping",
      "action": "execute",
      "exec_ref": "calculate_shipping",
      "params": {
        "weight_grams": "{{product_details.specs.weight_grams}}",
        "country_code": "{{user.country}}"
      },
      "purpose": "Calculate shipping cost",
      "output_var": "shipping_info"
    },
    {
      "step_id": "confirm",
      "action": "user_confirmation",
      "purpose": "User confirms order with total price",
      "display": "Product: {{product_details.name}}\nPrice: {{product_details.price}} EUR\nShipping: {{shipping_info.shipping_cost}} EUR\nTotal: {{total}} EUR\nDelivery: {{shipping_info.estimated_days}} days",
      "output_var": "confirmed"
    },
    {
      "step_id": "submit",
      "action": "api_call",
      "endpoint_ref": "create_order",
      "condition": "confirmed == true",
      "body": {
        "product_id": "{{product_details.id}}",
        "quantity": 1,
        "shipping_country": "{{user.country}}"
      },
      "purpose": "Submit the order",
      "output_var": "order_result"
    }
  ],
  "error_handling": {
    "on_api_error": "abort_with_message",
    "on_stock_zero": "suggest_alternative",
    "on_timeout": "retry_once"
  }
}
```

### 9.3 Workflow Step Actions

| Action | Description | Required Fields | Trust Implications |
|--------|-------------|----------------|-------------------|
| `"api_call"` | Call an API endpoint | `"endpoint_ref"` → api_endpoint block's ID | Trust 3: network interaction |
| `"execute"` | Run an executable block | `"exec_ref"` → executable block's ID | Trust 2: requires sandbox |
| `"transform"` | Transform data (Mustache template) | `"params"` with Mustache expressions | Trust 1: pure string op |
| `"user_choice"` | Present options to user | `"input_var"` (data to choose from) | No trust issue — user decides |
| `"user_confirmation"` | Ask user yes/no | `"display"` (what to show) | No trust issue — user decides |
| `"validate"` | Validate data against a schema | `"schema_ref"` → schema block's ID | Trust 0: pure validation |

### 9.4 Step Field Reference

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"step_id"` | text | REQUIRED | Unique identifier within the workflow |
| `"action"` | text | REQUIRED | Action type (see table above) |
| `"purpose"` | text | REQUIRED | What this step does |
| `"endpoint_ref"` | text | CONDITIONAL | API endpoint block ID. Required for `"api_call"`. |
| `"exec_ref"` | text | CONDITIONAL | Executable block ID. Required for `"execute"`. |
| `"schema_ref"` | text | CONDITIONAL | Schema block ID. Required for `"validate"`. |
| `"params"` | map | OPTIONAL | Parameters with Mustache expressions for variable substitution |
| `"body"` | map | OPTIONAL | Request body with Mustache expressions |
| `"input_var"` | text | OPTIONAL | Variable name to read input from |
| `"output_var"` | text | OPTIONAL | Variable name to store step output |
| `"condition"` | text | OPTIONAL | Condition expression (§11). Step only executes if condition is true. |
| `"display"` | text | OPTIONAL | Template string to display to user (for confirmation steps) |

### 9.5 Variable Scope

Workflow variables are scoped to the workflow execution. Each step can:
- **Read** variables from previous steps via `"input_var"` or Mustache expressions in `"params"`/`"body"`/`"display"`
- **Write** a result to a variable via `"output_var"`

Variables persist for the duration of the workflow and are discarded when the workflow completes.

**Example variable flow:**

```
Step 1 (browse):    output_var = "products"       → products = [...]
Step 2 (select):    input_var = "products"
                    output_var = "selected_product" → selected_product = {name: "Lion's Mane", ...}
Step 3 (check):     params.slug = "{{selected_product.slug}}"
                    output_var = "product_details"  → product_details = {price: 29.90, stock: 150, ...}
Step 4 (shipping):  params = "{{product_details.specs.weight_grams}}"
                    output_var = "shipping_info"    → shipping_info = {cost: 4.90, days: 3, ...}
Step 5 (confirm):   display uses all previous vars
Step 6 (submit):    body uses product_details + user info
```

---

## 10. Constraint Block

### 10.1 Overview

**Type code:** `"constraint"` | **Trust level:** 0 (declarative)

Business rules in structured logical format. An agent integrates these rules into its reasoning — for example, checking cart totals against minimum order amounts, applying free shipping thresholds, or enforcing age restrictions.

Constraints use a **non-Turing-complete expression language** (§11) that is safe to evaluate without sandboxing.

### 10.2 Structure

```cbor-diag
{
  "t": "constraint",
  "trust": 0,
  "constraint_id": "order_rules",
  "purpose": "Business rules for product ordering on Verdetao",
  "rules": [
    {
      "rule_id": "min_order",
      "condition": "order.total < 20.00",
      "action": "reject",
      "message": "Minimum order amount is 20.00 EUR"
    },
    {
      "rule_id": "max_quantity",
      "condition": "item.quantity > 10",
      "action": "reject",
      "message": "Maximum 10 units per product per order"
    },
    {
      "rule_id": "free_shipping",
      "condition": "order.total >= 50.00 AND shipping.country IN ['FR', 'ES', 'DE', 'IT']",
      "action": "apply",
      "effect": "shipping.cost = 0",
      "message": "Free shipping for EU orders over 50 EUR"
    },
    {
      "rule_id": "age_restriction",
      "condition": "product.category == 'supplements'",
      "action": "require",
      "effect": "user.age >= 18",
      "message": "Dietary supplements require buyer to be 18+"
    }
  ]
}
```

### 10.3 Field Reference

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"constraint"` |
| `"trust"` | uint | REQUIRED | `0` |
| `"constraint_id"` | text | REQUIRED | Unique identifier |
| `"purpose"` | text | REQUIRED | What these rules govern |
| `"rules"` | array | REQUIRED | Array of rule definitions |

Each rule:

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"rule_id"` | text | REQUIRED | Unique rule identifier |
| `"condition"` | text | REQUIRED | Boolean expression in constraint language (§11) |
| `"action"` | text | REQUIRED | `"reject"`, `"apply"`, `"require"`, `"warn"` |
| `"effect"` | text | OPTIONAL | What happens when the rule triggers |
| `"message"` | text | REQUIRED | Human-readable explanation |

### 10.4 Rule Actions

| Action | Meaning | Agent Behavior |
|--------|---------|---------------|
| `"reject"` | Block the operation if condition is true | Agent MUST NOT proceed |
| `"apply"` | Apply an effect if condition is true | Agent applies the effect (e.g., set shipping to 0) |
| `"require"` | Require a condition to be true before proceeding | Agent MUST verify the effect condition |
| `"warn"` | Warn but allow proceeding | Agent SHOULD inform user but MAY proceed |

---

## 11. Constraint Expression Language

### 11.1 Overview

Constraint conditions use a **minimal expression language** that is intentionally NOT Turing-complete. It supports comparisons, boolean logic, and membership tests — nothing more.

### 11.2 Operators

| Category | Operators | Example |
|----------|-----------|---------|
| **Comparison** | `==`, `!=`, `<`, `>`, `<=`, `>=` | `order.total < 20.00` |
| **Logical** | `AND`, `OR`, `NOT` | `a > 5 AND b < 10` |
| **Membership** | `IN [...]` | `country IN ['FR', 'ES', 'DE']` |
| **Dot notation** | `.` | `order.total`, `product.category` |

### 11.3 Literals

| Type | Syntax | Example |
|------|--------|---------|
| Number | Digits with optional decimal | `20.00`, `18`, `0.5` |
| String | Single quotes | `'supplements'`, `'FR'` |
| Boolean | `true`, `false` | `active == true` |
| Array (in IN) | `[...]` with comma-separated values | `['FR', 'ES', 'DE']` |

### 11.4 Operator Precedence (Highest to Lowest)

| Level | Operators | Associativity |
|-------|-----------|---------------|
| 1 (highest) | `.` (dot access) | Left |
| 2 | `NOT` | Right (prefix) |
| 3 | `<`, `>`, `<=`, `>=`, `==`, `!=`, `IN` | Left |
| 4 (lowest) | `AND`, `OR` | Left |

Parentheses `(` `)` may be used to override precedence.

### 11.5 Formal Grammar

See Appendix B for the complete EBNF grammar.

### 11.6 What is NOT Supported (by Design)

| Feature | Why Excluded |
|---------|-------------|
| Arithmetic (`+`, `-`, `*`, `/`) | Constraints are boolean, not computational |
| Assignment (`=`) | Constraints are read-only checks |
| Function calls | Would require a runtime |
| Loops | Would enable Turing-completeness |
| String concatenation | Constraints compare, not construct |
| Regular expressions | Too powerful for a constraint language |

---

## 12. Forms and Interactions

### 12.1 Overview

**Type code:** `"form"` | **Trust level:** 3 (interactive)

CBOR-Web forms enable an agent to understand, fill, validate, and submit HTML-equivalent forms without a browser. The form description is CBOR-native, the submission can be CBOR, JSON, or traditional form-encoded.

### 12.2 Structure

```cbor-diag
{
  "t": "form",
  "trust": 3,
  "form_id": "contact",
  "purpose": "Contact the Verdetao team with a question or message",
  "action": "https://api.verdetao.com/v1/contact",
  "method": "POST",
  "submit_format": "cbor",
  "fields": [
    {
      "name": "full_name",
      "type": "text",
      "label": "Nom complet",
      "required": true,
      "max_length": 100,
      "placeholder": "Jean Dupont"
    },
    {
      "name": "email",
      "type": "email",
      "label": "Adresse email",
      "required": true,
      "validation": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
    },
    {
      "name": "subject",
      "type": "select",
      "label": "Sujet",
      "required": true,
      "options": [
        {"label": "Question sur un produit", "value": "question"},
        {"label": "Suivi de commande", "value": "order"},
        {"label": "Commande en gros", "value": "wholesale"},
        {"label": "Autre", "value": "other"}
      ]
    },
    {
      "name": "message",
      "type": "textarea",
      "label": "Message",
      "required": true,
      "min_length": 20,
      "max_length": 5000
    },
    {
      "name": "newsletter",
      "type": "checkbox",
      "label": "S'inscrire a la newsletter",
      "required": false,
      "default": false
    }
  ],
  "success_message": "Votre message a ete envoye. Nous vous repondrons sous 24h.",
  "captcha": {
    "type": "none",
    "note": "CBOR-Web submissions validated by token, no CAPTCHA needed"
  }
}
```

### 12.3 Field Reference (Form Block)

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"form"` |
| `"trust"` | uint | REQUIRED | `3` |
| `"form_id"` | text | REQUIRED | Unique identifier. Referenced by workflows. |
| `"purpose"` | text | REQUIRED | What this form does |
| `"action"` | text | REQUIRED | Submission URL (`https://`). See CBOR-WEB-SECURITY.md §11.3 for validation. |
| `"method"` | text | REQUIRED | HTTP method: `"POST"`, `"PUT"`, `"PATCH"` |
| `"submit_format"` | text | REQUIRED | `"cbor"`, `"json"`, or `"form"` |
| `"fields"` | array | REQUIRED | Array of field definitions |
| `"success_message"` | text | OPTIONAL | Message to display after successful submission |
| `"captcha"` | map | OPTIONAL | CAPTCHA configuration. `"type": "none"` for token-authenticated submissions. |

### 12.4 Form Field Types

| Type | Description | Specific Keys |
|------|-------------|---------------|
| `"text"` | Single-line text input | `"min_length"`, `"max_length"`, `"pattern"`, `"placeholder"` |
| `"textarea"` | Multi-line text | `"min_length"`, `"max_length"` |
| `"email"` | Email address | `"validation"` (regex) |
| `"tel"` | Phone number | `"validation"` (regex) |
| `"number"` | Numeric input | `"min"`, `"max"`, `"step"` |
| `"select"` | Dropdown / single choice | `"options"` = `[{"value": text, "label": text}]` |
| `"multi_select"` | Multiple choice | `"options"`, `"min_selections"`, `"max_selections"` |
| `"checkbox"` | Boolean toggle | `"default"` (bool) |
| `"date"` | Date input | `"min_date"`, `"max_date"` (ISO 8601 date strings) |
| `"file"` | File upload | `"accepted_types"` (MIME array), `"max_size"` (bytes) |
| `"hidden"` | Hidden field | `"value"` (pre-set value) |

### 12.5 Form Field Common Keys

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"name"` | text | REQUIRED | Field identifier (used in submission) |
| `"type"` | text | REQUIRED | Field type (see table above) |
| `"label"` | text | REQUIRED | Human-readable label |
| `"required"` | bool | REQUIRED | Whether the field must be filled |
| `"placeholder"` | text | OPTIONAL | Placeholder text |
| `"default"` | any | OPTIONAL | Default value |
| `"validation"` | text | OPTIONAL | Regex pattern for custom validation |

### 12.6 Submission Format

When `"submit_format"` is `"cbor"`, the agent submits the form as a CBOR-encoded map:

```cbor-diag
; POST https://api.verdetao.com/v1/contact
; Content-Type: application/cbor
{
  "email": "jean@example.com",
  "full_name": "Jean Dupont",
  "message": "Bonjour, je voudrais savoir si le Lion's Mane est compatible avec...",
  "newsletter": true,
  "subject": "question"
}
```

When `"submit_format"` is `"json"`, the agent submits as `application/json`.
When `"submit_format"` is `"form"`, the agent uses `application/x-www-form-urlencoded`.

### 12.7 Form Placement

Forms are placed in page key 8:

```cbor-diag
8: [
  { "t": "form", "form_id": "contact", ... },
  { "t": "form", "form_id": "newsletter", ... }
]
```

---

## 13. Commerce Protocol

### 13.1 Overview

An agent SHOULD be able to browse a product catalog, understand pricing, check availability, and initiate a purchase — all through CBOR-Web, without HTML rendering.

Commerce data is placed in page key 9.

### 13.2 Product Block

**Type code:** `"product"` | **Trust level:** 0 (declarative)

```cbor-diag
{
  "t": "product",
  "trust": 0,
  "product_id": "lions-mane-90",
  "name": "Lion's Mane — Criniere de Lion",
  "slug": "lions-mane",
  "description": "Extrait de Hericium erinaceus concentre 10:1, 90 capsules bio",
  "price": 29.90,
  "currency": "EUR",
  "availability": "in_stock",
  "quantity_available": 150,
  "variants": [
    {
      "variant_id": "lions-mane-30",
      "name": "Lion's Mane — 30 capsules",
      "price": 12.90,
      "availability": "in_stock"
    },
    {
      "variant_id": "lions-mane-180",
      "name": "Lion's Mane — 180 capsules",
      "price": 49.90,
      "availability": "low_stock",
      "quantity_available": 8
    }
  ],
  "images": [
    {"alt": "Flacon Lion's Mane face", "semantic_role": "product_photo", "src": "https://verdetao.com/img/lm-front.webp"},
    {"alt": "Flacon Lion's Mane dos, etiquette", "semantic_role": "product_photo", "src": "https://verdetao.com/img/lm-back.webp"}
  ],
  "categories": ["champignons-fonctionnels", "nootropiques"],
  "specs": {
    "capsule_count": 90,
    "concentration": "10:1",
    "serving_size": "2 capsules",
    "servings_per_container": 45,
    "weight_grams": 95
  },
  "certifications": ["Bio EU", "Vegan", "Sans OGM", "GMP"],
  "rating": {"average": 4.7, "count": 89}
}
```

### 13.3 Product Field Reference

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"product"` |
| `"trust"` | uint | REQUIRED | `0` |
| `"product_id"` | text | REQUIRED | Unique product identifier |
| `"name"` | text | REQUIRED | Product display name |
| `"slug"` | text | REQUIRED | URL-safe identifier |
| `"description"` | text | REQUIRED | Product description |
| `"price"` | number | REQUIRED | Price in the specified currency |
| `"currency"` | text | REQUIRED | ISO 4217 currency code |
| `"availability"` | text | REQUIRED | Stock status (see §13.4) |
| `"quantity_available"` | uint | OPTIONAL | Exact stock count |
| `"variants"` | array | OPTIONAL | Product variants (sizes, quantities, etc.) |
| `"images"` | array | OPTIONAL | Product images with semantic roles |
| `"categories"` | array | OPTIONAL | Category slugs |
| `"specs"` | map | OPTIONAL | Technical specifications |
| `"certifications"` | array | OPTIONAL | Certification labels |
| `"rating"` | map | OPTIONAL | `{"average": number, "count": uint}` |

### 13.4 Availability Status

| Value | Description | Agent Interpretation |
|-------|-------------|---------------------|
| `"in_stock"` | Available for immediate purchase | Proceed with order |
| `"low_stock"` | Limited availability (< 10 units) | Proceed but warn user |
| `"out_of_stock"` | Currently unavailable | Do not attempt order |
| `"pre_order"` | Available for pre-order | Proceed with pre-order notice |
| `"discontinued"` | No longer sold | Do not attempt order, suggest alternatives |

### 13.5 Cart Action Block

**Type code:** `"cart_action"` | **Trust level:** 3 (interactive)

Describes how an agent can add products to cart or initiate purchase.

```cbor-diag
{
  "t": "cart_action",
  "trust": 3,
  "action": "add_to_cart",
  "endpoint": "https://api.verdetao.com/v1/cart/add",
  "method": "POST",
  "body_schema": {
    "product_id": {"type": "string", "required": true, "description": "Product ID"},
    "variant_id": {"type": "string", "required": false, "description": "Variant ID"},
    "quantity": {"type": "integer", "min": 1, "max": 10, "default": 1, "required": false, "description": "Quantity"}
  },
  "auth": {
    "type": "session",
    "description": "Session cookie or bearer token from login"
  }
}
```

### 13.6 Commerce Placement

Commerce data is placed in page key 9:

```cbor-diag
9: {
  "products": [
    { "t": "product", ... },
    { "t": "product", ... }
  ],
  "cart_actions": [
    { "t": "cart_action", "action": "add_to_cart", ... },
    { "t": "cart_action", "action": "checkout", ... }
  ],
  "checkout_url": "https://verdetao.com/checkout",
  "payment_methods": ["card", "paypal", "bank_transfer"],
  "shipping_zones": [
    {"base_cost": 4.90, "countries": ["FR", "ES", "DE", "IT", "PT", "BE", "NL", "AT"], "free_above": 50.00, "zone": "EU"},
    {"base_cost": 12.25, "countries": ["*"], "free_above": null, "zone": "international"}
  ]
}
```

### 13.7 Complete Commerce Workflow Example

A typical agent-driven purchase combines several generative blocks:

```
1. Agent reads product blocks (trust 0) → understands catalog
2. Agent reads constraint blocks (trust 0) → understands business rules
3. User says "I want to buy Lion's Mane"
4. Agent checks availability (trust 0) → "in_stock", 150 units
5. Agent executes calculate_shipping (trust 2, sandboxed) → 4.90 EUR
6. Agent applies constraints → total 29.90 EUR > 20 EUR minimum ✅
7. Agent checks free_shipping rule → 29.90 < 50 EUR → shipping applies
8. Agent confirms with user: "29.90 + 4.90 = 34.80 EUR, 3 days delivery"
9. User confirms
10. Agent calls cart_action (trust 3) → adds to cart
11. Agent redirects to checkout_url
```

---

## 14. Capability Declaration

### 14.1 Overview

The manifest key 7 declares what a site offers. An agent reads this **once** and knows exactly what interactions are possible — before downloading any page.

### 14.2 Structure

```cbor-diag
7: {
  "api": {
    "auth_required": true,
    "available": true,
    "docs_url": "https://api.verdetao.com/docs",
    "endpoint_count": 12
  },
  "commerce": {
    "available": true,
    "checkout_type": "api",
    "currencies": ["EUR"],
    "product_count": 15
  },
  "conformance": "full",
  "forms": {
    "available": true,
    "types": ["contact", "newsletter"]
  },
  "generative": {
    "constraints": true,
    "executables": true,
    "schemas": true,
    "templates": true,
    "workflows": true
  },
  "languages": ["fr", "es", "en"],
  "live": false,
  "multimedia": {
    "audio": false,
    "documents": true,
    "images": true,
    "live_streams": false,
    "video": true
  },
  "static_content": true
}
```

### 14.3 Capability Fields

| Capability | Type | Description |
|-----------|------|-------------|
| `"static_content"` | bool | Site has text pages (always true for CBOR-Web) |
| `"multimedia"` | map | Multimedia capabilities: `"images"`, `"video"`, `"audio"`, `"documents"`, `"live_streams"` |
| `"api"` | map | API availability: `"available"`, `"auth_required"`, `"endpoint_count"`, `"docs_url"` |
| `"generative"` | map | Generative blocks: `"templates"`, `"schemas"`, `"executables"`, `"workflows"`, `"constraints"` |
| `"live"` | bool | Real-time streaming channels available (see CBOR-WEB-MULTIMEDIA.md §8) |
| `"commerce"` | map | Commerce: `"available"`, `"currencies"`, `"checkout_type"`, `"product_count"` |
| `"forms"` | map | Forms: `"available"`, `"types"` |
| `"languages"` | array | Available languages (redundant with site metadata for quick access) |
| `"conformance"` | text | `"minimal"`, `"standard"`, `"full"` |

### 14.4 Agent Decision Making

An agent SHOULD use capabilities to make efficient decisions:

| Agent Need | Capability Check | Action if False |
|-----------|-----------------|----------------|
| Browse products | `commerce.available` | Skip site for commerce queries |
| Execute workflows | `generative.workflows` | Fall back to manual steps |
| Watch video transcriptions | `multimedia.video` | Skip video-related queries |
| Submit contact form | `forms.available` | Look for email in site metadata |
| Get real-time stock | `live` | Use static stock from product blocks |

---

## 15. Generative Block Placement

### 15.1 Page Key 7: Generative Blocks

All generative blocks (template, schema, api_endpoint, executable, workflow, constraint) are placed in page key 7:

```cbor-diag
55799({
  0: "cbor-web-page",
  1: 2,
  2: { ... },               ; identity
  3: { ... },               ; metadata
  4: [ ... ],               ; editorial content (core + multimedia)
  5: { ... },               ; links
  6: { ... },               ; structured data
  7: [                      ; GENERATIVE BLOCKS
    { "t": "template", "trust": 1, ... },
    { "t": "schema", "trust": 0, ... },
    { "t": "api_endpoint", "trust": 3, ... },
    { "t": "executable", "trust": 2, ... },
    { "t": "workflow", "trust": 3, ... },
    { "t": "constraint", "trust": 0, ... }
  ],
  8: [                      ; FORMS (key 8)
    { "t": "form", "trust": 3, ... }
  ],
  9: {                      ; COMMERCE (key 9)
    "products": [ { "t": "product", "trust": 0, ... } ],
    "cart_actions": [ { "t": "cart_action", "trust": 3, ... } ],
    "shipping_zones": [ ... ]
  }
})
```

### 15.2 Separation of Concerns

| Key | Content | Trust Range | Agent That Reads It |
|-----|---------|-------------|-------------------|
| 4 | Editorial content + multimedia | 0 | Every agent |
| 7 | Generative blocks | 0-3 | Agents with generative capabilities |
| 8 | Forms | 3 | Agents that can submit forms |
| 9 | Commerce | 0-3 | Agents that handle purchases |

An agent that only needs editorial content reads key 4 and ignores keys 7, 8, 9.

### 15.3 Manifest Flags

For each page, the manifest page entry SHOULD include capability flags:

```cbor-diag
{
  "path": "/products/lions-mane",
  "title": "Lion's Mane",
  "access": "token",
  "has_generative": true,
  "has_forms": false,
  "has_commerce": true,
  ...
}
```

These flags enable an agent to filter pages by capability without downloading them.

---

## Appendix A: Mustache Subset EBNF Grammar

This is the **formal grammar** for the `"output_template"` string in template blocks (§4). Implementations MUST conform to this grammar.

```ebnf
(* CBOR-Web Mustache Subset Grammar — EBNF *)
(* Based on Mustache (https://mustache.github.io/) with safety restrictions *)

template        = { segment } ;

segment         = text_segment | tag ;

text_segment    = text_char , { text_char } ;
text_char       = ? any character except '{{' ? ;

tag             = variable_tag
                | section_tag
                | inverted_tag
                | each_tag
                | comment_tag ;

(* Variable: {{name}} or {{object.field}} *)
variable_tag    = "{{" , identifier , "}}" ;

(* Section (if): {{#if name}}...{{/if}} *)
section_tag     = "{{#if " , identifier , "}}" , template , "{{/if}}" ;

(* Inverted section (unless): {{^if name}}...{{/if}} *)
inverted_tag    = "{{^if " , identifier , "}}" , template , "{{/if}}" ;

(* Each loop: {{#each name}}...{{/each}} *)
each_tag        = "{{#each " , identifier , "}}" , template , "{{/each}}" ;

(* Comment: {{! text }} *)
comment_tag     = "{{!" , { comment_char } , "}}" ;
comment_char    = ? any character except '}}' ? ;

(* Identifiers support dot notation for nested access *)
identifier      = name , { "." , name } ;
name            = letter , { letter | digit | "_" } ;

letter          = "a" | "b" | ... | "z" | "A" | "B" | ... | "Z" ;
digit           = "0" | "1" | ... | "9" ;

(* Special identifiers inside loops *)
(* "this" — refers to current array element *)
(* "this.field" — accesses field of current element (if element is an object) *)
(* "@index" — zero-based index of current element (OPTIONAL) *)
```

**Constraints:**
- Maximum nesting depth: 8 levels
- Maximum template length: 100 KB
- Maximum variable name length: 100 characters
- `{{` and `}}` are the ONLY delimiters (no custom delimiters)

---

## Appendix B: Constraint Expression EBNF Grammar

This is the **formal grammar** for the `"condition"` and `"effect"` strings in constraint blocks (§10).

```ebnf
(* CBOR-Web Constraint Expression Grammar — EBNF *)
(* Non-Turing-complete: comparisons + boolean logic only *)

expression      = or_expr ;

or_expr         = and_expr , { "OR" , and_expr } ;

and_expr        = not_expr , { "AND" , not_expr } ;

not_expr        = "NOT" , not_expr
                | comparison ;

comparison      = value , comp_operator , value
                | value , "IN" , array_literal
                | "(" , expression , ")" ;

comp_operator   = "==" | "!=" | "<" | ">" | "<=" | ">=" ;

value           = field_access | literal ;

field_access    = identifier , { "." , identifier } ;

identifier      = letter , { letter | digit | "_" } ;

literal         = number_literal | string_literal | boolean_literal ;

number_literal  = [ "-" ] , digit , { digit } , [ "." , digit , { digit } ] ;

string_literal  = "'" , { string_char } , "'" ;
string_char     = ? any character except "'" ? ;

boolean_literal = "true" | "false" ;

array_literal   = "[" , literal , { "," , literal } , "]" ;

(* Effect expressions (for "effect" field in "apply" and "require" rules) *)
effect_expr     = field_access , "=" , value
                | field_access , comp_operator , value ;

letter          = "a" | "b" | ... | "z" | "A" | "B" | ... | "Z" ;
digit           = "0" | "1" | ... | "9" ;
```

**Examples parsed against this grammar:**

| Expression | Parsed As |
|-----------|-----------|
| `order.total < 20.00` | comparison(field_access("order", "total"), "<", number(20.00)) |
| `item.quantity > 10` | comparison(field_access("item", "quantity"), ">", number(10)) |
| `order.total >= 50.00 AND shipping.country IN ['FR', 'ES']` | and(comparison(..., ">=", ...), in(field_access(...), array('FR', 'ES'))) |
| `NOT active` | not(comparison(field_access("active"))) — note: bare field access treated as truthy check |
| `product.category == 'supplements'` | comparison(field_access("product", "category"), "==", string('supplements')) |

---

## Appendix C: Generative CDDL Schema

```cddl
; ══════════════════════════════════════════════════════════
; CBOR-Web Generative Specification v2.1 — CDDL Schema
; Document: CBOR-WEB-GENERATIVE.md, Appendix C
; ══════════════════════════════════════════════════════════

; ── Variable Type System (shared by all generative blocks) ──

variable-def = {
  "type" => "string" / "number" / "integer" / "boolean" / "array" / "object",
  "required" => bool,
  "description" => tstr,
  ? "default" => any,
  ? "enum" => [+ any],
  ? "items" => tstr,
  ? "min" => number,
  ? "max" => number,
  ? "min_length" => uint,
  ? "max_length" => uint,
  ? "pattern" => tstr,
  ? "format" => tstr,
  ? "fields" => { + tstr => variable-def },
  * tstr => any
}

; ── Generative Blocks ──

generative-block = template-block / schema-block / api-endpoint-block /
                   executable-block / workflow-block / constraint-block

template-block = {
  "t" => "template",
  "trust" => 1,
  "template_id" => tstr,
  "purpose" => tstr,
  "variables" => { + tstr => variable-def },
  ? "output_template" => tstr,
  ? "example_instantiation" => { * tstr => any },
  * tstr => any
}

schema-block = {
  "t" => "schema",
  "trust" => 0,
  "schema_id" => tstr,
  "purpose" => tstr,
  ? "version" => uint,
  "fields" => { + tstr => variable-def },
  ? "primary_key" => tstr,
  ? "required" => [+ tstr],
  ? "indexes" => [+ tstr],
  * tstr => any
}

api-endpoint-block = {
  "t" => "api_endpoint",
  "trust" => 3,
  "endpoint_id" => tstr,
  "purpose" => tstr,
  "method" => "GET" / "POST" / "PUT" / "PATCH" / "DELETE",
  "url" => tstr,
  ? "url_params" => { + tstr => variable-def },
  ? "query_params" => { + tstr => variable-def },
  ? "body" => { + tstr => variable-def },
  ? "headers" => { + tstr => tstr },
  ? "auth" => { "type" => tstr, ? "description" => tstr, * tstr => any },
  "response" => {
    "content_type" => tstr,
    ? "schema_ref" => tstr,
    ? "example" => { * tstr => any },
    * tstr => any
  },
  ? "rate_limit" => { * tstr => uint },
  ? "errors" => [+ { "code" => uint, "description" => tstr }],
  * tstr => any
}

sandbox-requirements = {
  ? "network" => bool,
  ? "filesystem" => bool,
  ? "max_execution_time_ms" => uint,
  ? "max_memory_mb" => uint,
  ? "required_packages" => [+ tstr],
  * tstr => any
}

test-case = {
  "inputs" => { * tstr => any },
  "expected_output" => { * tstr => any },
  * tstr => any
}

executable-block = {
  "t" => "executable",
  "trust" => 2,
  "exec_id" => tstr,
  "purpose" => tstr,
  "lang" => tstr,
  "inputs" => { + tstr => variable-def },
  "outputs" => { + tstr => variable-def },
  "code" => tstr,
  ? "test_cases" => [+ test-case],
  "sandbox_requirements" => sandbox-requirements,
  * tstr => any
}

workflow-step = {
  "step_id" => tstr,
  "action" => "api_call" / "execute" / "transform" / "user_choice" / "user_confirmation" / "validate",
  "purpose" => tstr,
  ? "endpoint_ref" => tstr,
  ? "exec_ref" => tstr,
  ? "schema_ref" => tstr,
  ? "params" => { * tstr => any },
  ? "body" => { * tstr => any },
  ? "input_var" => tstr,
  ? "output_var" => tstr,
  ? "condition" => tstr,
  ? "display" => tstr,
  * tstr => any
}

workflow-block = {
  "t" => "workflow",
  "trust" => 3,
  "workflow_id" => tstr,
  "purpose" => tstr,
  "steps" => [+ workflow-step],
  ? "error_handling" => { * tstr => tstr },
  * tstr => any
}

constraint-rule = {
  "rule_id" => tstr,
  "condition" => tstr,
  "action" => "reject" / "apply" / "require" / "warn",
  ? "effect" => tstr,
  "message" => tstr,
  * tstr => any
}

constraint-block = {
  "t" => "constraint",
  "trust" => 0,
  "constraint_id" => tstr,
  "purpose" => tstr,
  "rules" => [+ constraint-rule],
  * tstr => any
}

; ── Form Blocks ──

form-field = {
  "name" => tstr,
  "type" => "text" / "textarea" / "email" / "tel" / "number" / "select" /
            "multi_select" / "checkbox" / "date" / "file" / "hidden",
  "label" => tstr,
  "required" => bool,
  ? "max_length" => uint,
  ? "min_length" => uint,
  ? "min" => number,
  ? "max" => number,
  ? "step" => number,
  ? "pattern" => tstr,
  ? "validation" => tstr,
  ? "placeholder" => tstr,
  ? "default" => any,
  ? "options" => [+ { "value" => tstr, "label" => tstr }],
  ? "min_selections" => uint,
  ? "max_selections" => uint,
  ? "accepted_types" => [+ tstr],
  ? "max_size" => uint,
  ? "value" => any,
  ? "min_date" => tstr,
  ? "max_date" => tstr,
  * tstr => any
}

form-block = {
  "t" => "form",
  "trust" => 3,
  "form_id" => tstr,
  "purpose" => tstr,
  "action" => tstr,
  "method" => "POST" / "PUT" / "PATCH",
  "submit_format" => "cbor" / "json" / "form",
  "fields" => [+ form-field],
  ? "success_message" => tstr,
  ? "captcha" => { "type" => tstr, ? "note" => tstr },
  * tstr => any
}

; ── Commerce ──

availability-status = "in_stock" / "low_stock" / "out_of_stock" / "pre_order" / "discontinued"

product-variant = {
  "variant_id" => tstr,
  "name" => tstr,
  "price" => number,
  "availability" => availability-status,
  ? "quantity_available" => uint,
  * tstr => any
}

product-block = {
  "t" => "product",
  "trust" => 0,
  "product_id" => tstr,
  "name" => tstr,
  "slug" => tstr,
  "description" => tstr,
  "price" => number,
  "currency" => tstr,
  "availability" => availability-status,
  ? "quantity_available" => uint,
  ? "variants" => [+ product-variant],
  ? "images" => [+ { "src" => tstr, "semantic_role" => tstr, "alt" => tstr }],
  ? "categories" => [+ tstr],
  ? "specs" => { * tstr => any },
  ? "certifications" => [+ tstr],
  ? "rating" => { "average" => number, "count" => uint },
  * tstr => any
}

cart-action-block = {
  "t" => "cart_action",
  "trust" => 3,
  "action" => tstr,
  "endpoint" => tstr,
  "method" => "POST" / "PUT",
  "body_schema" => { + tstr => variable-def },
  ? "auth" => { "type" => tstr, ? "description" => tstr, * tstr => any },
  * tstr => any
}

commerce-data = {
  ? "products" => [+ product-block],
  ? "cart_actions" => [+ cart-action-block],
  ? "checkout_url" => tstr,
  ? "payment_methods" => [+ tstr],
  ? "shipping_zones" => [+ {
    "zone" => tstr,
    "countries" => [+ tstr],
    "base_cost" => number,
    ? "free_above" => number / null
  }],
  * tstr => any
}

; ── Capability Declaration (manifest key 7) ──

capabilities = {
  ? "static_content" => bool,
  ? "multimedia" => { ? "images" => bool, ? "video" => bool, ? "audio" => bool, ? "documents" => bool, ? "live_streams" => bool, * tstr => any },
  ? "api" => { ? "available" => bool, ? "auth_required" => bool, ? "endpoint_count" => uint, ? "docs_url" => tstr, * tstr => any },
  ? "generative" => { ? "templates" => bool, ? "schemas" => bool, ? "executables" => bool, ? "workflows" => bool, ? "constraints" => bool, * tstr => any },
  ? "live" => bool,
  ? "commerce" => { ? "available" => bool, ? "currencies" => [+ tstr], ? "checkout_type" => tstr, ? "product_count" => uint, * tstr => any },
  ? "forms" => { ? "available" => bool, ? "types" => [+ tstr], * tstr => any },
  ? "languages" => [+ tstr],
  ? "conformance" => "minimal" / "standard" / "full",
  * tstr => any
}
```

---

## Appendix D: Generative Test Vectors

Binary test vectors for generative blocks are available in the repository at `test-vectors/`. These were generated using deterministic CBOR encoding (CBOR-WEB-CORE.md §3.1) and cross-validated by Rust (ciborium) and Python (cbor2).

See CBOR-WEB-REFERENCE.md for the complete test vector suite.

---

## Appendix E: Complete Examples

### E.1 Template + Instantiation (Product Page Generator)

The template from §4.2, instantiated with Reishi data:

**Variables:**
```cbor-diag
{
  "product_name": "Reishi",
  "latin_name": "Ganoderma lucidum",
  "price": 24.90,
  "capsule_count": 90,
  "concentration": "8:1",
  "benefits": ["Soutient le systeme immunitaire", "Favorise la relaxation", "Proprietes adaptogenes"],
  "certifications": ["Bio EU", "Vegan"]
}
```

**Generated output:**
```markdown
# Reishi

*Ganoderma lucidum*

Reishi est un extrait concentre 8:1, conditionne en flacons de 90 capsules.

## Bienfaits

- Soutient le systeme immunitaire
- Favorise la relaxation
- Proprietes adaptogenes

## Prix

**24.9 EUR**

## Certifications
- Bio EU
- Vegan
```

### E.2 Complete Order Workflow

See §9.2 for the full workflow definition. Here is the step-by-step execution trace:

```
Step 1 (browse):
  Action: api_call → GET /v1/products
  Output: products = [{name: "Lion's Mane", slug: "lions-mane", ...}, {name: "Reishi", ...}]

Step 2 (select):
  Action: user_choice
  Display: "Choose a product: 1. Lion's Mane (29.90 EUR) 2. Reishi (24.90 EUR)"
  User selects: 1
  Output: selected_product = {name: "Lion's Mane", slug: "lions-mane"}

Step 3 (check_stock):
  Action: api_call → GET /v1/products/lions-mane
  Output: product_details = {name: "Lion's Mane", price: 29.90, stock: 150, specs: {weight_grams: 95}}
  Condition: product_details.stock > 0 → 150 > 0 → TRUE ✅

Step 4 (shipping):
  Action: execute → calculate_shipping(weight_grams=95, country_code="FR")
  Sandbox: no network, no filesystem, 1s timeout, 64 MB memory
  Output: shipping_info = {shipping_cost: 4.90, estimated_days: 3, carrier: "Correos"}

Step 5 (confirm):
  Action: user_confirmation
  Display: "Product: Lion's Mane\nPrice: 29.90 EUR\nShipping: 4.90 EUR\nTotal: 34.80 EUR\nDelivery: 3 days"
  User confirms: true
  Output: confirmed = true

Step 6 (submit):
  Condition: confirmed == true → TRUE ✅
  Action: api_call → POST /v1/orders {product_id: "...", quantity: 1, shipping_country: "FR"}
  Output: order_result = {order_id: "ORD-2026-0042", status: "confirmed"}

Workflow complete. Order ORD-2026-0042 confirmed.
```

---

## References

### Normative References

- **[RFC 8949]** Bormann, C. and P. Hoffman, "Concise Binary Object Representation (CBOR)", STD 94, December 2020.
- **[RFC 8610]** Birkholz, H., et al., "Concise Data Definition Language (CDDL)", June 2019.
- **[Mustache]** "Mustache — Logic-less templates", https://mustache.github.io/

### Informative References

- **[CBOR-WEB-CORE.md]** CBOR-Web Core Specification v2.1.
- **[CBOR-WEB-SECURITY.md]** CBOR-Web Security Specification v2.1.
- **[CBOR-WEB-MULTIMEDIA.md]** CBOR-Web Multimedia Specification v2.1.
- **[CBOR-WEB-REFERENCE.md]** CBOR-Web Reference v2.1.
- **[OpenAPI]** "OpenAPI Specification", https://spec.openapis.org/oas/v3.1.0

---

*CBOR-Web Generative Specification v2.1 — Document 3 of 6*

*ExploDev 2026*
