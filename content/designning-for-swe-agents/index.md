# Designing for SWE agents

My anecdote is the following, I asigned and issue
to jules from google and it red my entire codebase
to try to understand the problem and also tried to
build the project in a virtual machine.

While this is nice I really mean it, beacuse
reading all the proejct to get context is something really good
for a coding agent, if the agente were to have some
metadata to fastly now how to get hints, like tags 
that dfirects you to specif parts of the code, woudl be ni e
to make thijs requests and reading faster.

Also some probnlems can be solved wthout the need of a entire vm with all the project specially
in some repos that have very complex dependendices.

For instance my problem was that the navar started with the animation fadeout 
and kind of gliteched, so I try to see how jules perfomend.
 It designed some solutions near to the actual solution but to prove him
 rigth, he need to spam an entire vm with all my project.

 WHile its true that a end to end test is the best way becasue you dont know
 what other interactiosn ocuur with side effects, I think there is a better aproach

 We can isolate the problem in a envirment that needs less resources and abstract the problem
 with less compelxiorty to solve a equivalent problem first.


In this case we can have a simple div, label and input with a simple implementation
of the idea behind showing things if the input is checked, and the label sends the event,
then iterate there the solutions lets say bu spawing a tailwindcss playground (they a have 
a play js script that runs in the browser) and with that first try to solve the real problem

Then, try an end-to-end (e2e) test, which might fail because we need the binary
compiled from our Rust code to generate the required HTML. Still, the agent can propose
a change or even deploy to a stable development environment and scrape that
with its Playwright sandbox to run the e2e tests.

End-to-end tests are also important—don’t misunderstand me—because there are several projects
where isolation and modularization are not possible due to certain limitations,
and a change in one part can lead to issues elsewhere.

But this also means we need e2e tests we can trust, which is another important thing to consider.

So, in summary, documentation, ideas, prompts, instruction files, LLM-specific hints,
robust tests, and automation are all needed to get the best results from these SWE agents,
in my opinion.
