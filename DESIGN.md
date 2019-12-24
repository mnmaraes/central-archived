# Feature Design

## Project

[Next Session](#session-2)

*TODOs*:
- [x] Create Projects
- [ ] Store Projects
- [ ] List Stored Projects
- [ ] Add Tasks to existing projects
- [ ] Follow Project Progress

### Development

#### Session 1 [Nov 15, 2019]

##### Create Projects

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

It seems like the way to do this is with `csv` + `serde`.
Both have been added to our project and should be accessible through the `cargo doc --open` command

So next thing todo:

1. [ ] Define and implement our storage engine

#### Session 2

In a proper Murillo fashion, I'm rethinking the storage decision mid implementation.
`csv` is a limited format for what we're doing.
Why not use a proper database? 
Doesn't have to be something large scale or scalable or super duper.
It just has to work and be somewhat portable.
It seems to me that SQLLite fits all of that.
So why not just go with it?

Well, if we're going to be messing around with SQL, let's just go for big papa Postgres

Next thing is Diesel doesn't support async stuff.
I don't **need** async stuff, but I **want** async stuff.
Moving away from `Diesel` and towards `tokio_postgres`

#### Unnumbered Session

Problem 3 (or 0): Rearchitecting Central.
The modular architecture for the code just wasn't cutting it.
It's not just the code that needs to be decentralized.
The whole functioning of the system needs to be distributed across different processes.

This makes things fun, as I'll get to do some IPC.
Given the event driven architecture I have in mind, 
it seems that sockets would be the right way to communicate between processes.
Also thought about shared memory,
and although it really wouldn't work for events 
(because of its passive read/write nature),
we might still use it for managed models in the future.

Still need to play around with both,
to make sure I can build a robust framework on top of rust primitives.
Clearing the [context](#dec-15th) due to the arch redo.

*Context*
1. [ ] Build IPC framework
  1. [ ] Establish connection between `station` and clients
      - [x] Await connections from `central_lslls
      - tation`
      - [ ] Connect from `central_cli`
      - [ ] Send messages from `central_cli`
  1. [ ] Wrap Events


## CLI

- [x] CLI Argument Parsing
Quick solution: 'clap'

Alright. Seems to work well enough

NOTE: Holy shit! Rust lifetimes are magic... 
They really, really just work (tm).
And here I was thinking they'd be super hard to reason around, or very complex to use...
Nope!

## Archived Contexts

### Dec 15th

*Context:*

1. [x] Add Clippy to your project
1. [ ] Define and implement our storage engine
1. [ ] Implement PM module's functionality
    - [ ] Store Projects
    - [ ] List Stored Projects
    - [ ] Add Tasks to existing projects
    - [ ] Follow Project Progress
1. [ ] Define CLI module's traits
1. [ ] Implement CLI traits on PM Structs
