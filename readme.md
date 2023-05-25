# Quite A High Level Virtual Machine (QAHLVM)

## What is this?
QAHLVM (pronounced as Kelvin) is a Virtual Machine meant for a 
backend for languages who only has created a front end or for 
developers developing a new language and want to test it out. 

## How do I use it?
QAHLVM is a rust Library, Inorder to use it you need to add it to cargo as a dependency. 
```toml
[dependencies]
qahlvm = "0.1.0"
```
Alternatively if you only have the source code you can
```toml
[dependencies]
qahlvm = { path = "path/to/qahlvm" }
```

Then you can use it in your code like this:
```rust
use qahlvm::vm::VM;
use qahlvm::ast::*;

fn main() {
    let mut vm = VirtualMachine::new();
    let instructions = vec![
        Node::FnCall("println".to_string(), vec![Eval::String("Hello World!".to_string())]),
    ];
    vm.run(instructions);
}
```

## What is the instruction set?
Instruction Set? What's that? in QAHLVM everything is basically just an AST evaluator, 
you don't have to compile it to low level instructions.

## What is the difference between this and other VMs?
QAHLVM is a very high level VM, Meaning integrating this with your language is very easy.
You work on the front end and QAHLVM works on the backend.

## What is the future of this project?
Well, I kinda don't expect this to last more than 2 years. You see, 
When I develop Projects I tend to get bored with them after a while and move on to other projects. 
But fear not, If I get bored with this, It just means I'll create A new version of this with a 
different name and a different API. It will basically be as Easy but there will massive improvements.