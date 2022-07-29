**Describe the change**
A clear and concise description of what the change is.

**Introduced tech-debts**
List the introduced tech-debts, stating why they aren't fixed now.

**Base issue**
Resolves #0

**Related issues**

**Additional context**
Add any other context about the change here.

**Checklist**
- [ ] If its a bug, the branch is `bug/<user>/<issue>-short-title`. Example: `bug/vlopes11/28-stack-overflow`
- [ ] If its a feature, the branch is `feature/<user>/<issue>-short-title`. Example: `feature/vlopes11/83-dynamic-menu`
- [ ] I have performed a self-review of my code
- [ ] If its a bug, I have added negative tests that will demonstrate it is fixed.
- [ ] If its a feature, I have added unit and integrated tests.
- [ ] I have succinctly documented the changes.
- [ ] I have added the label `breaking change` if a public API changed either its signature or output logic.
- [ ] If I used `unsafe` code, I added a comment on the line immediately above with a `Safety` remark, explaining why its usage won't cause undefined behavior if the user follow the constraints described in the documentation.
- [ ] If my public function can panic, I added a `# Panics` section in the function documentation.
