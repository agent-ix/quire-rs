---
# corpus origin=generated-input-only
# corpus module=architecture
# corpus object_type=port
# corpus source_repository=agent-ix/spec-objects-architecture
# corpus source_commit=5f9287ddb075b5422ae018e10f8e19f2c5a70e99
# corpus source_manifest_path=spec_objects_architecture/manifest.yaml
# corpus source_manifest_sha256=3ef6a69dbdd2ec9c8ebc2112dcb698eed279bc9a3af7a617df532561216acbe3
# corpus dsl_sha256_canonical_json=302cc19e9c8d77bd6671a0ff5c04358d45411a88bfa79660f728641b8259654c
# corpus content_sha256=a6b032b26379a0691498c14944db14d9c0a33fbc576660387a56e8e6dc3ecbf1
id: EXAMPLE-PORT
title: Example Port
type: port
---
## Port
| Owner | Direction | Interface | Multiplicity |
| --- | --- | --- | --- |
| billing_service | outbound | ExamplePort | 1..1 |
