- [ ] Test with real fs
  - [ ] From/to file
  - [ ] From/to dir
- [ ] Break down test into what it is actually asserting
- [ ] Implement "S3"
- [ ] Create, delete, update make no sense without output
- [ ] Read, print make no sense with "--save"
- [ ] Consider implement piping
- [ ] QT frontend (check nheko as GUI for JS-style UI)
- [ ] Web frontend
- [ ] Remove panics
- [ ] Slim down store
  - [ ] Move `delete_path` recursive logic into nested_map
  - [ ] Move `is_new_entry` recursive logic into nested_map
  - [ ] (?) Path iterator for nested map
- [X] Implement "create"
- [X] Dont create empty Nested
- [X] Remove "delete_helper"
- [X] Implement save
- [X] Clippy
- [X] Avoid clones
- [X] Test arg parsing
- [X] Implement "update"
- [X] Test "update"
- [X] Update with empty Nested is same as delete
- [X] Consider moving 'ops' to 'lib/store'
  - [X] Consider how to handle auto-delete empties
- [X] Implement from directory
- [X] Make directory a "source"
- [X] Implement to directory


"-s" takes optional parameter. If none is given, sabe back using same source/password. If "-s" is missing completely, pipe. Piping can be binary, json, or pretty json

