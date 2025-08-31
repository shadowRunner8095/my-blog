# I Had a Dream...

**TL;DR:** An edge-first JSX-based app builder that lets you  
easily build artifacts for different targets (web, mobile, desktop),  
reusing **80% of the frontend code**, with **blazing-fast performance**  
both client-side and server-side. Oh, and it supports **runtime composition of microfronts**.  

See, I like JSX. I like how it feels it’s declarative, just like HTML.  
But one thing is JSX itself, and another is what it **compiles into**.  

```jsx
// Example: JSX compiled via Babel or SWC
<div>Hello</div>
```

```js
// becomes
_jsx("div", { children: "Hello" });

```

The execution returns an object with enough information for `react-dom` to render that component.

## What about Astro or Qwik?  
Well, they have their **own compilers** that emit exactly the code they need.  

So I was wondering… could we have **one ergonomic source code** that compiles to:  

- \- Isomorphic JS that can run with an adapter in a specific environment
- \- Separated JS for client and server  
- \- NativeScript output that produces native mobile apps  
- \- A runtime/code that is the **most performant ever**  

## Isn’t that React Native + Tamagui?  

Yeah, React’s ecosystem lets us use JSX for web and mobile targets,  
and Tamagui solves the trilemma of **reusing components** using its Babel-based extractor + compiler.  

Isn’t that enough?  

I mean, Tamagui is wonderful, and the React team’s efforts with Fiber are brilliant.  
When I studied their architecture docs and source code, I saw **clever optimizations everywhere**.  

But…  

## Hydration is not for every app (web)  

Hydration reconciles what’s already in the DOM because the server already generated HTML.  
The JS code responsible for interactivity must map everything correctly.  

As Qwik’s blog states, this process is **costly**. Sometimes it’s a minor tradeoff, but for apps where **client performance is critical**, hydration can hurt.  

Qwik tries to **resume state from where the server stopped**, creating tiny JS files per operation.  
If a button is never clicked, its JS code is never downloaded.  
Startup metrics with Qwik? **Phenomenal**, because of this.  

## Server rendering must be fast  

We’re pushing storefronts to the edge, this is not new, but now we have **distributed edge workers** running near users.  
Combine this with an **efficient cache model**, and performance can be **amazing at low cost**.  

But “low” cost means **our code must be efficient**, use minimal CPU, and run **blazingly fast**.  
Next.js and Qwik are making strides here, especially for **edge-first rendering**.  

## But there is no Qwik Native  

Let’s assume Qwik solves my edge and client-side concerns.  
My dream? **JSX reused for native mobile apps**, a frontier Qwik hasn’t explored.  

## And Svelte?  

From what I’ve read, Svelte seems like the middle ground I expected beacuse it has svelte native but it has **no JSX**.  


## Is JSX that urgent?  

Templates that render seamlessly across multiple targets are good.  
But JSX has **momentum** in the JS ecosystem, making it more maintainable.  

We could have an **extra compilation layer**: JSX compiler → template markup → engine creates runtime code automatically.  
The developer doesn’t even notice what happened.  


## Platforms are different for microfront runtime composition  

- **Web:** Runtime composition is easy—just send JS and execute it.  
- **Server:** Open component ideas could work well, producing HTML, JS, and CSS generated anywhere.  
- **Native mobile:** Can’t fetch “native code” and expect it to run. NativeScript could hypothetically fetch JS, but… it feels hacky.  

## But the browser teaches us how  

HTML and CSS aren’t directly understood by your graphics card, they’re **parsed into instructions at runtime**.  
We could do the same for mobile: “templates” or markup DSLs can be fetched, and glue code composes widgets at runtime.  

## Runtimes matter  

I worry about “a runtime inside a runtime”, building widgets from templates on the fly inside V8 or some JS engine.  
Performance matters.  

Optimizations like **parsing templates when idle or preloading before navigation** are costly efforts.  
Even native ARM code requires careful strategy.  


## My approach: Web first  

Not new, is heavily inspired by decades-old concepts:  

**DSL → Intermediate → Runtime → Platform Bridge**  

## WebViews in mobile…  

Don’t make me go there.  
Ionic and Capacitor are clever, but DX isn’t my priority. Real solution? Yes. But… some people make me hate so hard webviews, I am not ready to talk about it, yet.

## If you’ve been reading this blog  

You know me: I like to **complicate my life**, just for fun.  
Do I like drama? Maybe too much 😆  
I like to break things and push them to the limit. Otherwise, I get bored.  


