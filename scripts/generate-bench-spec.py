#!/usr/bin/env python3
"""Generate a large synthetic OpenAPI 3.0 spec for benchmarking.

Emits N CRUD resources × 5 operations each (list/get/create/update/delete) and
corresponding object + enum schemas, producing a spec with roughly:
  - 5N endpoints
  - 3N schemas (one object, one enum, one NewFoo request per resource)

Usage:
  ./scripts/generate-bench-spec.py [N] > tests/fixtures/large_api.yaml

Default N = 20 → 100 endpoints, ~60 schemas.

The spec is deterministic — the same N always produces the same YAML.
"""

import sys

N = int(sys.argv[1]) if len(sys.argv) > 1 else 20


def resource(i: int) -> dict[str, object]:
    name = f"Resource{i}"
    new_name = f"New{name}"
    status = f"{name}Status"
    lc = f"resource{i}"
    plural = f"{lc}s"
    return {
        "name": name,
        "new_name": new_name,
        "status": status,
        "lc": lc,
        "plural": plural,
    }


def emit() -> str:
    lines: list[str] = []
    lines.append('openapi: "3.0.3"')
    lines.append("info:")
    lines.append(f"  title: Large synthetic API ({N} resources)")
    lines.append('  version: "1.0.0"')
    lines.append("servers:")
    lines.append("  - url: https://example.com/v1")
    lines.append("paths:")

    for i in range(1, N + 1):
        r = resource(i)
        plural = r["plural"]
        name = r["name"]
        new_name = r["new_name"]
        # list
        lines.append(f"  /{plural}:")
        lines.append("    get:")
        lines.append(f"      operationId: list{name}s")
        lines.append(f"      tags: [{plural}]")
        lines.append("      parameters:")
        lines.append("        - name: limit")
        lines.append("          in: query")
        lines.append("          required: false")
        lines.append("          schema: {type: integer}")
        lines.append("        - name: offset")
        lines.append("          in: query")
        lines.append("          required: false")
        lines.append("          schema: {type: integer}")
        lines.append("      responses:")
        lines.append('        "200":')
        lines.append("          description: OK")
        lines.append("          content:")
        lines.append("            application/json:")
        lines.append("              schema:")
        lines.append("                type: array")
        lines.append(
            f"                items: {{ $ref: '#/components/schemas/{name}' }}"
        )
        # create
        lines.append("    post:")
        lines.append(f"      operationId: create{name}")
        lines.append(f"      tags: [{plural}]")
        lines.append("      requestBody:")
        lines.append("        required: true")
        lines.append("        content:")
        lines.append("          application/json:")
        lines.append(
            f"            schema: {{ $ref: '#/components/schemas/{new_name}' }}"
        )
        lines.append("      responses:")
        lines.append('        "201":')
        lines.append("          description: Created")
        lines.append("          content:")
        lines.append("            application/json:")
        lines.append(
            f"              schema: {{ $ref: '#/components/schemas/{name}' }}"
        )
        lines.append('        "400":')
        lines.append("          description: Bad Request")
        lines.append("          content:")
        lines.append("            application/json:")
        lines.append(
            "              schema: { $ref: '#/components/schemas/Error' }"
        )
        # get
        lines.append(f"  /{plural}/{{id}}:")
        lines.append("    get:")
        lines.append(f"      operationId: get{name}")
        lines.append(f"      tags: [{plural}]")
        lines.append("      parameters:")
        lines.append("        - name: id")
        lines.append("          in: path")
        lines.append("          required: true")
        lines.append("          schema: {type: integer}")
        lines.append("      responses:")
        lines.append('        "200":')
        lines.append("          description: OK")
        lines.append("          content:")
        lines.append("            application/json:")
        lines.append(
            f"              schema: {{ $ref: '#/components/schemas/{name}' }}"
        )
        lines.append('        "404":')
        lines.append("          description: Not Found")
        lines.append("          content:")
        lines.append("            application/json:")
        lines.append(
            "              schema: { $ref: '#/components/schemas/Error' }"
        )
        # update
        lines.append("    put:")
        lines.append(f"      operationId: update{name}")
        lines.append(f"      tags: [{plural}]")
        lines.append("      parameters:")
        lines.append("        - name: id")
        lines.append("          in: path")
        lines.append("          required: true")
        lines.append("          schema: {type: integer}")
        lines.append("      requestBody:")
        lines.append("        required: true")
        lines.append("        content:")
        lines.append("          application/json:")
        lines.append(
            f"            schema: {{ $ref: '#/components/schemas/{new_name}' }}"
        )
        lines.append("      responses:")
        lines.append('        "200":')
        lines.append("          description: OK")
        lines.append("          content:")
        lines.append("            application/json:")
        lines.append(
            f"              schema: {{ $ref: '#/components/schemas/{name}' }}"
        )
        # delete
        lines.append("    delete:")
        lines.append(f"      operationId: delete{name}")
        lines.append(f"      tags: [{plural}]")
        lines.append("      parameters:")
        lines.append("        - name: id")
        lines.append("          in: path")
        lines.append("          required: true")
        lines.append("          schema: {type: integer}")
        lines.append("      responses:")
        lines.append('        "204":')
        lines.append("          description: No Content")

    lines.append("components:")
    lines.append("  schemas:")
    lines.append("    Error:")
    lines.append("      type: object")
    lines.append("      required: [code, message]")
    lines.append("      properties:")
    lines.append("        code: {type: integer}")
    lines.append("        message: {type: string}")
    lines.append("        details: {type: string}")

    for i in range(1, N + 1):
        r = resource(i)
        name = r["name"]
        new_name = r["new_name"]
        status = r["status"]
        lines.append(f"    {status}:")
        lines.append("      type: string")
        lines.append("      enum: [active, pending, archived]")
        lines.append(f"    {new_name}:")
        lines.append("      type: object")
        lines.append("      required: [name]")
        lines.append("      properties:")
        lines.append("        name: {type: string}")
        lines.append("        description: {type: string}")
        lines.append(f"        status: {{ $ref: '#/components/schemas/{status}' }}")
        lines.append(f"    {name}:")
        lines.append("      type: object")
        lines.append("      required: [id, name]")
        lines.append("      properties:")
        lines.append("        id: {type: integer}")
        lines.append("        name: {type: string}")
        lines.append("        description: {type: string, nullable: true}")
        lines.append(f"        status: {{ $ref: '#/components/schemas/{status}' }}")
        lines.append("        createdAt: {type: string, format: date-time}")

    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    sys.stdout.write(emit())
