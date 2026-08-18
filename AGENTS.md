See CLAUDE.md.

Design taste: write idiomatic Rust — lean on the type system (enums, `Option`/`Result`, newtypes, borrowed slices) over runtime checks and stringly-typed values.

## Adding or improving a check

Read **CLAUDE.md § Adding or improving a check** before changing any rule after a
measurement. The short form:

A new check pointed at the `~/dev` corpus will fire in the hundreds or thousands.
**That is expected, not evidence the check is wrong.** A high count means one of
two things, and it is a question of fact you settle by reading flagged documents:

- **Bad rule** — the check misreads correct data.
- **Bad corpus** — the check reads correctly and the specs are wrong.

Do not default to either. Agents wrote most of these specs and agents do not
write good specs — that is why quoin and quire exist.

**Never widen a rule because it lowers the count.** A rule states what a good spec
looks like; it does not fit the specs that exist. Any widening needs a
justification true independent of the number. Where two forms mean the same
thing, prefer unifying the corpus on one and flagging the rest over accepting
both.

Report the precision split as a number ("sampled 10, 3 rule, 7 real"), and say
which of the two conclusions you reached and why.
