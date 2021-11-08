# Node template gen tool

## Notepad

basti downer tool for replacing dependence?
https://github.com/bkchr/diener

SEE HERE FOR RECENT CHANGES:
https://github.com/paritytech/substrate/pull/9461/files

Option to set commit, tag or branch as what is in cargo
https://docs.rs/git2/0.13.23/git2/struct.Reference.html#method.peel_to_tag

order of toml parsing items
https://github.com/alexcrichton/toml-rs/blob/master/Cargo.toml#L33-L36

## Desired Features
- [ ] add a table for non-semver version notes
- [ ] add ability to set the repo to use
	- [ ] set multilple (sub, polka ...) BUT we are only changing `path` deps, so this hsould not be an issue?
- [ ] Integrate with Sacha's kitties thing to generate a new tempalte with name, author, otional added stuff...
- [ ] preserve order of cargo files for formatting
- [ ] optial via CLI
	- [ ] testing pallet (builds things!!)
	- [ ] build & retain build artifacts
	- [ ] zip whole node to transport
- [ ] option to bring in upstream lock file based on tag/branch or not.
	- [ ] this will not be "right" but running a build, cargo will sort it out.
- [ ] Custom tweaks
	- [ ] Remplace `ROC` with Unit in chain spec
	- [ ] `RocLocation` to `RelayLocation` in runtime
	- [ ] support_url to change to ...?
	- [ ] Other custom bits?
	- [ ] *issue: does this need to be the same for teleporting (common good with realy chain)*