# Node template gen tool

## Notepad

Use this tool for replacing dependence?
https://github.com/bkchr/diener

SEE HERE FOR RECENT CHANGES to upstream template generator tooling:
https://github.com/paritytech/substrate/pull/9461/files

Option to set commit, tag or branch as what is in cargo
https://docs.rs/git2/0.13.23/git2/struct.Reference.html#method.peel_to_tag

Order of toml parsing items
https://github.com/alexcrichton/toml-rs/blob/master/Cargo.toml#L33-L36

CI for checking upstream, once generated ideas:
https://github.com/paritytech/substrate-archive/blob/master/.github/workflows/node-template.yml
https://github.com/paritytech/substrate-archive/blob/master/bin/node-template-archive/Cargo.toml

## Desired Features

- [ ] TOML config file to run the generator as a bin (no need to build each time to use different settings)
  - [ ] upstream `remote` repo and the local `path` to use.
    - if git repo is at `path`, use it: checkout the requested branch/tag/rev
      - first check it's remote matches, ir warn....panic? cli override
    - pull from it's set remote (if any) to update
    - if it does not, pull from the remote upstream `git` TOML field
    - if no repo at `path`, use `remote` to clone depth =1 t specific requested branch/tag/rev
    - `output` to use the `remote` explicitly in cargo files, al local path is likely not useful (can easily add new field allowing for this though)
- [ ] preserve order of cargo files for formatting
- [ ] optional via TOML:
	- [ ] build & option to retain build artifacts
	- [ ] testing pallet (keep & use build artifacts)
	- [ ] zip whole node (and option build artifacts) to transport over the wire
- [ ] optional via CLI:
	- [ ] overwrite an exiting output folder using `rsync` or similar (with an *explicit* flag to do so, default will error out before anything else starts if dir is not empty)
- [ ] option to bring in upstream lock file based on tag/branch or not.
	- [ ] this will not be "right" but running a build, cargo will sort it out?
- [ ] Custom tweaks
	- [ ] `support_url` in cli command upstream to change to ...?
	- [ ] Other custom bits?
- [ ] CLI commands to use the embedded `config/...` TOML files by name, or to set a specific file to use you provide.
