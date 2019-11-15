# Feature Design

## Project

*TODOs*:
- [ ] Create Projects
  - [ ] Store Projects
  - [ ] List Stored Projects
- [ ] Add Tasks to existing projects
- [ ] Follow Project Progress

### Development

#### Create Projects

A project needs (on a high level):
- A name
- An optional description

Simple enough.

Problem 1: Architecture

This System is meant to be flexible.
So ideally, the CLI and the Project Management modules would be very loosely coupled.
The way to do this is via Traits, I believe.
The CLI module can define a Trait, that the Project Management module's Structs then conform to.
Trait conformance can be built separatly from each struct and can be the job of a Mastro module.

Which leads us to the roadmap:
1. [ ] Implement PM module's functionality
1. [ ] Define CLI module's traits
1. [ ] Implement CLI traits on PM Structs

Problem 2:

Data storage. We can start with a `.csv` file, given its low overhead.
We can also store `.csv`s on Github so we'll keep our data safe and decentralized.

Again, let's do this right.
We should define storage traits as well, so we can replace it in the future

## CLI

- [x] CLI Argument Parsing
Quick solution: 'clap'

Alright. Seems to work well enough

NOTE: Holy shit! Rust lifetimes are magic... 
They really, really just work (tm).
And here I was thinking they'd be super hard to reason around, or very complex to use...
Nope!
