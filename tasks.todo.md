- [ ] Break down test into what it is actually asserting
- [ ] Implement "S3"
- [ ] Create, delete, update make no sense without output
- [ ] Read, print make no sense with "--save"
- [ ] Consider implement piping
- [ ] Implement to directory
- [ ] QT frontend
- [ ] Web frontend
- [ ] Remove panics
- [ ] Test with real fs
  - [ ] From/to file
  - [ ] From/to dir
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


"-s" takes optional parameter. If none is given, sabe back using same source/password. If "-s" is missing completely, pipe. Piping can be binary, json, or pretty json

