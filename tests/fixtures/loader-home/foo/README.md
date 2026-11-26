This minimal loader fixture contains a valid manifest, JSON Schema, and example
record. FR-013-AC-8 uses it to verify that duplicate tilde-expanded search
roots load the module once; the registry lookup regression uses its uppercase
`FR` archetype to exercise lowercase lookup. It is not the full eight-type
ISO module and does not discharge FR-013-AC-2.
