# How I Am creating my first blog

I like to reivent the weel, that is somethign you are gonna read a lot  here.
But it does not mean I am going to code a OS (maybe another day) or create a full
crate in rust for everything I need. I will try to eliminate as many layers of abstraction as 
possible with the limited time I have.

Having claryfied that, I will use

- @tailwindcss/node for generating the css
- @tailwindcss/oxide for getting tailwindcss candidates
- syntect to syntax highlgit code snipets in build time
- pulldown-cmark to transform md to html
- Node.js 
- tsx to runn scripts
- esbuild to bundle/minify js

Basically I need some glue code for all of that

## Rust needs to be compiled
Surely uploading the compiled binary of rust is not actually a good practice, but
trying to make it run as a script thing is just too slow beacuse we need the compilation process first

An idea for that is to publish to github releases. This tool will rarely change so 
another option could be npm pacakges or the gh registry.

For the previous ideas this means I need a monorepo so that is going to be
the first step.

Another important thing is the part that reads the code, that is some
code in rust with glob that finds all the .md files, creates an content index page and then
also adds the needed metadata and uses jinja for templating repetead blocks by now